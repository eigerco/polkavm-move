// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    options::Options,
    stackless::{
        dwarf::{DIBuilder, UnresolvedPrintLogLevel},
        extensions::*,
        llvm::{self, TargetMachine},
        rttydesc::RttyContext,
        FunctionContext, RtCall, TargetPlatform,
    },
};
use codespan::Location;
use log::debug;
use move_binary_format::file_format::SignatureToken;
use move_core_types::u256::U256;
use move_model::{model as mm, ty as mty};
use move_stackless_bytecode::{
    function_target::FunctionData, stackless_bytecode as sbc,
    stackless_bytecode_generator::StacklessBytecodeGenerator,
};
use polkavm_move_native::types::{MOVE_TYPE_DESC_SIZE, MOVE_UNTYPED_VEC_DESC_SIZE};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use tiny_keccak::{Hasher, Keccak};

pub struct ModuleContext<'mm: 'up, 'up> {
    pub env: mm::ModuleEnv<'mm>,
    pub llvm_cx: &'up llvm::Context,
    pub llvm_module: &'up llvm::Module,
    pub llvm_builder: llvm::Builder,
    pub llvm_di_builder: DIBuilder<'up>,
    /// A map of move function id's to llvm function ids
    ///
    /// All functions that might be called are declared prior to function translation.
    /// This includes local functions and dependencies.
    pub fn_decls: BTreeMap<String, llvm::Function>,
    pub fn_is_entry: BTreeMap<String, bool>,
    pub expanded_functions: Vec<mm::QualifiedInstId<mm::FunId>>,
    pub target: TargetPlatform,
    pub target_machine: &'up TargetMachine,
    pub options: &'up Options,
    pub rtty_cx: RttyContext<'mm, 'up>,
    pub source: &'up str,
}

impl<'mm: 'up, 'up> ModuleContext<'mm, 'up> {
    pub fn translate(&mut self, exports: &mut Vec<String>) {
        let filename = self.env.get_source_path().to_str().expect("utf-8");
        self.llvm_module.set_source_file_name(filename);
        self.llvm_module.set_target(self.target.triple());
        self.llvm_module.set_data_layout(self.target_machine);
        debug!(
            "Translating module {} id {:?}",
            self.env.get_full_name_str(),
            self.env.get_id()
        );

        self.declare_structs();
        // self.llvm_module.declare_known_functions();

        // Declaring functions will populate list `expanded_functions` containing all
        // concrete Move functions and expanded concrete instances of generic Move functions.
        self.declare_functions(exports);

        let mut has_entry = false;

        for fn_qiid in &self.expanded_functions {
            let fn_env = self.env.env.get_function(fn_qiid.to_qualified_id());
            if fn_env.is_entry() {
                has_entry = true;
            }
            assert!(!fn_env.is_native());
            self.rtty_cx.reset_func(fn_qiid);
            let fn_cx = self.create_fn_context(fn_env, self, &fn_qiid.inst);
            fn_cx.translate();
        }

        if has_entry {
            // only generate the call selector if there is an entry function
            // Assumption: no other module contains an entry function
            self.generate_call_selector(exports);
        }

        self.llvm_di_builder
            .print_log_unresoled_types(UnresolvedPrintLogLevel::Warning);
        self.llvm_di_builder.finalize();
        self.llvm_module.finalize(); // this generates the inline ASM for the polkavm sections
        self.llvm_module.verify();
    }

    /// Generate LLVM IR struct declarations for all Move structures.
    fn declare_structs(&mut self) {
        use move_binary_format::{access::ModuleAccess, views::StructHandleView};
        let m_env = &self.env;
        let g_env = &m_env.env;

        // Collect all the externally defined structures (transitively) used within this module.
        //
        // Note that the ModuleData at ModuleEnv::data is private, while the same ModuleData is
        // public in GlobalEnv::module_data-- so we obtain it from the latter. We need access to
        // this to efficiently discover foreign structs. There is not yet a model-provided routine
        // as there is for foreign called functions.
        let mut external_sqids = BTreeSet::new();
        let mut worklist = VecDeque::new();
        let mut visited = BTreeSet::new();
        worklist.push_back(m_env.get_id());
        while let Some(mid) = worklist.pop_front() {
            let compiled_module = m_env.get_verified_module().unwrap();
            for shandle in compiled_module.struct_handles() {
                let struct_view = StructHandleView::new(compiled_module, shandle);
                let declaring_module_env = g_env
                    .find_module(&g_env.to_module_name(&struct_view.module_id()))
                    .expect("undefined module");
                let struct_env = declaring_module_env
                    .find_struct(m_env.symbol_pool().make(struct_view.name().as_str()))
                    .expect("undefined struct");
                let qid = struct_env.get_qualified_id();
                if qid.module_id != m_env.get_id() && !visited.contains(&qid.module_id) {
                    worklist.push_back(qid.module_id);
                    external_sqids.insert(qid);
                }
            }
            visited.insert(mid);
        }

        // Create a combined list of all structs (external plus local).
        //
        // Initially filter out generic structure handles (i.e., representing potentially many
        // concrete structures). The expansions will occur later when the struct definition
        // instantiations are processed.
        let has_type_params = |s_env: &mm::StructEnv| !s_env.get_type_parameters().is_empty();
        let mut local_structs: Vec<_> = m_env
            .get_structs()
            .filter_map(|s_env| (!has_type_params(&s_env)).then_some((s_env, vec![])))
            .collect();

        let mut all_structs: Vec<_> = external_sqids
            .iter()
            .map(|q| g_env.get_struct_qid(*q))
            .filter_map(|s_env| (!has_type_params(&s_env)).then_some((s_env, vec![])))
            .collect();
        all_structs.append(&mut local_structs);

        debug!(target: "structs",
               "Combined list of all structs{}",
               self.dump_all_structs(&all_structs, false),
        );

        // Visit each struct definition, creating corresponding LLVM IR struct types.
        //
        // Note that struct defintions can depend on other struct definitions. Inconveniently, the
        // order of structs given to us above by the model are not necessarily in topological order
        // of dependence.  Since we'll need a structure type to translate structure fields during
        // the visitation later, we need to ensure any dependent structure types are already
        // available. One way would be to build a dependence graph of structs and visit the nodes
        // topologically. A second way, which we adopt here, is to traverse the struct list twice.
        // That is, on the first traversal, we create opaque structs (i.e., partially formed,
        // deferring field translation). The second traversal will then fill in the struct bodies
        // where it will have all structure types previously defined.
        for (s_env, tyvec) in &all_structs {
            assert!(!has_type_params(s_env));
            let ll_name = s_env.ll_struct_name_from_raw_name(tyvec);
            self.llvm_cx.create_opaque_named_struct(&ll_name);
        }

        let create_opaque_named_struct = |s_env: &mm::StructEnv, tys: &[mty::Type]| {
            // Skip the structs that are not fully concretized,
            // i.e. any of the type parameters is not bound to
            // a concrete type.
            if Self::is_generic_struct(tys) {
                return false;
            }
            let ll_name = s_env.ll_struct_name_from_raw_name(tys);
            if self.llvm_cx.named_struct_type(&ll_name).is_none() {
                debug!(target: "structs", "Create struct {}", &ll_name);
                self.llvm_cx.create_opaque_named_struct(&ll_name);
                return true;
            }
            false
        };

        // Now that all the concrete structs are available, pull in the generic ones. Each such
        // StructDefInstantiation will induce a concrete expansion once fields are visited later.
        let cm = m_env.get_verified_module().unwrap();
        for s_def_inst in cm.struct_instantiations() {
            let tys = m_env
                .get_type_actuals(Some(s_def_inst.type_parameters))
                .unwrap_or_default();
            let s_env = m_env.get_struct_by_def_idx(s_def_inst.def);
            if create_opaque_named_struct(&s_env, &tys) {
                all_structs.push((s_env, tys));
            }
        }

        // Similarly, pull in generics from field instantiations.
        for f_inst in cm.field_instantiations() {
            let fld_handle = cm.field_handle_at(f_inst.handle);
            let tys = m_env
                .get_type_actuals(Some(f_inst.type_parameters))
                .unwrap_or_default();
            let s_env = m_env.get_struct_by_def_idx(fld_handle.owner);
            if create_opaque_named_struct(&s_env, &tys) {
                all_structs.push((s_env, tys));
            }
        }

        // Finally, some generic instantiations still may not have been seen. That would be
        // case where no explicit definition was already available, such as passing/returning
        // a generic or constructing a generic. Visit the signature table for any remaining.
        for sig in cm.signatures() {
            for st in &sig.0 {
                let mut inst_signatures: Vec<SignatureToken> = Vec::new();
                SignatureToken::find_struct_instantiation_signatures(st, &mut inst_signatures);
                for sti in &inst_signatures {
                    let gs = m_env.globalize_signature(sti);
                    if let Some(mty::Type::Struct(mid, sid, tys)) = gs {
                        let s_env = g_env.get_module(mid).into_struct(sid);
                        if create_opaque_named_struct(&s_env, &tys) {
                            all_structs.push((s_env, tys));
                        }
                    }
                }
            }
        }

        debug!(target: "structs",
               "Structs after visiting the signature table{}",
               self.dump_all_structs(&all_structs, false),
        );

        // Translate input IR representing Move struct MyMod::MyStruct:
        //   struct MyStruct has { copy, drop, key, store } {
        //       field1: type1, field2: type2, ..., fieldn: typeN
        //   }
        // to a LLVM IR structure type:
        //   %struct.MyMod__MyStruct = type {
        //       <llvm_type1>, <llvm_type2>, ..., <llvm_typeN>
        //   }
        //
        // The target layout is convenient in that the user field offsets [0..N) in the input IR
        // map one-to-one to values used to index into the LLVM struct with getelementptr,
        // extractvalue, and insertvalue.
        for (s_env, tyvec) in &all_structs {
            self.translate_struct(s_env, tyvec);

            // Note: too early to call here `llvm_di_builder.create_struct` since llvm type for struct
            // may be yet not defined, and will be defined in opcode translation.
        }

        debug!(
            target: "structs",
            "Structs after translation{}",
            self.dump_all_structs(&all_structs, true),
        );
    }

    // Translate struct declaration for structs parameterized by
    // nested struct types.
    // TODO: this probbaly doesn't work when other parameterized types
    // are mixed in the nesting of type parameters,
    // e.g. Struct_A<Vector<Struct_B<T>>>, where T is substituted by a
    // concrete type, won't be declared correctly.
    fn translate_struct(&self, s_env: &mm::StructEnv<'mm>, tyvec: &[mty::Type]) {
        let ll_name = s_env.ll_struct_name_from_raw_name(tyvec);
        debug!(target: "structs", "translating struct {}", s_env.struct_raw_type_name(tyvec));
        // Visit each field in this struct, collecting field types.
        let mut ll_field_tys = Vec::with_capacity(s_env.get_field_count() + 1);
        for fld_env in s_env.get_fields() {
            debug!(target: "structs", "translating field {:?}", &fld_env.get_type());
            if let mty::Type::Struct(_m, _s, _tys) = &fld_env.get_type() {
                let new_sty = &fld_env.get_type().instantiate(tyvec);
                if let mty::Type::Struct(m, s, tys) = new_sty {
                    let g_env = &self.env.env;
                    let s_env = g_env.get_module(*m).into_struct(*s);
                    self.translate_struct(&s_env, tys);
                }
            } else if let mty::Type::TypeParameter(x) = &fld_env.get_type() {
                if let mty::Type::Struct(m, s, tys) = &tyvec[*x as usize] {
                    let g_env = &self.env.env;
                    let s_env = g_env.get_module(*m).into_struct(*s);
                    self.translate_struct(&s_env, tys);
                }
            }
            let ll_fld_type = self.to_llvm_type(&fld_env.get_type(), tyvec).unwrap();
            debug!(
                target: "structs",
                "Field now should be concrete type for {ll_name} : {}",
                ll_fld_type.print_to_str()
            );
            ll_field_tys.push(ll_fld_type);
        }
        debug!(target: "structs", "Finished translating fields for {ll_name}");
        if self.llvm_cx.named_struct_type(&ll_name).is_none() {
            debug!(target: "structs", "Create struct {}", &ll_name);
            self.llvm_cx.create_opaque_named_struct(&ll_name);
        }
        let ll_sty = self
            .llvm_cx
            .named_struct_type(&ll_name)
            .expect("no struct type");
        ll_sty.set_struct_body(&ll_field_tys);
    }

    // This method is used to declare structs found when function
    // declrations are generated and new instantiations of generic
    // structs become known.
    // TODO: porbably other parameterized types such as Vector should
    // be handled by this function too.
    fn declare_struct_instance(&self, mty: &mty::Type, tyvec: &[mty::Type]) -> llvm::Type {
        if let mty::Type::Struct(m, s, _tys) = mty {
            let g_env = &self.env.env;
            let s_env = g_env.get_module(*m).into_struct(*s);
            self.translate_struct(&s_env, tyvec);
            self.to_llvm_type(mty, tyvec).unwrap()
        } else {
            unreachable!("Failed to declare a struct {mty:?}")
        }
    }

    fn is_generic_struct(tys: &[mty::Type]) -> bool {
        tys.iter().any(|t| match t {
            mty::Type::Reference(_, ty) => Self::is_generic_struct(&[ty.as_ref().clone()]),
            mty::Type::Struct(_m, _s, tys) => Self::is_generic_struct(tys),
            mty::Type::Tuple(tys) => Self::is_generic_struct(tys),
            mty::Type::TypeParameter(_) => true,
            mty::Type::Vector(ty) => Self::is_generic_struct(&[ty.as_ref().clone()]),
            _ => false,
        })
    }

    fn dump_all_structs(
        &self,
        all_structs: &Vec<(mm::StructEnv, Vec<mty::Type>)>,
        is_post_translation: bool,
    ) -> String {
        let mut s = "\n".to_string();
        for (s_env, tyvec) in all_structs {
            let ll_name = s_env.ll_struct_name_from_raw_name(tyvec);
            let loc = s_env.get_loc();
            let (filename, location) = s_env
                .module_env
                .env
                .get_file_and_location(&loc)
                .unwrap_or(("unknown".to_string(), Location::new(0, 0)));
            let prepost = if is_post_translation {
                "Translated"
            } else {
                "Translating"
            };
            s += &format!(
                "{} struct '{}' => '%{}' {}:{}\n",
                prepost,
                s_env.struct_raw_type_name(tyvec),
                ll_name,
                filename,
                location.line
            )
            .to_string();
            for fld_env in s_env.get_fields() {
                s += &format!(
                    "offset {}: '{}', type ",
                    fld_env.get_offset(),
                    fld_env.get_name().display(s_env.symbol_pool())
                );
                if is_post_translation {
                    if let Some(ll_fld_type) = self.to_llvm_type(&fld_env.get_type(), tyvec) {
                        s += ll_fld_type.print_to_str();
                    } else {
                        s += "<unresolved>";
                    }
                } else {
                    s += format!("{:?}", fld_env.get_type()).as_str();
                };
                s += "\n";
            }
            s += &format!("with abilities: {:?}\n\n", s_env.get_abilities());
        }
        s
    }

    /// Create LLVM function decls for all local functions and
    /// all extern functions that might be called.
    fn declare_functions(&mut self, exports: &mut Vec<String>) {
        let mod_env = self.env.clone(); // fixme bad clone

        // We have previously discovered through experience that some of the model-provided
        // information we once depended on to discover all module functions, called functions,
        // and concrete instantiations are not always consistent or reliable.
        //
        // For this reason, we now take a different approach and seed our discovery with just the
        // list of functions provided by `ModuleEnv::get_functions`. For any other called functions
        // (this module or foreign) and for any generic instantiations, we will expand the seed
        // frontier incrementally by gleaning the remaining information from a visitation of every
        // function call instruction (recursively) in every seed function.
        //
        // While this results in yet another linear walk over all the code, it seems to be the
        // simplest way to work around the model inconsistencies.
        for fn_env in mod_env.get_functions() {
            self.declare_functions_walk(&mod_env, &fn_env, vec![], exports);
        }
    }

    fn declare_functions_walk(
        &mut self,
        mod_env: &mm::ModuleEnv,
        curr_fn_env: &mm::FunctionEnv,
        curr_type_vec: Vec<mty::Type>,
        exports: &mut Vec<String>,
    ) {
        let g_env = &mod_env.env;

        // Do not process a previously declared function/expansion.
        let fn_name = if curr_fn_env.is_native() {
            curr_fn_env.llvm_native_fn_symbol_name()
        } else if curr_fn_env.get_type_parameter_count() == 0 {
            curr_fn_env.llvm_symbol_name(&[])
        } else {
            curr_fn_env.llvm_symbol_name(&curr_type_vec)
        };

        if curr_fn_env.is_inline() {
            // Inline functions are not declared here, but their code is expanded inline by the move compiler.
            // if we declare them here, we will end up with missing compiled module
            debug!("function: {fn_name} is inline - no need to declare");
            return;
        }

        debug!(
            "Checking if {fn_name} exists in current module {:?}",
            mod_env.get_id()
        );
        if self.fn_decls.contains_key(&curr_fn_env.get_full_name_str()) {
            debug!("{fn_name} Exists. Skipping");
            return;
        }

        debug!("Declaring function {fn_name}",);
        let fn_data = StacklessBytecodeGenerator::new(curr_fn_env).generate_function();
        debug!("Generated function {fn_name}",);

        // If the current function is either a native function or a concrete Move function,
        // we have all the information needed to declare a corresponding single function.
        //
        // If the current function is a generic Move function, we will defer declaring its
        // concrete expansions until a call path leading to a particular call site is visited.
        // At that point, the type parameters are either resolved or the function is not used
        // in the module. The generic function itself will not be emitted.
        let curr_fn_qid = curr_fn_env.get_qualified_id();
        if curr_fn_env.is_native() {
            // Declare the native and return early--- there is no function body to visit.
            self.declare_native_function(curr_fn_env, &fn_data, curr_fn_env.llvm_linkage());
            return;
        } else if curr_fn_env.get_type_parameter_count() == 0 {
            let curr_fn_qiid = curr_fn_qid.module_id.qualified_inst(curr_fn_qid.id, vec![]);
            self.declare_move_function(
                curr_fn_env,
                &[],
                &fn_data,
                curr_fn_env.llvm_linkage(),
                exports,
            );
            if curr_fn_qid.module_id != mod_env.get_id() {
                // True foreign functions are only declared in our module, don't process further.
                return;
            }
            self.expanded_functions.push(curr_fn_qiid);
        } else {
            // Determine whether any of the type parameters for this generic function are still
            // unresolved. If so, then function is not a concrete instance and we defer it until
            // a call path containing it is expanded.
            assert!(curr_fn_env.get_type_parameter_count() > 0);
            let inst_is_generic = curr_type_vec.iter().any(|t| t.is_open());
            if curr_type_vec.is_empty() || inst_is_generic {
                return;
            }

            // Note that we may be declaring a foreign function here. But since it is being
            // expanded into our current module, its linkage is effectively private.
            let curr_fn_qiid = curr_fn_qid
                .module_id
                .qualified_inst(curr_fn_qid.id, curr_type_vec.clone());
            self.declare_move_function(
                curr_fn_env,
                &curr_type_vec,
                &fn_data,
                llvm::LLVMLinkage::LLVMPrivateLinkage,
                exports,
            );
            self.expanded_functions.push(curr_fn_qiid);
        }

        // Visit every call site in the current function, instantiate their type parameters,
        // and then recursively grow the frontier.
        for instr in &fn_data.code {
            if let sbc::Bytecode::Call(
                _,
                _,
                sbc::Operation::Function(mod_id, fun_id, types),
                _,
                None,
            ) = instr
            {
                // Instantiate any type parameters at the current call site with the
                // enclosing type parameter scope `curr_type_vec`.
                let types = mty::Type::instantiate_vec(types.to_vec(), &curr_type_vec);

                // Recursively discover/declare more functions on this call path.
                let called_fn_env = g_env.get_function((*mod_id).qualified(*fun_id));
                self.declare_functions_walk(mod_env, &called_fn_env, types, exports);
            }
        }
    }

    fn declare_move_function(
        &mut self,
        fn_env: &mm::FunctionEnv,
        tyvec: &[mty::Type],
        fn_data: &FunctionData,
        linkage: llvm::LLVMLinkage,
        exports: &mut Vec<String>,
    ) {
        let mut linkage = linkage;
        let ll_sym_name = fn_env.llvm_symbol_name(tyvec);
        debug!(
            "Declare Move function {ll_sym_name} in {}",
            fn_env.get_full_name_str()
        );
        let ll_fn = {
            let ll_fnty = {
                let ll_rty = if let Some(ty) = self.to_llvm_type(&fn_data.result_type, tyvec) {
                    ty
                } else {
                    self.declare_struct_instance(&fn_data.result_type, tyvec)
                };

                let ll_parm_tys = fn_env
                    .get_parameter_types()
                    .iter()
                    .map(|mty| {
                        if let Some(ty) = self.to_llvm_type(mty, tyvec) {
                            ty
                        } else {
                            self.declare_struct_instance(mty, tyvec)
                        }
                    })
                    .collect::<Vec<_>>();

                llvm::FunctionType::new(ll_rty, &ll_parm_tys)
            };

            // For Move functions we can infer directly from parameters that:
            // - `&` and `&mut` will be `nonnull` pointers in the generated LLVM IR.
            // - '&' is `readonly` (shared, read only).
            // - '&mut' is `noalias` (exclusive, writeable).
            // There are other attributes we may infer in the future with more analysis.
            let mut attrs = Vec::new();
            for (i, pt) in fn_env.get_parameter_types().iter().enumerate() {
                let parm_num = (i + 1) as u32;
                if pt.is_reference() {
                    attrs.push((parm_num, "nonnull", None));
                }
                if pt.is_immutable_reference() {
                    attrs.push((parm_num, "readonly", None));
                } else if pt.is_mutable_reference() {
                    attrs.push((parm_num, "noalias", None));
                }
            }
            let unit_test = self.options.unit_test_function.clone().unwrap_or_default();
            if fn_env.is_entry() || fn_env.get_full_name_str().replace("::", "__") == unit_test {
                linkage = llvm::LLVMLinkage::LLVMExternalLinkage;
            }
            let tfn = self.llvm_module.add_function(
                exports,
                &fn_env.module_env.llvm_module_name(),
                &ll_sym_name,
                ll_fnty,
                fn_env.is_entry(),
            );
            self.llvm_module.add_attributes(tfn, &attrs);
            tfn
        };

        ll_fn.as_gv().set_linkage(linkage);
        debug!("Adding declared {ll_sym_name} to current module");
        self.fn_decls.insert(fn_env.get_full_name_str(), ll_fn);
        self.fn_is_entry
            .insert(fn_env.get_full_name_str(), fn_env.is_entry());
    }

    /// Generate the call selector function.
    /// pallet-revive calls 2 functions: `deploy` and `call`.
    /// The `call` is implemented in rust to get the input from the
    /// host and then call `call_selector` to select which actual function to call.
    /// The input to the contract contains a keccak hash of the function name (first 4 bytes of the keccak hash),
    /// and `call_selector` will select the function to call based on that hash.
    /// This method will loop over all declared functions check if the keccak hash of the function name
    /// matches the input hash, and if so, it will call the function. If no match is found, the
    /// function should abort.
    fn generate_call_selector(&mut self, exports: &mut Vec<String>) {
        let llvm_cx = self.llvm_cx;
        let llvm_module = self.llvm_module;
        if exports.contains(&"call_selector".to_string()) {
            debug!("call_selector already declared, skipping");
            return;
        }
        let i64_t = llvm_cx.int_type(64);
        let i32_t = llvm_cx.int_type(32);
        let i8_p = llvm_cx.ptr_type();
        let ret_ty = llvm_cx.void_type();

        let param_tys = [i8_p, i64_t];
        let llty = llvm::FunctionType::new(ret_ty, &param_tys);
        let ll_fn = llvm_module.add_function(&mut vec![], "native", "call_selector", llty, false);
        let attrs = vec![(1, "readonly", None), (1, "nonnull", None)];
        llvm_module.add_attributes(ll_fn, &attrs);
        let builder = llvm_cx.create_builder();
        let entry_bb = ll_fn.append_basic_block("entry");
        builder.position_at_end(entry_bb);
        let buf_ptr = ll_fn.get_param(0);

        // cast `i8*` → `i32*` so we can load a 4‐byte selector
        let sel_ptr = builder.build_unary_bitcast(buf_ptr.as_any_value(), i8_p, "sel_ptr");
        let raw_sel = builder.load(sel_ptr, i32_t, "raw_sel");
        let sel64 = builder.build_zext(raw_sel, i64_t, "sel64");

        // build the switch
        let default_bb = ll_fn.append_basic_block("default");
        let switch_inst = builder.build_switch(sel64, default_bb, self.fn_decls.len() as u32);
        for (name, func) in self.fn_decls.iter() {
            if !self.fn_is_entry.get(name).unwrap_or(&false) {
                debug!("Skipping function {name} as it is not an entry function");
                continue;
            }
            debug!("Adding call selector function {name} to exports");
            let mut keccak = Keccak::v256();
            keccak.update(name.as_bytes());
            let mut hash = [0u8; 32];
            keccak.finalize(&mut hash);
            let sel = u32::from_be_bytes([hash[3], hash[2], hash[1], hash[0]]);
            debug!("Adding call selector function {name} with selector {sel:x?} to exports");

            // create a basic block for this case
            let bb_name = format!("case_{name}");
            let case_bb = ll_fn.append_basic_block(&bb_name);
            debug!("Adding case for function {name} with selector {sel:x?} to call selector");
            switch_inst.add_case(llvm::Constant::const_int(i64_t, sel as u64, 0), case_bb);
            debug!("Added case for function {name} with selector {sel:x?} to call selector");

            builder.position_at_end(case_bb);

            let four = llvm::Constant::const_int(i64_t, 4, 0);
            let signer_ptr = builder.build_address_with_indices(
                llvm_cx.int_type(8),
                buf_ptr.as_any_value(),
                &[four.as_any_value()],
                "signer",
            );
            let args = &[signer_ptr];
            builder.call(*func, args);
            debug!("built call");
            builder.build_return_void();
            debug!("built return");
        }
        debug!("Added all cases to call selector");

        // create basic block for the default case which will call abort, this triggers terminate
        // on pallet-revive
        builder.position_at_end(default_bb);
        let abort_args = &[llvm::Constant::const_int(i64_t, 2, 0).as_any_value()];
        let abort_fn =
            Self::get_runtime_function_by_name(llvm_cx, llvm_module, &self.rtty_cx, "abort");
        builder.call(abort_fn, abort_args);
        builder.build_unreachable();
        exports.push("call_selector".to_string());
    }

    /// Declare native functions.
    ///
    /// Native functions are unlike Move functions in that they
    /// pass type descriptors for generics, and they follow
    /// the C ABI.
    ///
    /// Tweaks to the calling conventions here must be mirrored
    /// in `translate_native_fun_call.
    ///
    /// At some point we might want to factor out the platform-specific ABI
    /// decisions, but for now there are only a few ABI concerns, and we may
    /// never support another platform for which the ABI is different.
    fn declare_native_function(
        &mut self,
        fn_env: &mm::FunctionEnv,
        fn_data: &FunctionData,
        linkage: llvm::LLVMLinkage,
    ) {
        debug!("Declare native function {}", fn_env.get_full_name_str());
        assert!(fn_env.is_native());

        let llcx = &self.llvm_cx;
        let ll_native_sym_name = fn_env.llvm_native_fn_symbol_name();
        let ll_fn = {
            let ll_fnty = {
                // Generic return values are passed through a final return pointer arg.
                let mty0 = &&fn_data.result_type;
                let (ll_rty, ll_byref_rty) = if mty0.is_type_parameter() {
                    (llcx.void_type(), Some(llcx.ptr_type()))
                } else {
                    (self.to_llvm_type(mty0, &[]).unwrap(), None)
                };

                // Native functions take type parameters as the
                // first arguments.
                let num_typarams = fn_env.get_type_parameter_count();
                let ll_tydesc_parms = std::iter::repeat_n(llcx.ptr_type(), num_typarams);

                let ll_parm_tys = fn_env.get_parameter_types();
                let ll_parm_tys = ll_parm_tys.iter().map(|mty| {
                    // Pass type parameters and vectors as pointers
                    if mty.is_type_parameter() || mty.is_vector() {
                        llcx.ptr_type()
                    } else {
                        self.to_llvm_type(mty, &[]).unwrap()
                    }
                });

                let all_ll_parms = ll_tydesc_parms
                    .chain(ll_parm_tys)
                    .chain(ll_byref_rty)
                    .collect::<Vec<_>>();

                llvm::FunctionType::new(ll_rty, &all_ll_parms)
            };
            // native functions are functions imported by guest program and exported by polkavm
            // we don't need to export polka sections for those
            self.llvm_module.add_function(
                &mut vec![],
                "native",
                &ll_native_sym_name,
                ll_fnty,
                false,
            )
        };

        ll_fn.as_gv().set_linkage(linkage);

        self.fn_decls.insert(fn_env.get_full_name_str(), ll_fn);
    }

    pub fn lookup_move_fn_decl(&self, qiid: mm::QualifiedInstId<mm::FunId>) -> llvm::Function {
        let fn_env = self
            .env
            .env
            .get_module(qiid.module_id)
            .into_function(qiid.id);
        debug!(
            "Looking up move fn decl: {} in module {}",
            fn_env.get_full_name_str(),
            fn_env.module_env.get_full_name_str()
        );
        let sname = fn_env.get_full_name_str();
        debug!("Looking up move fn decl: {sname}");
        let decl = self.fn_decls.get(&sname);
        assert!(decl.is_some(), "move fn decl not found: {sname}");
        *decl.unwrap()
    }

    pub fn lookup_native_fn_decl(&self, qid: mm::QualifiedId<mm::FunId>) -> llvm::Function {
        let fn_env = self.env.env.get_module(qid.module_id).into_function(qid.id);
        let sname = fn_env.get_full_name_str();
        let decl = self.fn_decls.get(&sname);
        assert!(decl.is_some(), "native fn decl not found: {sname}");
        *decl.unwrap()
    }

    pub fn to_llvm_type(&self, mty: &mty::Type, tyvec: &[mty::Type]) -> Option<llvm::Type> {
        use mty::{PrimitiveType, Type};

        match mty {
            Type::Primitive(PrimitiveType::Bool)
            | Type::Primitive(PrimitiveType::U8)
            | Type::Primitive(PrimitiveType::U16)
            | Type::Primitive(PrimitiveType::U32)
            | Type::Primitive(PrimitiveType::U64)
            | Type::Primitive(PrimitiveType::U128)
            | Type::Primitive(PrimitiveType::U256) => {
                Some(self.llvm_cx.int_type(mty.get_bitwidth() as usize))
            }
            Type::Primitive(PrimitiveType::Address) => {
                Some(self.rtty_cx.get_llvm_type_for_address())
            }
            Type::Primitive(PrimitiveType::Signer) => Some(self.rtty_cx.get_llvm_type_for_signer()),
            Type::Primitive(PrimitiveType::Num)
            | Type::Primitive(PrimitiveType::Range)
            | Type::Primitive(PrimitiveType::EventStore) => {
                panic!("{mty:?} only appears in specifications.")
            }
            Type::Reference(_, _) => Some(self.llvm_cx.ptr_type()),
            Type::TypeParameter(tp_idx) => {
                if (*tp_idx as usize) < tyvec.len() {
                    self.to_llvm_type(&tyvec[*tp_idx as usize], &[])
                } else {
                    debug!("type parameter index is out of range {tp_idx}");
                    None
                }
            }
            Type::Struct(_mid, _sid, _tys) => {
                // First substitute any generic type parameters occuring in _tys.
                let new_sty = mty.instantiate(tyvec);

                debug!(
                    target: "structs",
                    "Instantiated struct {}",
                    new_sty
                        .get_struct(self.env.env)
                        .unwrap()
                        .0
                        .struct_raw_type_name(tyvec)
                );
                // Then process the (possibly type-substituted) struct.
                if let Type::Struct(declaring_module_id, struct_id, tys) = new_sty {
                    let global_env = &self.env.env;
                    let struct_env = global_env
                        .get_module(declaring_module_id)
                        .into_struct(struct_id);
                    let struct_name = struct_env.ll_struct_name_from_raw_name(&tys);
                    if let Some(stype) = self.llvm_cx.named_struct_type(&struct_name) {
                        Some(stype.as_any_type())
                    } else {
                        debug!(target: "structs", "struct type for '{}' not found", &struct_name);
                        None
                    }
                } else {
                    unreachable!("")
                }
            }
            Type::Vector(_) => Some(self.rtty_cx.get_llvm_type_for_move_vector(self, tyvec)),
            Type::Tuple(types_vec) => {
                if types_vec.is_empty() {
                    Some(self.llvm_cx.void_type())
                } else {
                    let llvm_types = types_vec
                        .iter()
                        .map(|move_type| {
                            self.to_llvm_type(move_type, &[])
                                .unwrap_or_else(|| panic!("{move_type:?} should be available"))
                        })
                        .collect::<Vec<_>>();
                    Some(
                        self.llvm_cx
                            .anonymous_struct_type(&llvm_types)
                            .as_any_type(),
                    )
                }
            }
            Type::Fun(_, _, _)
            | Type::TypeDomain(_)
            | Type::ResourceDomain(_, _, _)
            | Type::Error
            | Type::Var(_) => {
                panic!("unexpected field type {mty:?}")
            }
        }
    }

    fn create_fn_context<'this>(
        &'this self,
        fn_env: mm::FunctionEnv<'mm>,
        module_cx: &'mm ModuleContext,
        type_params: &'mm [mty::Type],
    ) -> FunctionContext<'mm, 'this> {
        let locals = Vec::with_capacity(fn_env.get_local_count().unwrap_or(0));
        FunctionContext {
            env: fn_env,
            module_cx,
            label_blocks: BTreeMap::new(),
            locals,
            type_params,
        }
    }

    pub fn get_rttydesc_ptrs(&self, types: &[mty::Type]) -> Vec<llvm::Constant> {
        let mut ll_global_ptrs = vec![];
        for type_ in types {
            let ll_tydesc = self.rtty_cx.define_llvm_tydesc(type_);
            ll_global_ptrs.push(ll_tydesc.ptr());
        }
        ll_global_ptrs
    }

    // This version is used in contexts where TempIndexes are not used and/or where the caller
    // expects a return value that it will decide how to use or store.
    pub fn emit_rtcall_with_retval(&self, rtcall: RtCall) -> llvm::AnyValue {
        match &rtcall {
            RtCall::VecCopy(ll_dst_value, ll_src_value, elt_mty) => {
                // Note, no retval from vec_copy.
                let llfn = Self::get_runtime_function(
                    self.llvm_cx,
                    self.llvm_module,
                    &self.rtty_cx,
                    &rtcall,
                );
                let mut typarams: Vec<_> = self
                    .get_rttydesc_ptrs(std::slice::from_ref(elt_mty))
                    .iter()
                    .map(|llval| llval.as_any_value())
                    .collect();
                typarams.push(*ll_dst_value);
                typarams.push(*ll_src_value);
                self.llvm_builder.call(llfn, &typarams)
            }
            RtCall::VecCmpEq(ll_dst_value, ll_src_value, elt_mty) => {
                let llfn = Self::get_runtime_function(
                    self.llvm_cx,
                    self.llvm_module,
                    &self.rtty_cx,
                    &rtcall,
                );
                let mut typarams: Vec<_> = self
                    .get_rttydesc_ptrs(std::slice::from_ref(elt_mty))
                    .iter()
                    .map(|llval| llval.as_any_value())
                    .collect();
                typarams.push(*ll_dst_value);
                typarams.push(*ll_src_value);
                self.llvm_builder.call(llfn, &typarams)
            }
            RtCall::VecEmpty(elt_mty) => {
                let llfn = Self::get_runtime_function(
                    self.llvm_cx,
                    self.llvm_module,
                    &self.rtty_cx,
                    &rtcall,
                );
                let typarams: Vec<_> = self
                    .get_rttydesc_ptrs(std::slice::from_ref(elt_mty))
                    .iter()
                    .map(|llval| llval.as_any_value())
                    .collect();
                self.llvm_builder.call(llfn, &typarams)
            }
            RtCall::StrCmpEq(str1_ptr, str1_len, str2_ptr, str2_len) => {
                let llfn = Self::get_runtime_function(
                    self.llvm_cx,
                    self.llvm_module,
                    &self.rtty_cx,
                    &rtcall,
                );
                let params = vec![*str1_ptr, *str1_len, *str2_ptr, *str2_len];
                self.llvm_builder.call(llfn, &params)
            }
            RtCall::StructCmpEq(ll_src1_value, ll_src2_value, s_mty) => {
                let llfn = Self::get_runtime_function(
                    self.llvm_cx,
                    self.llvm_module,
                    &self.rtty_cx,
                    &rtcall,
                );
                let mut typarams: Vec<_> = self
                    .get_rttydesc_ptrs(std::slice::from_ref(s_mty))
                    .iter()
                    .map(|llval| llval.as_any_value())
                    .collect();
                typarams.push(*ll_src1_value);
                typarams.push(*ll_src2_value);
                self.llvm_builder.call(llfn, &typarams)
            }
            _ => unreachable!(),
        }
    }

    // TODO: consider better refactoring for this and other
    // class-level methods, which used to be instance methods.
    // These methods were converted to class-level because their code
    // is resued by EntrypointGeenrator, which operates outside any
    // ModuleContext, yet needs to add declarations of functions
    // defined in other modules.
    pub fn emit_rtcall_abort_raw(
        llvm_cx: &'up llvm::Context,
        llvm_builder: &llvm::Builder,
        llvm_module: &'up llvm::Module,
        rtty_cx: &RttyContext,
        val: u64,
    ) {
        let thefn = Self::get_runtime_function_by_name(llvm_cx, llvm_module, rtty_cx, "abort");
        debug!(target: "runtime", "emit_rtcall_abort_raw({val}): {thefn:?}");
        let param_ty = llvm_cx.int_type(64);
        let const_llval = llvm::Constant::int(param_ty, U256::from(val));
        llvm_builder.build_call_imm(thefn, &[const_llval]);
        llvm_builder.build_unreachable();
    }

    pub fn get_runtime_function(
        llvm_cx: &'up llvm::Context,
        llvm_module: &'up llvm::Module,
        rtty_cx: &RttyContext,
        rtcall: &RtCall,
    ) -> llvm::Function {
        let name = match rtcall {
            RtCall::Abort(..) => "abort",
            RtCall::Deserialize(..) => "deserialize",
            RtCall::VecDestroy(..) => "vec_destroy",
            RtCall::VecCopy(..) => "vec_copy",
            RtCall::VecCmpEq(..) => "vec_cmp_eq",
            RtCall::VecEmpty(..) => "vec_empty",
            RtCall::StrCmpEq(..) => "str_cmp_eq",
            RtCall::StructCmpEq(..) => "struct_cmp_eq",
            RtCall::MoveTo(..) => "move_to",
            RtCall::MoveFrom(..) => "move_from",
            RtCall::BorrowGlobal(..) => "borrow_global",
            RtCall::Exists(..) => "exists",
            RtCall::Release(..) => "release",
        };
        Self::get_runtime_function_by_name(llvm_cx, llvm_module, rtty_cx, name)
    }

    fn get_runtime_function_by_name(
        llvm_cx: &'up llvm::Context,
        llvm_module: &'up llvm::Module,
        rtty_cx: &RttyContext,
        rtcall_name: &str,
    ) -> llvm::Function {
        debug!(target: "runtime", "get_runtime_function_by_name({rtcall_name})");
        let fn_name = format!("move_rt_{rtcall_name}");
        let llfn = llvm_module.get_named_function(&fn_name);
        if let Some(llfn) = llfn {
            debug!(target: "runtime", "Found existing runtime function {fn_name}");
            llfn
        } else {
            let (llty, attrs) = match rtcall_name {
                "abort" => {
                    debug!(target: "runtime", "Declaring abort function {fn_name}");
                    let ret_ty = llvm_cx.void_type();
                    let param_tys = &[llvm_cx.int_type(64)];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let attrs = vec![
                        (llvm::LLVMAttributeFunctionIndex, "noreturn", None),
                        (llvm::LLVMAttributeFunctionIndex, "cold", None),
                    ];
                    (llty, attrs)
                }
                "deserialize" => {
                    let ret_ty = llvm_cx.void_type();
                    let ptr_ty = llvm_cx.ptr_type();
                    let int_ty = llvm_cx.int_type(64);
                    let param_tys = &[ptr_ty, ptr_ty];
                    let ll_sret = llvm_cx.get_anonymous_struct_type(&[
                        llvm_cx.get_anonymous_struct_type(&[ptr_ty, int_ty]),
                        ptr_ty,
                        llvm_cx.get_anonymous_struct_type(&[ptr_ty, int_ty, int_ty]),
                    ]);
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let ll_fn =
                        llvm_module.add_function(&mut vec![], "native", &fn_name, llty, false);
                    llvm_module.add_type_attribute(ll_fn, 1, "sret", ll_sret);
                    return ll_fn;
                }
                "vec_destroy" => {
                    // vec_destroy(type_ve: &MoveType, v: MoveUntypedVector)
                    let ret_ty = llvm_cx.void_type();
                    let tydesc_ty = llvm_cx.ptr_type();
                    // The vector is passed by value, but the C ABI here passes structs by reference,
                    // so it's another pointer.
                    let vector_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty, vector_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let attrs = Self::mk_pattrs_for_move_type(1);
                    (llty, attrs)
                }
                "vec_copy" => {
                    // vec_copy(type_ve: &MoveType, dstv: &mut MoveUntypedVector, srcv: &MoveUntypedVector)
                    let ret_ty = llvm_cx.void_type();
                    let tydesc_ty = llvm_cx.ptr_type();
                    // The vectors are passed by value, but the C ABI here passes structs by reference,
                    // so it's another pointer.
                    let vector_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty, vector_ty, vector_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.extend(Self::mk_pattrs_for_move_untyped_vec(2, true /* mut */));
                    attrs.extend(Self::mk_pattrs_for_move_untyped_vec(
                        3, false, /* !mut */
                    ));
                    (llty, attrs)
                }
                "vec_cmp_eq" => {
                    // vec_cmp_eq(type_ve: &MoveType, v1: &MoveUntypedVector, v2: &MoveUntypedVector) -> bool
                    let ret_ty = llvm_cx.int_type(1);
                    let tydesc_ty = llvm_cx.ptr_type();
                    // The vectors are passed by value, but the C ABI here passes structs by reference,
                    // so it's another pointer.
                    let vector_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty, vector_ty, vector_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.extend(Self::mk_pattrs_for_move_untyped_vec(
                        2, false, /* !mut */
                    ));
                    attrs.extend(Self::mk_pattrs_for_move_untyped_vec(
                        3, false, /* !mut */
                    ));
                    (llty, attrs)
                }
                "vec_empty" => {
                    // vec_empty(type_ve: &MoveType) -> MoveUntypedVector
                    let ret_ty = rtty_cx.get_llvm_type_for_move_native_vector();
                    let tydesc_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let attrs = Self::mk_pattrs_for_move_type(1);
                    (llty, attrs)
                }
                "str_cmp_eq" => {
                    // str_cmp_eq(str1_ptr: &AnyValue, str1_len: &AnyValue,
                    //            str2_ptr: &AnyValue, str1_len: &AnyValue) -> bool
                    let ret_ty = llvm_cx.int_type(1);
                    let ptr_ty = llvm_cx.ptr_type();
                    let len_ty = llvm_cx.int_type(64);
                    let param_tys = &[ptr_ty, len_ty, ptr_ty, len_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let attrs = vec![
                        (1, "readonly", None),
                        (1, "nonnull", None),
                        (3, "readonly", None),
                        (3, "nonnull", None),
                    ];
                    (llty, attrs)
                }
                "struct_cmp_eq" => {
                    // struct_cmp_eq(type_ve: &MoveType, s1: &AnyValue, s2: &AnyValue) -> bool;
                    let ret_ty = llvm_cx.int_type(1);
                    let tydesc_ty = llvm_cx.ptr_type();
                    let anyval_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty, anyval_ty, anyval_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.push((2, "readonly", None));
                    attrs.push((2, "nonnull", None));
                    attrs.push((3, "readonly", None));
                    attrs.push((3, "nonnull", None));
                    (llty, attrs)
                }
                "move_to" => {
                    debug!(target: "runtime", "Declaring move_to function {fn_name}");
                    // move_to(address: &AnyValue, r: &AnyValue, type: &MoveType, type_tag) -> bool;
                    let ret_ty = llvm_cx.void_type();
                    let tydesc_ty = llvm_cx.ptr_type();
                    let anyval_ty = llvm_cx.ptr_type();
                    let tag_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty, anyval_ty, anyval_ty, tag_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.push((2, "readonly", None));
                    attrs.push((2, "nonnull", None));
                    attrs.push((3, "readonly", None));
                    attrs.push((3, "nonnull", None));
                    attrs.push((4, "readonly", None));
                    attrs.push((4, "nonnull", None));
                    attrs.push((4, "dereferenceable", Some(32u64)));
                    (llty, attrs)
                }
                "move_from" => {
                    debug!(target: "runtime", "Declaring move_from function {fn_name}");
                    // move_from(address: &AnyValue, type: &MoveType, retval, type_tag) -> T;
                    let ret_ty = llvm_cx.void_type();
                    let tydesc_ty = llvm_cx.ptr_type();
                    let anyval_ty = llvm_cx.ptr_type();
                    let retval_ty = llvm_cx.ptr_type();
                    let tag_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty, anyval_ty, retval_ty, tag_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.push((2, "readonly", None));
                    attrs.push((2, "nonnull", None));
                    attrs.push((3, "nonnull", None));
                    attrs.push((4, "readonly", None));
                    attrs.push((4, "nonnull", None));
                    attrs.push((4, "dereferenceable", Some(32u64)));
                    (llty, attrs)
                }
                "borrow_global" => {
                    debug!(target: "runtime", "Declaring borrow_global function {fn_name}");
                    // borrow_global(address: &AnyValue, type: &MoveType, retval, type_tag) -> &T;
                    let ret_ty = llvm_cx.void_type();
                    let tydesc_ty = llvm_cx.ptr_type();
                    let anyval_ty = llvm_cx.ptr_type();
                    let retval_ty = llvm_cx.ptr_type();
                    let tag_ty = llvm_cx.ptr_type();
                    let mut_ty = llvm_cx.int_type(1);
                    let param_tys = &[anyval_ty, tydesc_ty, retval_ty, tag_ty, mut_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.push((2, "readonly", None));
                    attrs.push((2, "nonnull", None));
                    attrs.push((3, "readonly", None));
                    attrs.push((3, "nonnull", None));
                    attrs.push((4, "readonly", None));
                    attrs.push((4, "nonnull", None));
                    attrs.push((4, "dereferenceable", Some(32u64)));
                    (llty, attrs)
                }
                "exists" => {
                    debug!(target: "runtime", "Declaring exists function {fn_name}");
                    // exists(address: &AnyValue, type: &MoveType, type_tag) -> bool;
                    let ret_ty = llvm_cx.int_type(1);
                    let tydesc_ty = llvm_cx.ptr_type();
                    let anyval_ty = llvm_cx.ptr_type();
                    let tag_ty = llvm_cx.ptr_type();
                    let param_tys = &[anyval_ty, tydesc_ty, tag_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.push((2, "readonly", None));
                    attrs.push((2, "nonnull", None));
                    attrs.push((3, "readonly", None));
                    attrs.push((3, "nonnull", None));
                    attrs.push((3, "dereferenceable", Some(32u64)));
                    (llty, attrs)
                }
                "release" => {
                    debug!(target: "runtime", "Declaring release function {fn_name}");
                    // release(address: &AnyValue, r: &AnyValue, type: &MoveType, type_tag);
                    let ret_ty = llvm_cx.void_type();
                    let tydesc_ty = llvm_cx.ptr_type();
                    let anyval_ty = llvm_cx.ptr_type();
                    let tag_ty = llvm_cx.ptr_type();
                    let param_tys = &[tydesc_ty, anyval_ty, anyval_ty, tag_ty];
                    let llty = llvm::FunctionType::new(ret_ty, param_tys);
                    let mut attrs = Self::mk_pattrs_for_move_type(1);
                    attrs.push((2, "readonly", None));
                    attrs.push((2, "nonnull", None));
                    attrs.push((3, "readonly", None));
                    attrs.push((3, "nonnull", None));
                    attrs.push((4, "readonly", None));
                    attrs.push((4, "nonnull", None));
                    attrs.push((4, "dereferenceable", Some(32u64)));
                    (llty, attrs)
                }
                n => panic!("unknown runtime function {n}"),
            };

            let ll_fn = llvm_module.add_function(&mut vec![], "native", &fn_name, llty, false);
            llvm_module.add_attributes(ll_fn, &attrs);
            ll_fn
        }
    }

    fn mk_pattrs_for_move_type(
        attr_idx: llvm::LLVMAttributeIndex,
    ) -> Vec<(llvm::LLVMAttributeIndex, &'static str, Option<u64>)> {
        assert!(
            attr_idx != llvm::LLVMAttributeReturnIndex
                && attr_idx != llvm::LLVMAttributeFunctionIndex
        );
        vec![
            (attr_idx, "readonly", None),
            (attr_idx, "nonnull", None),
            (attr_idx, "dereferenceable", Some(MOVE_TYPE_DESC_SIZE)),
        ]
    }

    fn mk_pattrs_for_move_untyped_vec(
        attr_idx: llvm::LLVMAttributeIndex,
        mutable: bool,
    ) -> Vec<(llvm::LLVMAttributeIndex, &'static str, Option<u64>)> {
        assert!(
            attr_idx != llvm::LLVMAttributeReturnIndex
                && attr_idx != llvm::LLVMAttributeFunctionIndex
        );
        let mut attrs = vec![
            (attr_idx, "nonnull", None),
            (
                attr_idx,
                "dereferenceable",
                Some(MOVE_UNTYPED_VEC_DESC_SIZE),
            ),
        ];
        if !mutable {
            attrs.push((attr_idx, "readonly", None));
        }
        attrs
    }
}
