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
                "output/returns.polkavm",
                "../../examples/basic/sources/returns.move",
                vec![],
            )
            .expect("Failed to compile Move source to PolkaVM bytecode")
        })
        .clone()
}
#[test]
pub fn test_rv_bool() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_bool_false", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 0);
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_bool_true", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 1);

    Ok(())
}

#[test]
pub fn test_rv_u16() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_u16", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    Ok(())
}

#[test]
pub fn test_rv_u32() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "rv_u32", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    Ok(())
}

#[test]
pub fn test_rv_u8() -> anyhow::Result<()> {
    let blob = create_blob_once();
    let (mut instance, mut allocator) = create_instance(blob)?;
    let result = instance
        .call_typed_and_get_result::<i32, ()>(&mut allocator, "rv_u8", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 19);

    Ok(())
}
