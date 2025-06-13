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
                "output/vector/vector.polkavm",
                "../../examples/vector/sources/vector.move",
                vec![],
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}

#[test]
pub fn test_vector_new() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "vecnew", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_isempty() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<i32, ()>(&mut allocator, "vecisempty", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_cmp() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "veccmp", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_singleton() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "singleton", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_popback() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "popback", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_reverse() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "reverse", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_contains() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "contains", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_swapremove() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "swapremove", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_remove() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "remove", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_indexof() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "indexof", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_foreach() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "foreach", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_foreachref() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "foreachref", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_fold() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "fold", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_map() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "map", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
pub fn test_vector_filter() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "filter", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
