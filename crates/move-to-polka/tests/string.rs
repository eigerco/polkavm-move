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
                "output/string/string.polkavm",
                "../../examples/string/sources/string.move",
                vec![],
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
pub fn test_string() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "foo", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_string_index_of() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "index_of", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_string_substring() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "substring", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_append() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "append", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_insert() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "insert", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
