use clap::{ArgGroup, Parser};
use move_to_polka::linker::{create_instance, new_move_program};
use polkavm::ProgramBlob;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
#[command(
    about = "CLI tool to compile Move source code to PolkaVM bytecode and execute a function",
    group(
        ArgGroup::new("input")
            .required(true)
            .args(["source", "module"])
    )
)]
struct Args {
    #[arg(short, long)]
    // path to Move source to compile and load
    pub source: Option<String>,
    #[arg(short, long)]
    // path to Move module to load
    pub module: Option<String>,
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

    let (mut instance, mut allocator) = if let Some(source) = args.source {
        let output = "/tmp/output.polkavm";
        info!("Compiled Move source to PolkaVM bytecode at {}", output);
        new_move_program(output, source.as_str(), vec![])?
    } else {
        let program_bytes = std::fs::read(args.module.unwrap())?; // clap guarantees that module is provided
        let blob =
            ProgramBlob::parse(program_bytes.into()).map_err(|e| anyhow::anyhow!("{e:?}"))?;
        create_instance(blob)?
    };
    let module = instance.module().clone();

    let entry_point_export = module
        .exports()
        .find(|export| export == args.entrypoint.as_str())
        .ok_or_else(|| anyhow::anyhow!("Module doesnt export {}", args.entrypoint))?;

    // now assuming all fuctions have args of u64, but thats not always true
    let reg_args = &args.params;
    let ep = entry_point_export.program_counter();
    info!(
        "Calling entry point {} at PC {} with args: {:?}",
        args.entrypoint, ep, reg_args
    );
    // assuming return value is u64. It's hard to handle with a dynamic CLI, when the function is generic
    let result = match reg_args.len() {
        0 => instance
            .call_typed_and_get_result::<u64, ()>(&mut allocator, ep, ())
            .map_err(|e| anyhow::anyhow!("{e:?}"))?,
        1 => {
            let (a,) = (reg_args[0],);
            instance
                .call_typed_and_get_result::<u64, (u64,)>(&mut allocator, ep, (a,))
                .map_err(|e| anyhow::anyhow!("{e:?}"))?
        }
        2 => {
            let (a, b) = (reg_args[0], reg_args[1]);
            instance
                .call_typed_and_get_result::<u64, (u64, u64)>(&mut allocator, ep, (a, b))
                .map_err(|e| anyhow::anyhow!("{e:?}"))?
        }
        // … repeat up to your max arity …
        _ => anyhow::bail!("too many arguments (max = 2)"),
    };

    info!("Result: {:?}", result);

    Ok(())
}
