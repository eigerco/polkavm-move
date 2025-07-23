use crate::{options::Options, run_to_polka};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use core::mem::MaybeUninit;
use gix::{
    progress::Discard,
    remote::{fetch::Shallow, Direction},
};
use log::{debug, info, trace, warn};
use move_package::source_package::{
    layout::SourcePackageLayout, manifest_parser, parsed_manifest::SubstOrRename,
};
use polkavm::{
    Caller, Config, Engine, Instance, InterruptKind, Linker, MemoryAccessError, Module,
    ModuleConfig, ProgramBlob, RawInstance, Reg,
};
use polkavm_move_native::{
    allocator::MemAllocator,
    host::{ProgramError, Runtime},
    types::{MoveAddress, MoveByteVector, MoveSigner, MoveType, TypeDesc},
    ALLOC_CODE, HEAP_BASE, PANIC_CODE,
};
use sha2::Digest;
use std::{
    collections::HashMap, fs::create_dir_all, num::NonZero, path::Path, sync::atomic::AtomicBool,
};

pub fn create_colored_stdout() -> StandardStream {
    let color = if atty::is(atty::Stream::Stderr) && atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };
    StandardStream::stderr(color)
}

pub fn parse_to_blob(program_bytes: &[u8]) -> anyhow::Result<ProgramBlob> {
    ProgramBlob::parse(program_bytes.into()).map_err(|e| anyhow::anyhow!("{e:?}"))
}

#[derive(Debug, Default)]
pub struct BuildOptions {
    options: Options,
}

impl BuildOptions {
    pub fn new(output_file: &str) -> Self {
        let options = Options {
            output: output_file.to_string(),
            llvm_ir: false,
            ..Default::default()
        };
        Self { options }
    }

    pub fn source(mut self, source_file: &str) -> Self {
        self.options.sources.push(source_file.to_string());
        self
    }

    pub fn address_mapping(mut self, mapping: String) -> Self {
        self.options.named_address_mapping.push(mapping);
        self
    }

    pub fn dependency(mut self, dependency_path: &str) -> Self {
        self.options.dependencies.push(dependency_path.to_string());
        self
    }

    pub fn build(self) -> Options {
        self.options
    }
}

pub fn build_polka_from_move(options: BuildOptions) -> anyhow::Result<Vec<u8>> {
    let output_file = options.options.output.clone();
    // parse move source files
    let mut color_writer = create_colored_stdout();
    run_to_polka(&mut color_writer, options.options)?;

    //TODO it would be so nice if compile won't access FS directly so we can work purely in-memory
    let data = std::fs::read(output_file)?;
    Ok(data)
}

pub fn load_from_elf_with_polka_linker(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    // config is taken from polkatool with default values
    let mut config = polkavm_linker::Config::default();
    config.set_strip(false);
    config.set_optimize(false);

    let res = polkavm_linker::program_from_elf(config, data)?;
    Ok(res)
}

pub type PolkaError = polkavm::Error;
pub type LinkerResult<T> = Result<T, PolkaError>;

pub type MoveProgramLinker = Linker<Runtime, ProgramError>;

/// creates new polkavm instance with native functions prepared for move program
/// all native functions declared by move std must defined here
pub fn new_move_program(
    output: &str,
    source: &str,
    mapping: Vec<String>,
) -> Result<(Instance<Runtime, ProgramError>, Runtime), anyhow::Error> {
    create_instance(create_blob(output, source, mapping)?)
}

/// Load a Move program from source and create a PolkaVM blob.
pub fn create_blob(
    output: &str,
    source: &str,
    mut mapping: Vec<String>,
) -> Result<ProgramBlob, anyhow::Error> {
    let mut build_options = BuildOptions::new(output);
    build_options = build_options.source(source);
    let path = std::path::Path::new(source);
    let mut dep_sources = vec![];
    if !path.is_dir() {
        return Err(anyhow::anyhow!(
            "Source must be a directory containing Move.toml: {source}"
        ));
    }
    let toml = SourcePackageLayout::try_find_root(path)?;
    let manifest = manifest_parser::parse_move_manifest_from_file(&toml)
        .map_err(|e| anyhow::anyhow!("Failed to parse Move manifest: {e}"))?;
    manifest
        .dependencies
        .values()
        .chain(manifest.dev_dependencies.values())
        .for_each(|dep| {
            if let Some(git_url) = dep.git_info.as_ref().map(|g| g.git_url.as_str()) {
                fetch_git_dep(&mut mapping, &mut dep_sources, dep, git_url)
                    .expect("Failed to fetch git dependency");
            } else {
                let local_path = path.join(Path::new(&dep.local));
                if local_path.exists() && local_path.is_dir() {
                    // check if the directory contains Move.toml
                    let _toml = SourcePackageLayout::try_find_root(&local_path)
                        .expect("Failed to find Move.toml in dependency");
                    if let Some(dep_mapping) = dep.subst.as_ref() {
                        for (name, subst) in dep_mapping {
                            if let SubstOrRename::Assign(ref addr) = subst {
                                let mapping_str = format!("{}={}", name, addr.to_standard_string());
                                mapping.push(mapping_str);
                            }
                        }
                    }
                    dep_sources.push(local_path.to_string_lossy().to_string());
                }
            }
        });
    if let Some(addresses) = &manifest.addresses {
        for (name, addr) in addresses.iter() {
            if let Some(addr) = addr {
                let mapping_str = format!("{}={}", name.as_str(), addr.to_standard_string());
                mapping.push(mapping_str);
            }
        }
    }
    for source in dep_sources {
        build_options = build_options.dependency(&source);
    }
    for m in mapping {
        build_options = build_options.address_mapping(m);
    }
    debug!("Build options: {build_options:?}");
    let program_bytes = build_polka_from_move(build_options)?;
    let blob = parse_to_blob(&program_bytes)?;
    Ok(blob)
}

fn fetch_git_dep(
    mapping: &mut Vec<String>,
    dep_sources: &mut Vec<String>,
    dep: &move_package::source_package::parsed_manifest::Dependency,
    git_url: &str,
) -> Result<(), anyhow::Error> {
    let path = Path::new("/tmp/move-deps");
    create_dir_all(path).expect("Failed to create temporary directory for dependencies");
    match gix::open(path) {
        Ok(repo) => {
            let remote = repo
                .find_default_remote(Direction::Fetch)
                .expect("Failed to find default remote")?;

            remote
                .connect(Direction::Fetch)?
                .prepare_fetch(Discard, gix::remote::ref_map::Options::default())?
                .with_shallow(Shallow::DepthAtRemote(NonZero::new(1).unwrap()))
                .receive(Discard, &AtomicBool::new(false))?;
        }
        Err(_) => {
            let mut prep = gix::prepare_clone(git_url, path)
                .expect("Failed to prepare clone")
                .with_shallow(Shallow::DepthAtRemote(NonZero::new(1).unwrap()));

            let (mut checkout, _) = prep.fetch_then_checkout(Discard, &AtomicBool::new(false))?;
            let (_, _) = checkout.main_worktree(Discard, &AtomicBool::new(false))?;
        }
    };
    let git_info = dep.git_info.as_ref().unwrap();
    let source = format!("/tmp/move-deps/{}/sources", git_info.subdir.display());
    dep_sources.push(source);
    if let Some(dep_mapping) = dep.subst.as_ref() {
        for (name, subst) in dep_mapping {
            if let SubstOrRename::Assign(ref addr) = subst {
                let mapping_str = format!("{}={}", name, addr.to_standard_string());
                mapping.push(mapping_str);
            }
        }
    }
    Ok(())
}

/// Creates a new PolkaVM instance with the Move program blob.
pub fn create_instance(
    blob: ProgramBlob,
) -> Result<(Instance<Runtime, ProgramError>, Runtime), anyhow::Error> {
    // AUX segment is used to inject data into the guest. The guest allocates on the heap
    // using the LeakingAllocator.
    const AUX_DATA_SIZE: u32 = 4 * 1024;
    let config = Config::from_env()?;

    let mut module_config = ModuleConfig::new();
    // enforce module loading fail if not all host functions are provided
    module_config.set_strict(true);
    module_config.set_aux_data_size(AUX_DATA_SIZE);

    let engine = Engine::new(&config)?;
    let module = Module::from_blob(&engine, &module_config, blob.clone())?;
    // Create a memory allocator for the module.
    let allocator = MemAllocator::init(module.memory_map());
    let storage = polkavm_move_native::storage::GlobalStorage::default();
    let runtime = Runtime {
        allocator,
        storage: Box::new(storage),
    };
    let mut linker: MoveProgramLinker = Linker::new();

    // Define the host functions that will be used by the Move program.
    // Note: when using the low-level `run_lowlevel` function, these are not called automatically,
    // but the program loop must handle the `Ecalli` interrupts and call these functions manually
    // setting up the parameters in the registers.
    linker.define_typed("hex_dump", |caller: Caller<Runtime>| {
        let instance = caller.instance;
        hexdump(instance);
    })?;

    linker.define_typed(
        "debug_print",
        |caller: Caller<Runtime>, ptr_to_type: u32, ptr_to_data: u32| {
            let instance = caller.instance;
            debug_print(instance, ptr_to_type, ptr_to_data)
        },
    )?;

    const SELECTOR: &[u8] = &hex_literal::hex!("c429b279");
    linker.define_typed("call_data_size", || SELECTOR.len() as u64)?;

    linker.define_typed("call_selector", || {})?;

    linker.define_typed(
        "call_data_copy",
        |caller: Caller<Runtime>, ptr_to_buf: u32, _size: u32, _offset: u32| {
            let instance = caller.instance;
            instance.write_memory(ptr_to_buf, SELECTOR)?;
            Result::<(), ProgramError>::Ok(())
        },
    )?;

    const ORIGIN_ADDR: &[u8] = &hex_literal::hex!("ab010101010101010101010101010101010101ce");
    const ACCOUNT_ID: &[u8] =
        &hex_literal::hex!("ab010101010101010101010101010101010101010101010101010101010101ce");

    linker.define_typed("origin", |caller: Caller<Runtime>, ptr_to_buf: u32| {
        let instance = caller.instance;
        instance.write_memory(ptr_to_buf, ORIGIN_ADDR)?;
        Result::<(), ProgramError>::Ok(())
    })?;

    linker.define_typed(
        "to_account_id",
        |caller: Caller<Runtime>, _ptr_to_addr: u32, ptr_to_account: u32| {
            let instance = caller.instance;
            instance.write_memory(ptr_to_account, ACCOUNT_ID)?;
            Result::<(), ProgramError>::Ok(())
        },
    )?;

    linker.define_typed(
        "move_to",
        |caller: Caller<Runtime>, ptr_to_signer: u32, ptr_to_struct: u32, ptr_to_tag: u32| {
            let runtime = caller.user_data;
            let instance = caller.instance;
            move_to(runtime, instance, ptr_to_signer, ptr_to_struct, ptr_to_tag)
        },
    )?;

    linker.define_typed(
        "move_from",
        |caller: Caller<Runtime>, ptr_to_addr: u32, remove: u32, ptr_to_tag: u32, is_mut: u32| {
            let instance = caller.instance;
            let runtime = caller.user_data;
            move_from(runtime, instance, ptr_to_addr, remove, ptr_to_tag, is_mut)
        },
    )?;

    linker.define_typed(
        "exists",
        |caller: Caller<Runtime>, ptr_to_addr: u32, ptr_to_tag: u32| {
            let runtime = caller.user_data;
            let instance = caller.instance;
            exists(runtime, instance, ptr_to_addr, ptr_to_tag)
        },
    )?;

    linker.define_typed(
        "release",
        |caller: Caller<Runtime>, ptr_to_addr: u32, ptr_to_struct: u32, ptr_to_tag: u32| {
            let runtime = caller.user_data;
            let instance = caller.instance;
            release(runtime, instance, ptr_to_addr, ptr_to_struct, ptr_to_tag)
        },
    )?;

    linker.define_typed(
        "terminate",
        |caller: Caller<Runtime>, ptr_to_beneficiary: u32| {
            let instance = caller.instance;
            let beneficiary = copy_bytes_from_guest(instance, ptr_to_beneficiary, 20)
                .expect("Failed to copy beneficiary address from guest");
            guest_abort(instance, beneficiary[0] as u64)
        },
    )?;

    linker.define_typed(
        "hash_sha2_256",
        |caller: Caller<Runtime>, ptr_to_buf: u32| {
            let instance = caller.instance;
            hash_sha2_256(caller.user_data, instance, ptr_to_buf)
        },
    )?;

    linker.define_typed(
        "hash_sha3_256",
        |caller: Caller<Runtime>, ptr_to_buf: u32| {
            debug!("hash_sha3_256 called with type: ptr: 0x{ptr_to_buf:X}");
            let instance = caller.instance;
            hash_sha3_256(caller.user_data, instance, ptr_to_buf)
        },
    )?;

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module)?;

    // Instantiate the module.
    let mut instance = instance_pre.instantiate()?;
    // zero aux data
    instance.zero_memory(
        module.memory_map().aux_data_address(),
        module.memory_map().aux_data_size(),
    )?;
    debug!(
        "Module loaded with RW data size: {}, RO data size: {}, aux data size: {}, heap start: {:x?}",
        module.memory_map().rw_data_size(),
        module.memory_map().ro_data_size(),
        module.memory_map().aux_data_size(),
        module.memory_map().heap_base(),
    );
    Ok((instance, runtime))
}

/// Copy memory host -> guest (aux)
pub fn copy_to_guest<T: Sized + Copy>(
    instance: &mut RawInstance,
    allocator: &mut MemAllocator,
    value: &T,
) -> Result<u32, MemoryAccessError> {
    trace!(
        "Copying value of type {} to guest memory",
        core::any::type_name::<T>()
    );
    let size_to_write = core::mem::size_of::<T>();
    let address = allocator.alloc(size_to_write, core::mem::align_of::<T>())?;

    // safety: we know we have memory, we just checked
    let slice =
        unsafe { core::slice::from_raw_parts((value as *const T) as *const u8, size_to_write) };

    instance.write_memory(address, slice)?;

    Ok(address)
}

/// Copy a byte slice (host -> guest aux memory)
pub fn copy_bytes_to_guest(
    instance: &mut RawInstance,
    allocator: &mut MemAllocator,
    bytes: &[u8],
) -> Result<u32, MemoryAccessError> {
    let size = bytes.len();
    let align = core::mem::align_of::<u8>(); // usually 1, but explicit for clarity

    trace!("Copying {size} bytes to guest memory with alignment {align}");

    let address = allocator.alloc(size, align)?;

    instance.write_memory(address, bytes)?;

    Ok(address)
}

/// Copy memory guest (aux) -> host
pub fn copy_from_guest<T: Sized + Copy>(
    instance: &mut RawInstance,
    address: u32,
) -> Result<T, MemoryAccessError> {
    trace!(
        "Copying value of type {} from guest memory at address 0x{:X}",
        core::any::type_name::<T>(),
        address
    );
    let mut uninit = MaybeUninit::<T>::uninit();
    unsafe {
        let dst_bytes: &mut [u8] =
            core::slice::from_raw_parts_mut(uninit.as_mut_ptr() as *mut u8, size_of::<T>());
        trace!(
            "Reading {} bytes from guest memory at address 0x{:X}",
            size_of::<T>(),
            address
        );
        instance.read_memory_into(address, dst_bytes)?;
        trace!("read:: {dst_bytes:x?}");
        Ok(uninit.assume_init())
    }
}

/// Copy memory guest (aux) -> host into a Vec<u8>
pub fn copy_bytes_from_guest(
    instance: &mut RawInstance,
    address: u32,
    length: usize,
) -> Result<std::vec::Vec<u8>, MemoryAccessError> {
    trace!("Copying {length} bytes from guest memory at address 0x{address:X}");
    let mut uninit: std::boxed::Box<[MaybeUninit<u8>]> = std::boxed::Box::new_uninit_slice(length);

    // Step 2: let `read_memory_into` initialize it
    let initialized: &mut [u8] = instance.read_memory_into(address, &mut *uninit)?;
    trace!("read: {initialized:x?}");
    // Step 3: create a Vec<u8> from the slice
    Ok(initialized.to_vec())
}

/// Different way to run the program, which allows to handle low-level interrupts
/// The caller must store the parameters to the entrypoint function into registers before calling this function.
pub fn run_lowlevel(
    instance: &mut Instance<Runtime, ProgramError>,
    runtime: &mut Runtime,
    entry: &str,
) -> Result<(), anyhow::Error> {
    let start = instance
        .module()
        .exports()
        .find(|export| export.symbol() == entry)
        .expect("'pvm_start' export not found")
        .program_counter();
    let module = instance.module();
    let imports = module.imports().iter().collect::<Vec<_>>();
    // cache imports with their indices
    const ALLOWED_IMPORTS: &[&[u8]] = &[
        b"debug_print",
        b"hex_dump",
        b"terminate",
        b"move_to",
        b"move_from",
        b"exists",
        b"release",
        b"hash_sha2_256",
        b"hash_sha3_256",
    ];
    let map: HashMap<usize, &'static str> = imports
        .into_iter()
        .enumerate()
        .filter_map(|(i, import)| {
            let import = import?;
            ALLOWED_IMPORTS
                .iter()
                .find(|&&allowed| allowed == import.as_bytes())
                .map(|&name| (i, std::str::from_utf8(name).unwrap())) // safe to unwrap since we control the names
        })
        .collect();

    // set the initial program counter and stack pointer
    let sp = module.default_sp();
    instance.set_next_program_counter(start);
    instance.set_reg(Reg::RA, polkavm::RETURN_TO_HOST);
    instance.set_reg(Reg::SP, sp);
    // run the program loop. We must handle the interrupts manually.
    loop {
        match instance.run()? {
            InterruptKind::Finished => {
                info!("Program finished successfully.");
                runtime.storage.release_all();
                break;
            }
            InterruptKind::Ecalli(n) => {
                let syscall = map.get(&(n as usize)).unwrap_or(&"unknown syscall");
                debug!("Ecalli interrupt with code: {n}: {syscall}");
                handle_ecalli(instance, runtime, syscall);
                if syscall == &"abort" {
                    let code = instance.reg(Reg::A0);
                    panic!("Aborted: {code}");
                }
            }
            InterruptKind::Segfault(segfault) => {
                runtime.storage.release_all();
                panic!("Segfault occurred at address {:x?}", segfault.page_address);
            }
            InterruptKind::Trap => {
                info!("Trap occurred, releasing all resources.");
                runtime.storage.release_all();
                panic!("Trap");
            }
            InterruptKind::NotEnoughGas => {
                warn!("Not enough gas to continue execution, releasing all resources.");
                runtime.storage.release_all();
                panic!("Not enough gas to continue execution");
            }
            other => {
                warn!("Program interrupted: {other:?}");
                break;
            }
        }
    }
    Ok(())
}

fn handle_ecalli(
    instance: &mut polkavm::Instance<Runtime, ProgramError>,
    runtime: &mut Runtime,
    syscall: &str,
) {
    match syscall {
        "debug_print" => {
            let ptr_to_type = instance.reg(Reg::A0) as u32;
            let ptr_to_data = instance.reg(Reg::A1) as u32;
            debug_print(instance, ptr_to_type, ptr_to_data).expect("Failed to print debug info");
        }
        "hex_dump" => {
            hexdump(instance);
        }
        "move_to" => {
            let ptr_to_signer = instance.reg(Reg::A0) as u32;
            let ptr_to_struct = instance.reg(Reg::A1) as u32;
            let ptr_to_tag = instance.reg(Reg::A2) as u32;
            move_to(runtime, instance, ptr_to_signer, ptr_to_struct, ptr_to_tag)
                .expect("Failed to print debug info");
        }
        "move_from" => {
            let ptr_to_signer = instance.reg(Reg::A0) as u32;
            let remove = instance.reg(Reg::A1) as u32;
            let ptr_to_tag = instance.reg(Reg::A2) as u32;
            let is_mut = instance.reg(Reg::A3) as u32;
            let result = move_from(runtime, instance, ptr_to_signer, remove, ptr_to_tag, is_mut)
                .expect("Failed to move from global storage");
            instance.set_reg(Reg::A0, result as u64);
        }
        "exists" => {
            let ptr_to_signer = instance.reg(Reg::A0) as u32;
            let ptr_to_tag = instance.reg(Reg::A1) as u32;
            let result = exists(runtime, instance, ptr_to_signer, ptr_to_tag)
                .expect("Failed to check if global exists");
            instance.set_reg(Reg::A0, result as u64);
        }
        "hash_sha2_256" => {
            let ptr_to_vec = instance.reg(Reg::A0) as u32;
            let result =
                hash_sha2_256(runtime, instance, ptr_to_vec).expect("Failed to calculate hash");
            instance.set_reg(Reg::A0, result as u64);
        }
        "hash_sha3_256" => {
            let ptr_to_vec = instance.reg(Reg::A0) as u32;
            let result =
                hash_sha3_256(runtime, instance, ptr_to_vec).expect("Failed calculate hash");
            instance.set_reg(Reg::A0, result as u64);
        }
        "terminate" => {
            let code = instance.reg(Reg::A0);
            guest_abort(instance, code).ok();
        }
        _ => {}
    }
}

fn hash_sha2_256(
    runtime: &mut Runtime,
    instance: &mut RawInstance,
    ptr_to_buf: u32,
) -> Result<u32, ProgramError> {
    let bytes = from_move_byte_vector(instance, ptr_to_buf)?;
    debug!("hash_sha2_256 called with type: ptr: 0x{ptr_to_buf:X}");
    debug!("bytes: {bytes:?}");
    let digest = sha2::Sha256::digest(&bytes);
    debug!(
        "hash_sha2_256 called with {} bytes, digest: {digest:X?}",
        bytes.len(),
    );
    let address = to_move_byte_vector(instance, &mut runtime.allocator, digest.to_vec())?;
    debug!("Allocated address for digest: 0x{address:X}");
    Result::<u32, ProgramError>::Ok(address)
}

fn hash_sha3_256(
    runtime: &mut Runtime,
    instance: &mut RawInstance,
    ptr_to_buf: u32,
) -> Result<u32, ProgramError> {
    let bytes = from_move_byte_vector(instance, ptr_to_buf)?;
    debug!("bytes: {bytes:?}");
    let digest = sha3::Sha3_256::digest(&bytes);
    debug!(
        "hash_sha3_256 called with {} bytes, digest: {digest:X?}",
        bytes.len(),
    );
    let address = to_move_byte_vector(instance, &mut runtime.allocator, digest.to_vec())?;
    debug!("Allocated address for digest: 0x{address:X}");
    Result::<u32, ProgramError>::Ok(address)
}

fn guest_abort(instance: &mut RawInstance, code: u64) -> Result<(), ProgramError> {
    hexdump(instance);
    let program_error = match code {
        PANIC_CODE => ProgramError::NativeLibPanic,
        ALLOC_CODE => ProgramError::NativeLibAllocatorCall,
        _ => ProgramError::Abort(code),
    };
    Result::<(), _>::Err(program_error)
}

fn release(
    runtime: &mut Runtime,
    instance: &mut RawInstance,
    ptr_to_addr: u32,
    ptr_to_struct: u32,
    ptr_to_tag: u32,
) -> Result<(), ProgramError> {
    debug!(
        "release called with address ptr: 0x{ptr_to_addr:X}, ptr_to_tag: 0x{ptr_to_tag:X}, value ptr: 0x{ptr_to_struct:X}",
    );
    let address: MoveAddress =
        copy_from_guest(instance, ptr_to_addr).expect("Failed to copy address from guest");
    let tag: [u8; 32] = copy_from_guest(instance, ptr_to_tag).unwrap_or([0; 32]);
    let value = from_move_byte_vector(instance, ptr_to_struct).unwrap_or_default();
    debug!("release called with address: {address:?}, tag: {tag:?}, value: {value:x?}",);
    runtime.storage.update(address, tag, value)?;
    runtime.storage.release(address, tag);
    Result::<(), ProgramError>::Ok(())
}

fn exists(
    runtime: &mut Runtime,
    instance: &mut RawInstance,
    ptr_to_addr: u32,
    ptr_to_tag: u32,
) -> Result<u32, ProgramError> {
    debug!("exists called with address ptr: 0x{ptr_to_addr:X}, ptr_to_tag: 0x{ptr_to_tag:X}",);
    let address: MoveAddress = copy_from_guest(instance, ptr_to_addr)?;
    let tag: [u8; 32] = copy_from_guest(instance, ptr_to_tag)?;
    debug!("exists called with address: {address:?}, tag: {tag:?}",);
    let value = runtime.storage.exists(address, tag)?;
    Result::<u32, ProgramError>::Ok(value as u32)
}

fn move_from(
    runtime: &mut Runtime,
    instance: &mut RawInstance,
    ptr_to_addr: u32,
    remove_u32: u32,
    ptr_to_tag: u32,
    is_mut_u32: u32,
) -> Result<u32, ProgramError> {
    debug!(
        "move_from called with address ptr: 0x{ptr_to_addr:X}, remove: {remove_u32}, is_mut: {is_mut_u32}",
    );
    let remove = remove_u32 != 0;
    let is_mut = is_mut_u32 != 0;
    let address: MoveAddress = copy_from_guest(instance, ptr_to_addr)?;
    let tag: [u8; 32] = copy_from_guest(instance, ptr_to_tag)?;
    debug!("move_from called with address ptr: 0x{ptr_to_addr:X}, address: {address:?}",);
    let value = runtime.storage.load(address, tag, remove, is_mut)?;
    debug!("move_from loaded value: {value:x?}");
    let address = to_move_byte_vector(instance, &mut runtime.allocator, value.to_vec())?;
    debug!("move_from returned address: 0x{address:X}");
    Result::<u32, ProgramError>::Ok(address)
}

fn move_to(
    runtime: &mut Runtime,
    instance: &mut RawInstance,
    ptr_to_signer: u32,
    ptr_to_struct: u32,
    ptr_to_tag: u32,
) -> Result<(), ProgramError> {
    debug!("move_to called with address ptr: 0x{ptr_to_signer:X}, value ptr: 0x{ptr_to_struct:X}");
    let signer_ptr: u32 = copy_from_guest(instance, ptr_to_signer)?;
    let signer: MoveSigner = copy_from_guest(instance, signer_ptr)?;
    let address = signer.0;
    let tag: [u8; 32] = copy_from_guest(instance, ptr_to_tag)?;
    let value = from_move_byte_vector(instance, ptr_to_struct)?;
    debug!(
        "move_to called with address ptr: 0x{ptr_to_signer:X}, value ptr: 0x{ptr_to_struct:X}, address: {address:?}, value: {value:x?}",
    );
    runtime.storage.store(address, tag, value.to_vec())?;
    Result::<(), ProgramError>::Ok(())
}

fn debug_print(
    instance: &mut RawInstance,
    ptr_to_type: u32,
    ptr_to_data: u32,
) -> Result<(), ProgramError> {
    let mut move_type_string = "Unknown".to_string();
    let move_type: Result<MoveType, MemoryAccessError> = copy_from_guest(instance, ptr_to_type);
    // for some reason, the type is stored in RO memory, which we can't read when dynamic paging is enabled
    if let Ok(move_type) = move_type {
        move_type_string = move_type.to_string();
        match move_type.type_desc {
            TypeDesc::Bool | TypeDesc::U8 => {
                let move_value: u8 = copy_from_guest(instance, ptr_to_data)?;
                debug!("debug_print called. type ptr: 0x{ptr_to_type:X} Data ptr: 0x{ptr_to_data:X}, type: {move_type_string:?}, value: 0x{move_value}");
            }
            TypeDesc::U16 | TypeDesc::U32 => {
                let move_value: u32 = copy_from_guest(instance, ptr_to_data)?;
                debug!("debug_print called. type ptr: 0x{ptr_to_type:X} Data ptr: 0x{ptr_to_data:X}, type: {move_type_string:?}, value: 0x{move_value:x?}");
            }
            TypeDesc::Signer => {
                let move_signer: MoveSigner = copy_from_guest(instance, ptr_to_data)?;
                debug!("debug_print called. type ptr: 0x{ptr_to_type:X} Data ptr: 0x{ptr_to_data:X}, type: {move_type_string:?}, value: {move_signer:?}");
            }
            TypeDesc::U64 => {
                let move_value: u64 = copy_from_guest(instance, ptr_to_data)?;
                debug!("debug_print called. type ptr: 0x{ptr_to_type:X} Data ptr: 0x{ptr_to_data:X}, type: {move_type_string:?}, value: 0x{move_value:x?}");
            }
            TypeDesc::Vector => {
                let vec: MoveByteVector = copy_from_guest(instance, ptr_to_data)?;
                let len = vec.length as usize;
                let bytes = copy_bytes_from_guest(instance, vec.ptr as u32, len)?;
                let s = String::from_utf8(bytes.clone());
                if let Ok(s) = s {
                    debug!("debug_print called: {s}");
                } else {
                    debug!("debug_print called. type ptr: 0x{ptr_to_type:X} Data ptr: 0x{ptr_to_data:X}, type: {move_type_string:?}, value: {vec:?}, bytes: {bytes:x?}");
                }
            }
            _ => {
                let move_value: u64 = copy_from_guest(instance, ptr_to_data)?;
                debug!("debug_print called. type ptr: 0x{ptr_to_type:X} Data ptr: 0x{ptr_to_data:X}, type: {move_type_string:?}, value: 0x{move_value:x}");
            }
        }
    } else {
        let move_value: u32 = copy_from_guest(instance, ptr_to_data)?;
        debug!("debug_print called. type ptr: 0x{ptr_to_type:X} Data ptr: 0x{ptr_to_data:X}, type: {move_type_string:?}, value: {move_value}");
    }
    Result::<(), ProgramError>::Ok(())
}

fn from_move_byte_vector(
    instance: &mut RawInstance,
    ptr_to_buf: u32,
) -> Result<Vec<u8>, ProgramError> {
    let move_byte_vec: MoveByteVector = copy_from_guest(instance, ptr_to_buf)?;
    debug!("move_byte_vec: {move_byte_vec:?}");
    let len = move_byte_vec.length as usize;
    let bytes = copy_bytes_from_guest(instance, move_byte_vec.ptr as u32, len)?;
    Ok(bytes)
}

fn to_move_byte_vector(
    instance: &mut RawInstance,
    allocator: &mut MemAllocator,
    bytes: Vec<u8>,
) -> Result<u32, ProgramError> {
    let len = bytes.len();
    let data_ptr = copy_bytes_to_guest(instance, allocator, bytes.as_slice())?;
    debug!("Data copied to guest memory at address: 0x{data_ptr:X}, length: {len}",);
    let move_byte_vec = MoveByteVector {
        ptr: data_ptr as *mut u8,
        capacity: len as u64,
        length: len as u64,
    };
    debug!("move_byte_vec: {move_byte_vec:?}");
    Ok(copy_to_guest(instance, allocator, &move_byte_vec)?)
}

fn hexdump(instance: &mut RawInstance) {
    let ro_base = 0x10000u32;
    let ro = instance
        .read_memory(ro_base, 256)
        .unwrap_or_else(|_| vec![]);
    print_mem(ro, ro_base as usize, " RO  ");
    let stack_base = 0xfffcf940;
    let stack_end = 0xfffd0000;
    println!(
        "Stack base: 0x{stack_base:X}, Stack end: 0x{stack_end:X}: len: {}",
        stack_end - stack_base
    );
    let stack = instance
        .read_memory(stack_base, stack_end - stack_base)
        .unwrap_or_else(|_| vec![]);
    print_mem(stack, stack_base as usize, " STACK ");
    let heap = instance
        .read_memory(HEAP_BASE, 256)
        .unwrap_or_else(|_| vec![]);
    print_mem(heap, HEAP_BASE as usize, " HEAP ");
    let address = instance.module().memory_map().aux_data_address();
    let length = 100;
    let aux = instance
        .read_memory(address, length)
        .unwrap_or_else(|_| vec![]);
    print_mem(aux, address as usize, " AUX ");
}

fn print_mem(mem: Vec<u8>, base: usize, label: &str) {
    let start_address = 0usize;
    let mut offset = 0;

    println!("{label:-^78}");
    while offset < mem.len() {
        // Print the address
        print!("{:08x}  ", base + start_address + offset);

        // Print hex values
        for i in 0..16 {
            if offset + i < mem.len() {
                print!("{:02x} ", mem[offset + i]);
            } else {
                print!("   ");
            }
            if i == 7 {
                print!(" "); // extra space between 8-byte halves
            }
        }

        print!(" |");

        // Print ASCII representation
        for i in 0..16 {
            if offset + i < mem.len() {
                let byte = mem[offset + i];
                let ch = if byte.is_ascii_graphic() || byte == b' ' {
                    byte as char
                } else {
                    '.'
                };
                print!("{ch}");
            } else {
                print!(" ");
            }
        }

        println!("|");
        offset += 16;
    }
    println!("{:-<78}", "");
}
