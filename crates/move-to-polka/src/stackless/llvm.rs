// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

//! LLVM wrappers.
//!
//! The stackless code generator accesses llvm only through this mod.
//!
//! It:
//!
//! - Runs dtors
//! - Encapsulates unsafety, though making LLVM fully memsafe is hard.
//! - Hides weirdly mutable array pointers.
//! - Provides high-level instruction builders compatible with the stackless bytecode model.

use libc::abort;
use llvm_sys::{core::*, prelude::*, target::*, target_machine::*, LLVMOpcode, LLVMUnnamedAddr};
use log::{debug, trace, warn};
use move_core_types::u256;
use num_traits::{PrimInt, ToPrimitive};

use crate::cstr::SafeCStr;

use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    hash::DefaultHasher,
    ptr,
    rc::Rc,
};

pub use llvm_sys::{
    debuginfo::{
        LLVMCreateDIBuilder, LLVMDIBuilderCreateFile, LLVMDITypeGetName, LLVMDisposeDIBuilder,
    },
    LLVMAttributeFunctionIndex, LLVMAttributeIndex, LLVMAttributeReturnIndex, LLVMIntPredicate,
    LLVMLinkage,
    LLVMLinkage::LLVMInternalLinkage,
    LLVMTypeKind::LLVMIntegerTypeKind,
    LLVMValue,
};

use crate::stackless::{
    dwarf::{from_raw_slice_to_string, DIBuilder},
    GlobalContext, ModuleContext,
};

pub fn initialize_riscv() {
    unsafe {
        LLVMInitializeRISCVTargetInfo();
        LLVMInitializeRISCVTarget();
        LLVMInitializeRISCVTargetMC();
        LLVMInitializeRISCVAsmPrinter();
        LLVMInitializeRISCVAsmParser();
    }
}

// Return a unique id given the name of an enum attribute, or None if no attribute by
// that name exists. See the LLVM LangRef for attribute names.
pub fn get_attr_kind_for_name(attr_name: &str) -> Option<usize> {
    unsafe {
        let uint_kind = LLVMGetEnumAttributeKindForName(attr_name.cstr(), attr_name.len());
        if uint_kind == 0 {
            None
        } else {
            Some(uint_kind as usize)
        }
    }
}

fn get_name(value: *mut LLVMValue) -> String {
    let mut length: ::libc::size_t = 0;
    let name_ptr = unsafe { LLVMGetValueName2(value, &mut length) };
    let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
    name_cstr.to_string_lossy().into_owned()
}

// Reserved for future usage
fn _set_name(value: LLVMValueRef, name: &str) {
    let cstr = std::ffi::CString::new(name).expect("Failed to create CString");
    unsafe {
        LLVMSetValueName2(value, cstr.as_ptr(), cstr.as_bytes().len());
    }
}

#[derive(Debug)]
pub struct Context(pub LLVMContextRef);

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.0);
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Context {
        unsafe { Context(LLVMContextCreate()) }
    }

    pub fn create_module(&self, name: &str) -> Module {
        unsafe {
            Module(
                LLVMModuleCreateWithNameInContext(name.cstr(), self.0),
                Rc::new(RefCell::new(String::with_capacity(100))),
                name.to_owned(),
            )
        }
    }

    pub fn create_builder(&self) -> Builder {
        unsafe { Builder(LLVMCreateBuilderInContext(self.0)) }
    }

    pub fn create_di_builder<'up>(
        &'up self,
        g_ctx: &'up GlobalContext,
        module: &Module,
        source: &str,
        debug: bool,
    ) -> DIBuilder<'up> {
        DIBuilder::new(g_ctx, module, source, debug)
    }

    pub fn get_anonymous_struct_type(&self, field_tys: &[Type]) -> Type {
        unsafe {
            let mut field_tys: Vec<_> = field_tys.iter().map(|f| f.0).collect();
            Type(LLVMStructTypeInContext(
                self.0,
                field_tys.as_mut_ptr(),
                field_tys.len() as u32,
                0, /* !packed */
            ))
        }
    }

    pub fn void_type(&self) -> Type {
        unsafe { Type(LLVMVoidTypeInContext(self.0)) }
    }

    pub fn int_type(&self, len: usize) -> Type {
        unsafe { Type(LLVMIntTypeInContext(self.0, len as libc::c_uint)) }
    }

    pub fn ptr_type(&self) -> Type {
        unsafe { Type(LLVMPointerTypeInContext(self.0, 0)) }
    }

    pub fn array_type(&self, ll_elt_ty: Type, len: usize) -> Type {
        unsafe { Type(LLVMArrayType2(ll_elt_ty.0, len as u64)) }
    }

    pub fn vector_type(&self, ll_elt_ty: Type, len: usize) -> Type {
        let info = ll_elt_ty.print_to_str();
        debug!(target: "vector", "vector_type {info}");
        unsafe { Type(LLVMVectorType(ll_elt_ty.0, len as libc::c_uint)) }
    }

    fn llvm_type_from_rust_int_type<T: 'static>(&self) -> Type {
        match std::any::type_name::<T>() {
            "u8" => self.int_type(8),
            "u16" => self.int_type(16),
            "u32" => self.int_type(32),
            "u64" => self.int_type(64),
            "u128" => self.int_type(128),
            _ => todo!("{}", std::any::type_name::<T>()),
        }
    }

    pub fn named_struct_type(&self, name: &str) -> Option<StructType> {
        unsafe {
            let tyref = LLVMGetTypeByName2(self.0, name.cstr());
            if tyref.is_null() {
                None
            } else {
                Some(StructType(tyref))
            }
        }
    }

    pub fn anonymous_struct_type(&self, field_tys: &[Type]) -> StructType {
        unsafe {
            let mut field_tys: Vec<_> = field_tys.iter().map(|f| f.0).collect();
            StructType(LLVMStructTypeInContext(
                self.0,
                field_tys.as_mut_ptr(),
                field_tys.len() as u32,
                0, /* !packed */
            ))
        }
    }

    pub fn create_opaque_named_struct(&self, name: &str) -> StructType {
        unsafe { StructType(LLVMStructCreateNamed(self.0, name.cstr())) }
    }

    pub fn const_string(&self, v: &str) -> ArrayValue {
        unsafe {
            ArrayValue(LLVMConstStringInContext2(
                self.0,
                v.cstr(),
                v.len(),
                true as i32, /* !null_terminated */
            ))
        }
    }

    pub fn const_int_array<T: PrimInt + ToPrimitive + 'static>(&self, v: &[T]) -> ArrayValue {
        let llty = self.llvm_type_from_rust_int_type::<T>();
        unsafe {
            let mut vals: Vec<_> = v
                .iter()
                .map(|x| Constant::int(llty, u256::U256::from((*x).to_u128().unwrap())).0)
                .collect();
            ArrayValue(LLVMConstArray2(
                llty.0,
                vals.as_mut_ptr(),
                vals.len() as u64,
            ))
        }
    }

    pub fn const_array(&self, vals: &[Constant], llty: Type) -> ArrayValue {
        let mut llvals: Vec<_> = vals.iter().map(|v| v.get0()).collect();
        unsafe {
            ArrayValue(LLVMConstArray2(
                llty.0,
                llvals.as_mut_ptr(),
                vals.len() as u64,
            ))
        }
    }

    pub fn const_struct(&self, fields: &[Constant]) -> Constant {
        unsafe {
            let mut fields: Vec<_> = fields.iter().map(|f| f.0).collect();
            Constant(LLVMConstStructInContext(
                self.0,
                fields.as_mut_ptr(),
                fields.len() as u32,
                false as i32, /* packed */
            ))
        }
    }

    pub fn const_named_struct(&self, fields: &[Constant], name: &str) -> Constant {
        unsafe {
            let tyref = LLVMGetTypeByName2(self.0, name.cstr());
            assert!(!tyref.is_null());
            let mut fields: Vec<_> = fields.iter().map(|f| f.0).collect();
            Constant(LLVMConstNamedStruct(
                tyref,
                fields.as_mut_ptr(),
                fields.len() as u32,
            ))
        }
    }

    pub fn abi_size_of_type(&self, data_layout: TargetData, ty: Type) -> usize {
        unsafe { LLVMABISizeOfType(data_layout.0, ty.0) as usize }
    }

    pub fn abi_alignment_of_type(&self, data_layout: TargetData, ty: Type) -> usize {
        unsafe { LLVMABIAlignmentOfType(data_layout.0, ty.0) as usize }
    }
}

#[derive(Copy, Clone)]
pub struct TargetData(LLVMTargetDataRef);

#[derive(Debug)]
pub struct Module(pub LLVMModuleRef, pub Rc<RefCell<String>>, pub String); // (module, asm, name)

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.0);
        }
    }
}

impl AsMut<llvm_sys::LLVMModule> for Module {
    fn as_mut(&mut self) -> &mut llvm_sys::LLVMModule {
        unsafe { &mut *self.0 }
    }
}

impl Module {
    pub fn set_target(&self, triple: &str) {
        unsafe {
            LLVMSetTarget(self.0, triple.cstr());
        }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.0);
        }
    }

    pub fn finalize(&self) {
        unsafe {
            let asm = self.1.borrow();
            LLVMSetModuleInlineAsm2(self.0, asm.as_ptr() as *const i8, asm.len());
            let ir_str_ptr = LLVMPrintModuleToString(self.0);
            let ir_str = CStr::from_ptr(ir_str_ptr);
            debug!("Generated LLVM IR:\n{}", ir_str.to_string_lossy());
            /*
            File::create(format!("{}.ll", self.2))
                .unwrap()
                .write_all(ir_str.to_bytes())
                .unwrap();
            */
            LLVMDisposeMessage(ir_str_ptr); // must free the string
        }
    }

    pub fn get_module_id(&self) -> String {
        let mut mod_len: ::libc::size_t = 0;
        let mod_ptr = unsafe { LLVMGetModuleIdentifier(self.0, &mut mod_len) };
        from_raw_slice_to_string(mod_ptr, mod_len)
    }

    pub fn get_module_source(&self) -> String {
        let mut mod_len: ::libc::size_t = 0;
        let mod_ptr = unsafe { LLVMGetSourceFileName(self.0, &mut mod_len) };
        from_raw_slice_to_string(mod_ptr, mod_len)
    }

    pub fn get_source_file_name(&self) -> String {
        let mut src_len: ::libc::size_t = 0;
        let src_ptr = unsafe { LLVMGetSourceFileName(self.0, &mut src_len) };
        from_raw_slice_to_string(src_ptr, src_len)
    }

    pub fn set_source_file_name(&self, name: &str) {
        unsafe { LLVMSetSourceFileName(self.0, name.as_ptr() as *const libc::c_char, name.len()) }
    }

    pub fn add_function(
        &self,
        exports: &mut Vec<String>,
        module: &str,
        name: &str,
        ty: FunctionType,
        polka_export: bool,
    ) -> Function {
        log::debug!("Adding function {module}:{name}");
        unsafe {
            let mut symbol = name.to_owned();
            if module != "native" {
                let hash = hash_string(format!("{module}::{name}").as_str());
                let mangled = format!(
                    "_ZN{}{}{}{}17h{}E",
                    module.len(),
                    module,
                    name.len(),
                    name,
                    hash
                );
                symbol = mangled;
            }
            let function = LLVMAddFunction(self.0, symbol.cstr(), ty.0);
            // TODO: it doesnt feel like the right place for polka section generation just on the fly
            // on any function we need to declare. Its looks more like additional pass when finalizing module
            // but we leave this for now to move forward
            if polka_export && !exports.contains(&symbol) {
                let context = LLVMGetModuleContext(self.0);
                let num_args = LLVMCountParams(function) as u8;
                add_polkavm_metadata(
                    self.0,
                    context,
                    module,
                    name,
                    symbol.as_str(),
                    num_args,
                    self.1.clone(),
                );
                exports.push(symbol.clone());
            }
            Function(function)
        }
    }

    pub fn get_named_function(&self, name: &str) -> Option<Function> {
        unsafe {
            let llfn = LLVMGetNamedFunction(self.0, name.cstr());
            if !llfn.is_null() {
                Some(Function(llfn))
            } else {
                None
            }
        }
    }

    // Add one or more enum/int attributes to `func`, where each attr is specified by:
    // LVMAttributeIndex: { LLVMAttributeReturnIndex, LLVMAttributeFunctionIndex,
    //                      or a parameter number from 1 to N. }.
    // &str: Attribute name from the LLVM LangRef.
    // Option<u64>: The attribute value (for int attributes) or None (for enum attributes).
    pub fn add_attributes(
        &self,
        func: Function,
        attrs: &[(llvm_sys::LLVMAttributeIndex, &str, Option<u64>)],
    ) {
        unsafe {
            let cx = LLVMGetModuleContext(self.0);
            for (idx, name, opt_val) in attrs {
                let kind_id = get_attr_kind_for_name(name);
                let attr_ref = LLVMCreateEnumAttribute(
                    cx,
                    kind_id.expect("attribute not found") as libc::c_uint,
                    opt_val.unwrap_or(0),
                );
                LLVMAddAttributeAtIndex(func.0, *idx, attr_ref);
            }
        }
    }

    pub fn add_type_attribute(
        &self,
        func: Function,
        idx: llvm_sys::LLVMAttributeIndex,
        name: &str,
        ty: Type,
    ) {
        unsafe {
            let cx = LLVMGetModuleContext(self.0);
            let kind_id = get_attr_kind_for_name(name);
            let attr_ref = LLVMCreateTypeAttribute(
                cx,
                kind_id.expect("attribute not found") as libc::c_uint,
                ty.0,
            );
            LLVMAddAttributeAtIndex(func.0, idx, attr_ref);
        }
    }

    // pub fn declare_known_functions(&self) {
    //     // Declare i32 @memcmp(ptr, ptr, i64).
    //     unsafe {
    //         let cx = LLVMGetModuleContext(self.0);
    //         let memcmp_arg_tys: Vec<Type> = vec![
    //             Type(LLVMPointerTypeInContext(cx, 0 as libc::c_uint)),
    //             Type(LLVMPointerTypeInContext(cx, 0 as libc::c_uint)),
    //             Type(LLVMInt64TypeInContext(cx)),
    //         ];
    //         let memcmp_rty = Type(LLVMInt32TypeInContext(cx));
    //         let memcmp_fty = FunctionType::new(memcmp_rty, &memcmp_arg_tys);
    //         self.add_function("native", "memcmp", memcmp_fty, false);
    //     }
    // }

    pub fn verify(&self) {
        use llvm_sys::analysis::*;
        unsafe {
            let name = &self.get_module_id();
            let addr = &self.0;
            debug!(target: "module", "{name} module verification address {addr:#?}");
            if LLVMVerifyModule(
                self.0,
                LLVMVerifierFailureAction::LLVMPrintMessageAction,
                ptr::null_mut(),
            ) == 1
            {
                println!("\n{} module verification failed\n", &self.get_module_id());
                let module_info = &self.print_to_str();
                debug!(target: "module", "Module content:\n{module_info}\n");
                abort();
            }
        }
    }

    pub fn set_data_layout(&self, machine: &TargetMachine) {
        unsafe {
            let target_data = LLVMCreateTargetDataLayout(machine.0);
            let layout_str = LLVMCopyStringRepOfTargetData(target_data);
            LLVMSetDataLayout(self.0, layout_str);
            LLVMDisposeMessage(layout_str);
            LLVMDisposeTargetData(target_data);
        }
    }

    pub fn get_module_data_layout(&self) -> TargetData {
        unsafe {
            let dl = LLVMGetModuleDataLayout(self.0);
            debug!(target: "dl", "\n{}", CStr::from_ptr(LLVMCopyStringRepOfTargetData(dl)).to_str().unwrap());
            TargetData(dl)
        }
    }

    pub fn get_global(&self, name: &str) -> Option<Global> {
        unsafe {
            let v = LLVMGetNamedGlobal(self.0, name.cstr());
            if v.is_null() {
                None
            } else {
                Some(Global(v))
            }
        }
    }

    pub fn add_global(&self, ty: Type, name: &str) -> Global {
        assert!(self.get_global(name).is_none());
        unsafe {
            let v = LLVMAddGlobal(self.0, ty.0, name.cstr());
            Global(v)
        }
    }

    pub fn add_global2(&self, ty: Type, name: &str) -> Global {
        unsafe {
            let v = LLVMAddGlobal(self.0, ty.0, name.cstr());
            Global(v)
        }
    }

    pub fn write_to_file(self, llvm_ir: bool, filename: &str) -> anyhow::Result<()> {
        use std::{fs::File, os::unix::io::AsRawFd};

        unsafe {
            if llvm_ir {
                if filename != "-" {
                    let mut err_string = ptr::null_mut();
                    let filename = CString::new(filename.to_string()).expect("interior nul byte");
                    let mut filename = filename.into_bytes_with_nul();
                    let filename: *mut u8 = filename.as_mut_ptr();
                    let filename = filename as *mut libc::c_char;
                    let res = LLVMPrintModuleToFile(self.0, filename, &mut err_string);

                    if res != 0 {
                        assert!(!err_string.is_null());
                        let msg = CStr::from_ptr(err_string).to_string_lossy();
                        LLVMDisposeMessage(err_string);
                        anyhow::bail!("{}", msg);
                    }
                } else {
                    let buf = LLVMPrintModuleToString(self.0);
                    assert!(!buf.is_null());
                    let cstr = CStr::from_ptr(buf);
                    print!("{}", cstr.to_string_lossy());
                    LLVMDisposeMessage(buf);
                }
            } else {
                if filename == "-" {
                    anyhow::bail!("Not writing bitcode to stdout");
                }
                let bc_file = File::create(filename)?;
                let res = llvm_sys::bit_writer::LLVMWriteBitcodeToFD(
                    self.0,
                    bc_file.as_raw_fd(),
                    false as i32,
                    true as i32,
                );

                if res != 0 {
                    anyhow::bail!("Failed to write bitcode to file");
                }
            }
        }

        Ok(())
    }

    pub fn print_to_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(LLVMPrintModuleToString(self.0))
                .to_str()
                .unwrap()
        }
    }
}

pub struct Switch(pub LLVMValueRef);

impl Switch {
    pub fn add_case(&self, value: Constant, bb: BasicBlock) {
        unsafe {
            LLVMAddCase(self.0, value.0, bb.0);
        }
    }

    pub fn get_default_dest(&self) -> BasicBlock {
        unsafe { BasicBlock(LLVMGetSwitchDefaultDest(self.0)) }
    }
}

pub struct Builder(pub LLVMBuilderRef);

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.0);
        }
    }
}

impl Builder {
    pub fn get_entry_basic_block(&self, f: Function) -> BasicBlock {
        unsafe { BasicBlock(LLVMGetEntryBasicBlock(f.0)) }
    }

    pub fn position_at_beginning(&self, bb: BasicBlock) {
        unsafe {
            let inst = LLVMGetFirstInstruction(bb.0);
            LLVMPositionBuilderBefore(self.0, inst);
        }
    }

    pub fn get_insert_block(&self) -> BasicBlock {
        unsafe { BasicBlock(LLVMGetInsertBlock(self.0)) }
    }

    pub fn position_at_end(&self, bb: BasicBlock) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.0, bb.0);
        }
    }

    pub fn build_alloca(&self, ty: Type, name: &str) -> Alloca {
        unsafe { Alloca(LLVMBuildAlloca(self.0, ty.0, name.cstr())) }
    }

    pub fn store_param_to_alloca(&self, param: Parameter, alloca: Alloca) {
        unsafe {
            LLVMBuildStore(self.0, param.0, alloca.0);
        }
    }

    pub fn build_switch(&self, val: AnyValue, else_bb: BasicBlock, num_cases: u32) -> Switch {
        unsafe {
            let switch = LLVMBuildSwitch(self.0, val.0, else_bb.0, num_cases);
            Switch(switch)
        }
    }

    /// Load an alloca and store in another.
    pub fn load_store(
        &self,
        ty: Type,
        src: Alloca,
        dst: Alloca,
    ) -> (*mut LLVMValue, *mut LLVMValue) {
        unsafe {
            let load = LLVMBuildLoad2(self.0, ty.0, src.0, "load_store_tmp".cstr());
            let store = LLVMBuildStore(self.0, load, dst.0);
            (load, store)
        }
    }

    /// Reference an alloca and store it in another.
    pub fn ref_store(&self, src: Alloca, dst: Alloca) {
        unsafe {
            // allocas are pointers, so we're just storing the value of one alloca in another
            LLVMBuildStore(self.0, src.0, dst.0);
        }
    }

    /// Load a struct pointer alloca, add a field offset to it, and store the new pointer value.
    pub fn field_ref_store(&self, src: Alloca, dst: Alloca, struct_ty: StructType, offset: usize) {
        unsafe {
            let ty = src.llvm_type().0;
            let tmp_reg = LLVMBuildLoad2(self.0, ty, src.0, "tmp".cstr());
            let field_ptr = LLVMBuildStructGEP2(
                self.0,
                struct_ty.0,
                tmp_reg,
                offset as libc::c_uint,
                "fld_ref".cstr(),
            );
            LLVMBuildStore(self.0, field_ptr, dst.0);
        }
    }

    /// Get a struct element.
    pub fn getelementptr(
        &self,
        val: AnyValue,
        struct_ty: &StructType,
        offset: usize,
        name: &str,
    ) -> AnyValue {
        unsafe {
            let ptr = LLVMBuildStructGEP2(
                self.0,
                struct_ty.0,
                val.0,
                offset as libc::c_uint,
                name.cstr(),
            );
            AnyValue(ptr)
        }
    }

    /// Get an address at a specific index from a pointer
    pub fn build_address_with_indices(
        &self,
        ty: Type,
        pointer: AnyValue,
        indices: &[AnyValue],
        name: &str,
    ) -> AnyValue {
        unsafe {
            let ptr = LLVMBuildGEP2(
                self.0,
                ty.0,
                pointer.0,
                indices.as_ptr() as *mut LLVMValueRef,
                indices.len() as libc::c_uint,
                name.cstr(),
            );
            AnyValue(ptr)
        }
    }

    /// Load a value.
    pub fn load(&self, val: AnyValue, ty: Type, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildLoad2(self.0, ty.0, val.0, name.cstr())) }
    }

    /// Store a value.
    pub fn store(&self, val: AnyValue, ptr: AnyValue) {
        unsafe {
            LLVMBuildStore(self.0, val.0, ptr.0);
        }
    }

    // Load the source fields, insert them into a new struct value, then store the struct value.
    pub fn insert_fields_and_store(
        &self,
        src: &[(Type, Alloca)],
        dst: (Type, Alloca),
        stype: StructType,
    ) {
        unsafe {
            let loads = src
                .iter()
                .enumerate()
                .map(|(i, (ty, val))| {
                    let name = format!("fv.{i}");
                    LLVMBuildLoad2(self.0, ty.0, val.0, name.cstr())
                })
                .collect::<Vec<_>>();

            let mut agg_val = LLVMGetUndef(stype.0);
            for (i, ld) in loads.iter().enumerate() {
                let s = format!("insert_{i}").cstr();
                agg_val = LLVMBuildInsertValue(self.0, agg_val, *ld, i as libc::c_uint, s);
            }

            assert_eq!(LLVMTypeOf(agg_val), dst.0 .0);
            LLVMBuildStore(self.0, agg_val, dst.1 .0);
        }
    }

    // Load the source struct, extract fields, then store each field in a local.
    pub fn load_and_extract_fields(
        &self,
        src: (Type, Alloca),
        dst: &[(Type, Alloca)],
        stype: StructType,
    ) {
        unsafe {
            assert_eq!(src.0 .0, stype.0);
            let srcval = LLVMBuildLoad2(self.0, stype.0, src.1 .0, "srcval".cstr());

            let user_field_count = dst.len();
            assert_eq!(
                user_field_count,
                LLVMCountStructElementTypes(stype.0) as usize
            );

            let mut extracts = Vec::with_capacity(user_field_count);
            for i in 0..user_field_count {
                let name = format!("ext_{i}");
                let ev = LLVMBuildExtractValue(self.0, srcval, i as libc::c_uint, name.cstr());
                extracts.push(ev);
            }

            for i in 0..user_field_count {
                assert_eq!(
                    dst[i].0 .0,
                    LLVMStructGetTypeAtIndex(stype.0, i as libc::c_uint)
                );
                LLVMBuildStore(self.0, extracts[i], dst[i].1 .0);
            }
        }
    }

    /// Load a pointer alloca, dereference, and store the value.
    pub fn load_deref_store(&self, ty: Type, src: Alloca, dst: Alloca) {
        unsafe {
            let tmp_reg1 = LLVMBuildLoad2(
                self.0,
                ty.ptr_type().0,
                src.0,
                "load_deref_store_tmp1".cstr(),
            );
            let tmp_reg2 = LLVMBuildLoad2(self.0, ty.0, tmp_reg1, "load_deref_store_tmp2".cstr());
            LLVMBuildStore(self.0, tmp_reg2, dst.0);
        }
    }

    /// Load a value from src alloca, store it to the location pointed to by dst alloca.
    pub fn load_store_ref(&self, ty: Type, src: Alloca, dst: Alloca) {
        unsafe {
            let src_reg = LLVMBuildLoad2(self.0, ty.0, src.0, "load_store_ref_src".cstr());
            let dst_ptr_reg = LLVMBuildLoad2(
                self.0,
                ty.ptr_type().0,
                dst.0,
                "load_store_ref_dst_ptr".cstr(),
            );
            LLVMBuildStore(self.0, src_reg, dst_ptr_reg);
        }
    }

    pub fn build_return_void(&self) {
        unsafe {
            LLVMBuildRetVoid(self.0);
        }
    }

    pub fn build_return(&self, val: AnyValue) {
        unsafe {
            LLVMBuildRet(self.0, val.0);
        }
    }

    pub fn load_return(&self, ty: Type, val: Alloca) {
        unsafe {
            let tmp_reg = LLVMBuildLoad2(self.0, ty.0, val.0, "retval".cstr());
            LLVMBuildRet(self.0, tmp_reg);
        }
    }

    pub fn load_multi_return(&self, return_ty: Type, vals: &[(Type, Alloca)]) {
        unsafe {
            let loads = vals
                .iter()
                .enumerate()
                .map(|(i, (ty, val))| {
                    let name = format!("rv.{i}");
                    LLVMBuildLoad2(self.0, ty.0, val.0, name.cstr())
                })
                .collect::<Vec<_>>();

            let mut agg_val = LLVMGetUndef(return_ty.0);
            for (i, load) in loads.into_iter().enumerate() {
                let s = format!("insert_{i}").cstr();
                agg_val = LLVMBuildInsertValue(self.0, agg_val, load, i as libc::c_uint, s);
            }
            LLVMBuildRet(self.0, agg_val);
        }
    }

    pub fn store_const(&self, src: Constant, dst: Alloca) {
        unsafe {
            LLVMBuildStore(self.0, src.0, dst.0);
        }
    }

    pub fn build_br(&self, bb: BasicBlock) {
        unsafe {
            LLVMBuildBr(self.0, bb.0);
        }
    }

    pub fn build_cond_br(&self, cnd_reg: AnyValue, bb0: BasicBlock, bb1: BasicBlock) {
        unsafe {
            LLVMBuildCondBr(self.0, cnd_reg.0, bb0.0, bb1.0);
        }
    }

    pub fn load_cond_br(&self, ty: Type, val: Alloca, bb0: BasicBlock, bb1: BasicBlock) {
        unsafe {
            let cnd_reg = LLVMBuildLoad2(self.0, ty.0, val.0, "cnd".cstr());
            LLVMBuildCondBr(self.0, cnd_reg, bb0.0, bb1.0);
        }
    }

    pub fn build_extract_value(&self, agg_val: AnyValue, index: u32, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildExtractValue(self.0, agg_val.0, index, name.cstr())) }
    }

    // Build call to an intrinsic (use the 'types' parameter for overloaded intrinsics).
    pub fn build_intrinsic_call(
        &self,
        module: &Module,
        iname: &str,
        types: &[Type],
        args: &[AnyValue],
        resname: &str,
    ) -> AnyValue {
        let mut tys = types.iter().map(|ty| ty.0).collect::<Vec<_>>();
        let mut args = args.iter().map(|arg| arg.0).collect::<Vec<_>>();

        unsafe {
            let iid = LLVMLookupIntrinsicID(iname.cstr(), iname.len());
            let fv = LLVMGetIntrinsicDeclaration(module.0, iid, tys.as_mut_ptr(), tys.len());
            assert_eq!(LLVMIsAFunction(fv), fv);

            let cx = LLVMGetModuleContext(module.0);
            let fnty = LLVMIntrinsicGetType(cx, iid, tys.as_mut_ptr(), tys.len());
            AnyValue(LLVMBuildCall2(
                self.0,
                fnty,
                fv,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                resname.cstr(),
            ))
        }
    }

    pub fn load_alloca(&self, val: Alloca, ty: Type) -> AnyValue {
        unsafe {
            let name = "loaded_alloca";
            AnyValue(LLVMBuildLoad2(self.0, ty.0, val.0, name.cstr()))
        }
    }

    pub fn call(&self, fnval: Function, args: &[AnyValue]) -> AnyValue {
        let fnty = fnval.llvm_type();

        unsafe {
            let mut args = args.iter().map(|val| val.0).collect::<Vec<_>>();
            AnyValue(LLVMBuildCall2(
                self.0,
                fnty.0,
                fnval.0,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                "".cstr(),
            ))
        }
    }

    pub fn call_store(&self, fnval: Function, args: &[AnyValue], dst: &[(Type, Alloca)]) {
        let fnty = fnval.llvm_type();

        unsafe {
            let mut args = args.iter().map(|a| a.0).collect::<Vec<_>>();
            let func_name = get_name(fnval.0);
            let ret = LLVMBuildCall2(
                self.0,
                fnty.0,
                fnval.0,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                (if dst.is_empty() { "" } else { "retval" }).cstr(),
            );
            let ret_name = get_name(ret);
            debug!(target: "functions", "call_store function {} ret {}", &func_name, &ret_name);

            if dst.is_empty() {
                // No return values.
            } else if dst.len() == 1 {
                // Single return value.
                let alloca = dst[0].1;
                let alloca_name = alloca.get_name();
                let ret = LLVMBuildStore(self.0, ret, dst[0].1 .0);
                let ret_name = get_name(ret);
                debug!(target: "functions", "call_store alloca_name {} ret {} alloca {:#?} ", &alloca_name, &ret_name, alloca);
            } else {
                // Multiple return values-- unwrap the struct.
                let extracts = dst
                    .iter()
                    .enumerate()
                    .map(|(i, (_ty, dval))| {
                        let _name = dval.get_name();
                        let name = format!("extract_{i}");
                        let ev = LLVMBuildExtractValue(self.0, ret, i as libc::c_uint, name.cstr());
                        (ev, dval)
                    })
                    .collect::<Vec<_>>();
                for (ev, dval) in extracts {
                    LLVMBuildStore(self.0, ev, dval.0);
                }
            }
        }
    }

    pub fn load_call_store(
        &self,
        fnval: Function,
        args: &[(Type, Alloca)],
        dst: &[(Type, Alloca)],
        instr_dbg: super::dwarf::PublicInstruction<'_>,
    ) {
        unsafe {
            let args = args
                .iter()
                .enumerate()
                .map(|(i, (ty, val))| {
                    let name = format!("call_arg_{i}");
                    AnyValue(LLVMBuildLoad2(self.0, ty.0, val.0, name.cstr()))
                })
                .collect::<Vec<_>>();
            self.call_store_with_dst(fnval, &args, dst, instr_dbg)
        }
    }

    fn call_store_with_dst(
        &self,
        fnval: Function,
        args: &[AnyValue],
        dst: &[(Type, Alloca)],
        instr_dbg: super::dwarf::PublicInstruction<'_>,
    ) {
        let fnty = fnval.llvm_type();

        unsafe {
            let mut args = args.iter().map(|a| a.0).collect::<Vec<_>>();
            let func_name = get_name(fnval.0);
            let ret = LLVMBuildCall2(
                self.0,
                fnty.0,
                fnval.0,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                (if dst.is_empty() { "" } else { "retval" }).cstr(),
            );
            let ret_name = get_name(ret);
            instr_dbg.create_call(ret);
            debug!(target: "functions", "call_store function {} ret {}", &func_name, &ret_name);

            if dst.is_empty() {
                // No return values.
            } else if dst.len() == 1 {
                // Single return value.
                let alloca = dst[0].1;
                let alloca_name = alloca.get_name();
                let ret = LLVMBuildStore(self.0, ret, dst[0].1 .0);
                let ret_name = get_name(ret);
                debug!(target: "functions", "call_store alloca_name {} ret {} alloca {:#?} ", &alloca_name, &ret_name, alloca);
            } else {
                // Multiple return values-- unwrap the struct.
                let extracts = dst
                    .iter()
                    .enumerate()
                    .map(|(i, (_ty, dval))| {
                        let _name = dval.get_name();
                        let name = format!("extract_{i}");
                        let ev = LLVMBuildExtractValue(self.0, ret, i as libc::c_uint, name.cstr());
                        (ev, dval)
                    })
                    .collect::<Vec<_>>();
                for (ev, dval) in extracts {
                    LLVMBuildStore(self.0, ev, dval.0);
                }
            }
        }
    }

    pub fn build_call_imm(&self, fnval: Function, args: &[Constant]) {
        let fnty = fnval.llvm_type();
        unsafe {
            let mut args = args.iter().map(|val| val.0).collect::<Vec<_>>();
            LLVMBuildCall2(
                self.0,
                fnty.0,
                fnval.0,
                args.as_mut_ptr(),
                args.len() as libc::c_uint,
                "".cstr(),
            );
        }
    }

    pub fn build_unreachable(&self) {
        unsafe {
            LLVMBuildUnreachable(self.0);
        }
    }

    pub fn build_load(&self, ty: Type, src0_reg: Alloca, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildLoad2(self.0, ty.0, src0_reg.0, name.cstr())) }
    }

    pub fn build_load_from_valref(&self, ty: Type, src0_reg: AnyValue, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildLoad2(self.0, ty.0, src0_reg.0, name.cstr())) }
    }

    pub fn build_load_global_const(&self, gval: Global) -> Constant {
        unsafe {
            let ty = LLVMGlobalGetValueType(gval.0);
            Constant(LLVMBuildLoad2(self.0, ty, gval.0, "".cstr()))
        }
    }

    pub fn build_store(&self, dst_reg: AnyValue, dst: Alloca) {
        unsafe {
            LLVMBuildStore(self.0, dst_reg.0, dst.0);
        }
    }

    #[allow(dead_code)]
    pub fn load_add_store(&self, ty: Type, src0: Alloca, src1: Alloca, dst: Alloca) {
        unsafe {
            let src0_reg = LLVMBuildLoad2(self.0, ty.0, src0.0, "add_src_0".cstr());
            let src1_reg = LLVMBuildLoad2(self.0, ty.0, src1.0, "add_src_1".cstr());
            let dst_reg = LLVMBuildAdd(self.0, src0_reg, src1_reg, "add_dst".cstr());
            LLVMBuildStore(self.0, dst_reg, dst.0);
        }
    }

    pub fn build_binop(
        &self,
        op: LLVMOpcode,
        lhs: AnyValue,
        rhs: AnyValue,
        name: &str,
    ) -> AnyValue {
        unsafe { AnyValue(LLVMBuildBinOp(self.0, op, lhs.0, rhs.0, name.cstr())) }
    }
    pub fn build_compare(
        &self,
        pred: LLVMIntPredicate,
        lhs: AnyValue,
        rhs: AnyValue,
        name: &str,
    ) -> AnyValue {
        unsafe { AnyValue(LLVMBuildICmp(self.0, pred, lhs.0, rhs.0, name.cstr())) }
    }
    #[allow(dead_code)]
    pub fn build_unary_bitcast(&self, val: AnyValue, dest_ty: Type, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildBitCast(self.0, val.0, dest_ty.0, name.cstr())) }
    }
    pub fn build_zext(&self, val: AnyValue, dest_ty: Type, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildZExt(self.0, val.0, dest_ty.0, name.cstr())) }
    }
    pub fn build_trunc(&self, val: AnyValue, dest_ty: Type, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildTrunc(self.0, val.0, dest_ty.0, name.cstr())) }
    }

    pub fn wrap_as_any_value(&self, val: LLVMValueRef) -> AnyValue {
        AnyValue(val)
    }

    pub fn build_pointer_to_int(&self, val: AnyValue, dest_ty: Type, name: &str) -> AnyValue {
        unsafe { AnyValue(LLVMBuildPtrToInt(self.0, val.0, dest_ty.0, name.cstr())) }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Type(pub LLVMTypeRef);

impl Type {
    pub fn ptr_type(&self) -> Type {
        unsafe { Type(LLVMPointerType(self.0, 0)) }
    }

    pub fn as_struct_type(&self) -> StructType {
        StructType(self.0)
    }

    pub fn get_int_type_width(&self) -> u32 {
        unsafe { LLVMGetIntTypeWidth(self.0) }
    }

    pub fn is_integer_ty(&self) -> bool {
        unsafe { LLVMGetTypeKind(self.0) == LLVMIntegerTypeKind }
    }

    pub fn get_context(&self) -> Context {
        unsafe { Context(LLVMGetTypeContext(self.0)) }
    }

    pub fn get_array_length(&self) -> usize {
        unsafe { LLVMGetArrayLength2(self.0) as usize }
    }

    pub fn get_element_type(&self) -> Type {
        unsafe { Type(LLVMGetElementType(self.0)) }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpType(self.0);
            eprintln!();
        }
    }

    pub fn dump_properties_to_str(&self, data_layout: TargetData) -> String {
        unsafe {
            let ty = self.0;
            let s = &format!(
                "StoreSizeOfType: {}\nABISizeOfType: {}\nABIAlignmnetOfType: {}\nSizeOfTypeInBits: {}\n",
                LLVMStoreSizeOfType(data_layout.0, ty) as u32,
                LLVMABISizeOfType(data_layout.0, ty) as u32,
                LLVMABIAlignmentOfType(data_layout.0, ty),
                LLVMSizeOfTypeInBits(data_layout.0, ty) as u32,
            );
            s.to_string()
        }
    }

    pub fn store_size_of_type(&self, data_layout: TargetData) -> u64 {
        unsafe { LLVMStoreSizeOfType(data_layout.0, self.0) }
    }

    pub fn abi_size_of_type(&self, data_layout: TargetData) -> u64 {
        unsafe { LLVMABISizeOfType(data_layout.0, self.0) }
    }

    pub fn abi_alignment_of_type(&self, data_layout: TargetData) -> u32 {
        unsafe { LLVMABIAlignmentOfType(data_layout.0, self.0) }
    }

    pub fn size_of_type_in_bits(&self, data_layout: TargetData) -> u64 {
        unsafe { LLVMSizeOfTypeInBits(data_layout.0, self.0) }
    }

    pub fn call_frame_alignment_of_type(&self, data_layout: TargetData) -> u32 {
        unsafe { LLVMCallFrameAlignmentOfType(data_layout.0, self.0) }
    }

    pub fn preferred_alignment_of_type(&self, data_layout: TargetData) -> u32 {
        unsafe { LLVMPreferredAlignmentOfType(data_layout.0, self.0) }
    }

    pub fn element_at_offset(
        &self,
        data_layout: TargetData,
        struct_ty: &LLVMTypeRef,
        offset: u64,
    ) -> u32 {
        unsafe { LLVMElementAtOffset(data_layout.0, *struct_ty, offset as ::libc::c_ulonglong) }
    }

    pub fn offset_of_element(
        &self,
        data_layout: TargetData,
        struct_ty: &LLVMTypeRef,
        offset: u32,
    ) -> u64 {
        unsafe { LLVMOffsetOfElement(data_layout.0, *struct_ty, offset as ::libc::c_uint) }
    }

    pub fn print_to_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(self.0))
                .to_str()
                .unwrap()
        }
    }
}

#[derive(Copy, Clone)]
pub struct StructType(LLVMTypeRef);

impl StructType {
    pub fn as_any_type(&self) -> Type {
        Type(self.0)
    }

    pub fn ptr_type(&self) -> Type {
        unsafe { Type(LLVMPointerType(self.0, 0)) }
    }

    pub fn get_context(&self) -> Context {
        unsafe { Context(LLVMGetTypeContext(self.0)) }
    }

    pub fn set_struct_body(&self, field_tys: &[Type]) {
        unsafe {
            let mut field_tys: Vec<_> = field_tys.iter().map(|f| f.0).collect();
            LLVMStructSetBody(
                self.0,
                field_tys.as_mut_ptr(),
                field_tys.len() as u32,
                0, /* !packed */
            );
        }
    }

    pub fn count_struct_element_types(&self) -> usize {
        unsafe { LLVMCountStructElementTypes(self.0) as usize }
    }

    pub fn struct_get_type_at_index(&self, idx: usize) -> Type {
        unsafe { Type(LLVMStructGetTypeAtIndex(self.0, idx as libc::c_uint)) }
    }

    pub fn offset_of_element(&self, data_layout: TargetData, idx: usize) -> usize {
        unsafe { LLVMOffsetOfElement(data_layout.0, self.0, idx as libc::c_uint) as usize }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpType(self.0);
            eprintln!();
        }
    }

    pub fn dump_to_string(&self) -> &str {
        let c_char_ptr = unsafe { LLVMPrintTypeToString(self.0) };
        let c_str = unsafe { CStr::from_ptr(c_char_ptr) };
        let str_slice = c_str.to_str().expect("Failed to convert CStr to str");
        str_slice
    }
}

#[derive(Copy, Clone)]
pub struct FunctionType(LLVMTypeRef);

impl FunctionType {
    pub fn new(return_type: Type, parameter_types: &[Type]) -> FunctionType {
        let mut parameter_types: Vec<_> = parameter_types.iter().map(|t| t.0).collect();
        unsafe {
            FunctionType(LLVMFunctionType(
                return_type.0,
                parameter_types.as_mut_ptr(),
                parameter_types.len() as libc::c_uint,
                false as LLVMBool,
            ))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Function(pub LLVMValueRef);

impl Function {
    pub fn as_gv(&self) -> Global {
        Global(self.0)
    }

    pub fn get_name(&self) -> String {
        get_name(self.0)
    }

    pub fn get_next_basic_block(&self, basic_block: BasicBlock) -> Option<BasicBlock> {
        let next_bb = unsafe { BasicBlock(LLVMGetNextBasicBlock(basic_block.0)) };
        if next_bb.0.is_null() {
            return None;
        }
        Some(next_bb)
    }

    pub fn append_basic_block(&self, name: &str) -> BasicBlock {
        unsafe { BasicBlock(LLVMAppendBasicBlock(self.0, name.cstr())) }
    }

    pub fn prepend_basic_block(&self, basic_block: BasicBlock, name: &str) -> BasicBlock {
        unsafe { BasicBlock(LLVMInsertBasicBlock(basic_block.0, name.cstr())) }
    }

    pub fn insert_basic_block_after(&self, basic_block: BasicBlock, name: &str) -> BasicBlock {
        match self.get_next_basic_block(basic_block) {
            Some(bb) => self.prepend_basic_block(bb, name),
            None => self.append_basic_block(name),
        }
    }

    pub fn count_params(&self) -> ::libc::c_uint {
        unsafe { LLVMCountParams(self.0) }
    }

    pub fn get_param(&self, i: usize) -> Parameter {
        unsafe { Parameter(LLVMGetParam(self.0, i as u32)) }
    }

    pub fn get_params(&self) -> Vec<Parameter> {
        let param_count = self.count_params();
        let mut params: Vec<Parameter> = vec![];
        for idx in 0..param_count {
            params.push(self.get_param(idx as usize));
        }
        params
    }

    pub fn llvm_type(&self) -> FunctionType {
        unsafe { FunctionType(LLVMGlobalGetValueType(self.0)) }
    }

    pub fn llvm_return_type(&self) -> Type {
        unsafe { Type(LLVMGetReturnType(LLVMGlobalGetValueType(self.0))) }
    }

    pub fn verify(&self, module_cx: &ModuleContext<'_, '_>) {
        use llvm_sys::analysis::*;
        let module_info = module_cx.llvm_module.print_to_str();
        trace!(target: "verify function", "Module content:");
        trace!(target: "verify function", "------------------------------");
        trace!(target: "verify function", "{module_info}");
        trace!(target: "verify function", "------------------------------");
        unsafe {
            if LLVMVerifyFunction(self.0, LLVMVerifierFailureAction::LLVMPrintMessageAction) == 1 {
                println!("{} function verifiction failed", &self.get_name());
                abort();
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BasicBlock(LLVMBasicBlockRef);

impl BasicBlock {
    pub fn get_basic_block_parent(&self) -> Function {
        unsafe { Function(LLVMGetBasicBlockParent(self.0)) }
    }
    pub fn get_basic_block_ref(&self) -> &LLVMBasicBlockRef {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Alloca(LLVMValueRef);

impl Alloca {
    pub fn as_any_value(&self) -> AnyValue {
        AnyValue(self.0)
    }

    pub fn as_constant(&self) -> Constant {
        Constant(self.0)
    }

    pub fn llvm_type(&self) -> Type {
        unsafe { Type(LLVMTypeOf(self.0)) }
    }
    pub fn get0(&self) -> LLVMValueRef {
        self.0
    }

    pub fn set_name(&self, name: &str) {
        let value = self.0;
        let cstr = std::ffi::CString::new(name).expect("CString conversion failed");
        unsafe {
            LLVMSetValueName2(value, cstr.as_ptr(), cstr.as_bytes().len());
        }
    }

    pub fn get_name(&self) -> String {
        let value = self.0;
        let mut length: ::libc::size_t = 0;
        let name_ptr = unsafe { LLVMGetValueName2(value, &mut length) };
        let name_cstr = unsafe { std::ffi::CStr::from_ptr(name_ptr) };
        name_cstr.to_string_lossy().into_owned()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AnyValue(LLVMValueRef);

impl AnyValue {
    pub fn get0(&self) -> LLVMValueRef {
        self.0
    }

    pub fn llvm_type(&self) -> Type {
        unsafe { Type(LLVMTypeOf(self.0)) }
    }

    pub fn as_constant(&self) -> Constant {
        Constant(self.0)
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpValue(self.0);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Global(LLVMValueRef);

impl Global {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn from_array(
        llvm_cx: &Context,
        builder: &Builder,
        module: LLVMModuleRef,
        bytes: &[u8],
    ) -> Self {
        unsafe {
            let i8_type = LLVMInt8TypeInContext(LLVMGetGlobalContext());
            let array_ty = LLVMArrayType2(i8_type, bytes.len() as u64);
            let values: Vec<LLVMValueRef> = bytes
                .iter()
                .map(|&b| LLVMConstInt(i8_type, b as u64, 0))
                .collect();
            let const_array =
                LLVMConstArray2(array_ty, values.as_ptr() as *mut _, bytes.len() as u64);
            let cname = std::ffi::CString::new("struct_tag").unwrap();
            let global = LLVMAddGlobal(module, array_ty, cname.as_ptr());

            LLVMSetInitializer(global, const_array);
            LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);

            let global = AnyValue(global);
            let i8_ptr_type = llvm_cx.ptr_type();

            // LLVM is not happy with the global as is, we need to cast it to a pointer type.
            let tag_ptr_cast =
                builder.build_unary_bitcast(global, i8_ptr_type, "struct_tag_as_i8_ptr");

            Global(tag_ptr_cast.0)
        }
    }

    pub fn ptr(&self) -> Constant {
        Constant(self.0)
    }

    pub fn as_any_value(&self) -> AnyValue {
        AnyValue(self.0)
    }

    pub fn set_alignment(&self, align: usize) {
        unsafe {
            LLVMSetAlignment(self.0, align as libc::c_uint);
        }
    }

    pub fn set_constant(&self) {
        unsafe {
            LLVMSetGlobalConstant(self.0, true as i32);
        }
    }

    pub fn set_linkage(&self, linkage: LLVMLinkage) {
        unsafe {
            LLVMSetLinkage(self.0, linkage);
        }
    }

    pub fn set_unnamed_addr(&self) {
        unsafe {
            LLVMSetUnnamedAddress(self.0, LLVMUnnamedAddr::LLVMGlobalUnnamedAddr);
        }
    }

    pub fn set_initializer(&self, v: Constant) {
        unsafe {
            LLVMSetInitializer(self.0, v.0);
        }
    }

    pub fn set_internal_linkage(&self) {
        unsafe {
            LLVMSetLinkage(self.0, LLVMInternalLinkage);
        }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpValue(self.0);
            eprintln!();
        }
    }

    pub fn print_to_str(&self) -> &str {
        unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.0))
                .to_str()
                .unwrap()
        }
    }
}

pub struct Parameter(pub LLVMValueRef);

impl Parameter {
    pub fn as_any_value(&self) -> AnyValue {
        AnyValue(self.0)
    }
}

pub struct Constant(LLVMValueRef);

impl Constant {
    pub fn as_any_value(&self) -> AnyValue {
        AnyValue(self.0)
    }

    pub fn const_int(ty: Type, v: u64, sign_extend: i32) -> Constant {
        unsafe { Constant(LLVMConstInt(ty.0, v, sign_extend)) }
    }

    pub fn int(ty: Type, v: u256::U256) -> Constant {
        unsafe {
            let val_as_str = format!("{v}");
            Constant(LLVMConstIntOfString(ty.0, val_as_str.cstr(), 10))
        }
    }

    pub fn get_const_null(ty: Type) -> Constant {
        unsafe { Constant(LLVMConstNull(ty.0)) }
    }

    pub fn get0(&self) -> LLVMValueRef {
        self.0
    }

    pub fn llvm_type(&self) -> Type {
        unsafe { Type(LLVMTypeOf(self.0)) }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpValue(self.0);
            eprintln!();
        }
    }
}

pub struct ArrayValue(LLVMValueRef);

impl ArrayValue {
    pub fn as_const(&self) -> Constant {
        Constant(self.0)
    }

    pub fn llvm_type(&self) -> Type {
        unsafe { Type(LLVMTypeOf(self.0)) }
    }
}

pub struct Target(LLVMTargetRef);

impl Target {
    pub fn from_triple(triple: &str) -> anyhow::Result<Target> {
        unsafe {
            let target: &mut LLVMTargetRef = &mut ptr::null_mut();
            let error: &mut *mut libc::c_char = &mut ptr::null_mut();
            let result = LLVMGetTargetFromTriple(triple.cstr(), target, error);

            if result == 0 {
                assert!((*error).is_null());
                Ok(Target(*target))
            } else {
                assert!(!(*error).is_null());
                let rust_error = CStr::from_ptr(*error).to_str()?.to_string();
                LLVMDisposeMessage(*error);
                anyhow::bail!("{rust_error}");
            }
        }
    }

    fn map_opt_level(opt_level: &str) -> LLVMCodeGenOptLevel {
        match opt_level {
            "none" => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            "less" => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            "default" => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            "aggressive" => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            _ => {
                warn!("Invalid opt level: {opt_level}, defaulting to \'none\'");
                LLVMCodeGenOptLevel::LLVMCodeGenLevelNone
            }
        }
    }
    pub fn create_target_machine(
        &self,
        triple: &str,
        cpu: &str,
        features: &str,
        opt_level: &str,
    ) -> TargetMachine {
        debug!(
            "Creating target machine with triple: {triple}, cpu: {cpu}, features: {features}, opt_level: {opt_level}"
        );
        unsafe {
            let reloc = LLVMRelocMode::LLVMRelocPIC;
            let code_model = LLVMCodeModel::LLVMCodeModelDefault;

            let machine = LLVMCreateTargetMachine(
                self.0,
                triple.cstr(),
                cpu.cstr(),
                features.cstr(),
                Self::map_opt_level(opt_level),
                reloc,
                code_model,
            );

            TargetMachine(machine)
        }
    }
}

pub struct TargetMachine(LLVMTargetMachineRef);

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.0);
        }
    }
}

impl TargetMachine {
    pub fn emit_to_obj_file(&self, module: &Module, filename: &str) -> anyhow::Result<()> {
        unsafe {
            // nb: llvm-sys seemingly-incorrectly wants
            // a mutable c-string for the filename.
            let filename = CString::new(filename.to_string()).expect("interior nul byte");
            let mut filename = filename.into_bytes_with_nul();
            let filename: *mut u8 = filename.as_mut_ptr();
            let filename = filename as *mut libc::c_char;

            let error: &mut *mut libc::c_char = &mut ptr::null_mut();
            let result = LLVMTargetMachineEmitToFile(
                self.0,
                module.0,
                filename,
                LLVMCodeGenFileType::LLVMObjectFile,
                error,
            );

            if result == 0 {
                assert!((*error).is_null());
                Ok(())
            } else {
                assert!(!(*error).is_null());
                let rust_error = CStr::from_ptr(*error).to_str()?.to_string();
                LLVMDisposeMessage(*error);
                anyhow::bail!("{rust_error}");
            }
        }
    }
}

unsafe fn add_polkavm_metadata(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    module_name: &str,
    fn_name: &str,
    mangled_fn_name: &str,
    num_args: u8,
    asm: Rc<RefCell<String>>,
) {
    debug!("Adding PolkaVM metadata for function: {fn_name} in module: {module_name}");
    // Create the metadata symbol
    let i8_type = LLVMInt8TypeInContext(context);
    let array_ty = LLVMArrayType2(i8_type, fn_name.len() as u64);

    let mut struct_field_types = [array_ty];
    let struct_ty = LLVMStructType(struct_field_types.as_mut_ptr(), 1, 0);
    let text = CString::new(fn_name).unwrap();
    let const_array = LLVMConstStringInContext2(context, text.as_ptr(), text.as_bytes().len(), 1);
    let mut struct_values = [const_array];
    let const_struct = LLVMConstStruct(struct_values.as_mut_ptr(), 1, 0);
    let hashed = hash_string(fn_name);
    let metadata_str = CString::new(format!("alloc_{hashed}")).unwrap();
    let metadata_global = LLVMAddGlobal(module, struct_ty, metadata_str.as_ptr());
    LLVMSetInitializer(metadata_global, const_struct);
    LLVMSetLinkage(metadata_global, LLVMLinkage::LLVMPrivateLinkage);
    LLVMSetUnnamedAddress(metadata_global, LLVMUnnamedAddr::LLVMGlobalUnnamedAddr);
    LLVMSetAlignment(metadata_global, 1);
    LLVMSetGlobalConstant(metadata_global, 1);
    LLVMSetSection(
        metadata_global,
        CString::new(format!(".rodata..Lalloc_{hashed}"))
            .unwrap()
            .as_ptr(),
    );

    // Define metadata data
    let i8_type = LLVMInt8TypeInContext(context);
    let ptr_type = LLVMPointerType(i8_type, 0);
    let arr9_type = LLVMArrayType2(i8_type, 9);
    let arr2_type = LLVMArrayType2(i8_type, 2);
    let mut field_types = [arr9_type, ptr_type, arr2_type];
    let metadata_struct_ty = LLVMStructType(field_types.as_mut_ptr(), 3, 1);
    let mut byte_consts_field0 = Vec::with_capacity(9);
    // version
    byte_consts_field0.push(LLVMConstInt(i8_type, 1, 0));
    // flags -> 0
    for _ in 0..4 {
        byte_consts_field0.push(LLVMConstInt(i8_type, 0, 0));
    }
    // function name length
    let bytes_field0 = (fn_name.len() as u64).to_le_bytes();
    for &b in &bytes_field0 {
        byte_consts_field0.push(LLVMConstInt(i8_type, b as u64, 0));
    }
    // pointer to the symbol
    let const_arr9 = LLVMConstArray2(i8_type, byte_consts_field0.as_mut_ptr(), 9);
    let const_ptr = LLVMConstPointerCast(metadata_global, ptr_type);
    // number of input and output args
    let bytes_field2: [u64; 2] = [num_args as u64, 1];
    let mut byte_consts_field2 = Vec::with_capacity(2);
    for &b in &bytes_field2 {
        byte_consts_field2.push(LLVMConstInt(i8_type, b, 0));
    }
    let const_arr2 = LLVMConstArray2(i8_type, byte_consts_field2.as_mut_ptr(), 2);
    let mut metadata_fields = [const_arr9, const_ptr, const_arr2];
    let metadata_const = LLVMConstStruct(metadata_fields.as_mut_ptr(), 3, 1);

    let mangled = format!(
        "_ZN{}{}{}{}8METADATA17h{}E",
        module_name.len(),
        module_name,
        fn_name.len(),
        fn_name,
        hash_string("METADATA")
    );
    let metadata = LLVMAddGlobal(
        module,
        metadata_struct_ty,
        CString::new(mangled.clone()).unwrap().as_ptr(),
    );
    LLVMSetInitializer(metadata, metadata_const);
    LLVMSetAlignment(metadata, 1);
    LLVMSetGlobalConstant(metadata, 1);
    LLVMSetSection(
        metadata,
        CString::new(".polkavm_metadata").unwrap().as_ptr(),
    );
    LLVMSetLinkage(metadata, LLVMLinkage::LLVMInternalLinkage);

    asm.borrow_mut().push_str(
        format!(
        ".pushsection .polkavm_exports,\"R\",@note\n.byte 1\n.8byte {mangled}\n.8byte {mangled_fn_name}\n.popsection\n"
    )
        .as_str(),
    );
}

fn hash_string(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();
    hex::encode(hash.to_be_bytes())
}
