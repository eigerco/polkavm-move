use move_to_polka::{
    initialize_logger,
    linker::{create_blob, create_instance},
};
use once_cell::sync::OnceCell;
use polkavm::ProgramBlob;

static COMPILE_ONCE: OnceCell<ProgramBlob> = OnceCell::new();

fn create_blob_once() -> ProgramBlob {
    COMPILE_ONCE
        .get_or_init(|| {
            initialize_logger();
            create_blob(
                "output/entry/entry.polkavm",
                "../../examples/entry/",
                vec![],
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
pub fn entry() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut runtime) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut runtime, "main", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
