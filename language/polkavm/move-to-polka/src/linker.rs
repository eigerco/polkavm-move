use crate::{options::Options, run_to_polka};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use log::{debug, info};
use polkavm::{
    Caller, Config, Engine, Instance, Linker, MemoryAccessError, Module, ModuleConfig, ProgramBlob,
};
use polkavm_move_native::{
    host::{copy_from_guest, MemAllocator, ProgramError},
    types::MoveType,
    ALLOC_CODE, PANIC_CODE,
};

pub const MOVE_STDLIB_PATH: &str = env!("MOVE_STDLIB_PATH");

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

    pub fn address_mapping(mut self, mapping: &str) -> Self {
        self.options.named_address_mapping.push(mapping.to_string());
        self
    }

    pub fn dependency(mut self, dependency_path: &str) -> Self {
        self.options.dependencies.push(dependency_path.to_string());
        self
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
    config.set_optimize(true);

    let res = polkavm_linker::program_from_elf(config, data)?;
    Ok(res)
}

pub type PolkaError = polkavm::Error;
pub type LinkerResult<T> = Result<T, PolkaError>;

pub type MoveProgramLinker = Linker<MemAllocator, ProgramError>;

/// creates new polkavm instance with native functions prepared for move program
/// all native functions declared by move std must defined here
pub fn new_move_program(
    build_options: BuildOptions,
) -> Result<(Instance<MemAllocator, ProgramError>, MemAllocator), anyhow::Error> {
    const AUX_DATA_SIZE: u32 = 4 * 1024;

    let program_bytes = build_polka_from_move(build_options)?;
    let blob = parse_to_blob(&program_bytes)?;

    let mut config = Config::from_env()?;
    config.set_allow_dynamic_paging(true);
    let engine = Engine::new(&config)?;

    let mut module_config = ModuleConfig::new();
    module_config.set_strict(true); // enforce module loading fail if not all host functions are provided
    module_config.set_aux_data_size(AUX_DATA_SIZE);
    module_config.set_dynamic_paging(true);

    let module = Module::from_blob(&engine, &module_config, blob)?;
    // Create a memory allocator for the module.
    let allocator = MemAllocator::init(module.memory_map());
    let memory_map = module.memory_map();
    info!(
        "RO: {:X} size {}",
        memory_map.ro_data_address(),
        memory_map.ro_data_size()
    );

    info!(
        "AUX: {:X} size: {}",
        memory_map.aux_data_address(),
        memory_map.aux_data_size()
    );
    let mut linker: MoveProgramLinker = Linker::new();

    // additional "native" function used by move program and also exposed by host
    // it is just for testing/debuging only
    linker.define_typed(
        "debug_print",
        |caller: Caller<MemAllocator>, ptr_to_type: u32, ptr_to_data: u32| {
            let mut move_type_string = "Unknown".to_string();
            let move_type: Result<MoveType, MemoryAccessError> = copy_from_guest(caller.instance, ptr_to_type);
            // for some reason, the type is stored in RO memory, which we can't read when dynamic paging is enabled
            if let Ok(move_type) = move_type {
                move_type_string = move_type.to_string();
            }
            let move_value: u64 = copy_from_guest(caller.instance, ptr_to_data)?;
            info!("debug_print called. type ptr: 0x{ptr_to_type:x} Data ptr: 0x{ptr_to_data:x}, type: {move_type_string:?}, value: {move_value}");
            Result::<(), ProgramError>::Ok(())
        },
    )?;

    linker.define_typed("abort", |code: u64| {
        let program_error = match code {
            PANIC_CODE => ProgramError::NativeLibPanic,
            ALLOC_CODE => ProgramError::NativeLibAllocatorCall,
            _ => ProgramError::Abort(code),
        };
        Result::<(), _>::Err(program_error)
    })?;

    linker.define_typed(
        "guest_alloc",
        |caller: Caller<MemAllocator>, size: u64, align: u64| {
            debug!("guest_alloc called with size: {size}, align: {align}");
            let allocator = caller.user_data;
            let address = allocator.alloc(size.try_into().unwrap(), align.try_into().unwrap());
            Result::<u32, ProgramError>::Ok(address.expect("Failed to allocate memory"))
        },
    )?;

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module)?;

    // Instantiate the module.
    let mut instance = instance_pre.instantiate()?;
    // Initialize the memory block for auxiliary data.
    instance.write_memory(
        memory_map.aux_data_address(),
        &[0u8; AUX_DATA_SIZE as usize],
    )?;
    Ok((instance, allocator))
}
