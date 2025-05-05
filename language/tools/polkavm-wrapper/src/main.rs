use clap::Parser;
use polkavm::{Config, Engine, InterruptKind, Module, ProgramBlob, RegValue};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    #[arg(short, long)]
    // path to polka linked module to load into VM
    pub module: String,
    #[arg(short, long)]
    // entry point function name to call
    pub entrypoint: String,
    #[arg(short, long, value_delimiter = ' ', num_args = 0..)]
    // parameters to pass to function - only u64 args are supported
    pub params: Vec<u64>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    info!("Reading {} module", args.module);
    let program_bytes = std::fs::read(&args.module)?;
    let program_blob =
        ProgramBlob::parse(program_bytes.into()).map_err(|e| anyhow::anyhow!("{e:?}"))?;

    let config = Config::from_env()?;
    let engine = Engine::new(&config)?;
    let module = Module::from_blob(&engine, &Default::default(), program_blob)?;
    info!("64bit module?: {}", module.is_64_bit());

    let entry_point_export = module
        .exports()
        .find(|export| export == args.entrypoint.as_str())
        .ok_or_else(|| anyhow::anyhow!("Module doesnt export {}", args.entrypoint))?;

    let mut instance = module.instantiate()?;

    info!("Calling {} with args {:?}", args.entrypoint, args.params);

    // now assuming all fuctions have args of u64, but thats not always true
    // need a way to dynamically detect (or indicate through args?) u32 values too
    // also assumes that module is 32bits, and u64 values are passed through 2 regs,
    let reg_args: Vec<RegValue> = args
        .params
        .into_iter()
        .flat_map(|arg| [arg, arg >> 32])
        .collect();

    instance.prepare_call_untyped(entry_point_export.program_counter(), &reg_args);

    let interrupt_kind = instance.run()?;
    match interrupt_kind {
        InterruptKind::Finished => info!("VM finished"),
        InterruptKind::Ecalli(num) => anyhow::bail!("unexpected external call: {num}"),
        _ => anyhow::bail!("unexpected interruption: {interrupt_kind:?}"),
    }
    let res: u64 = instance.get_result_typed();
    info!("Result = {}", res);

    Ok(())
}
