use move_to_polka::{initialize_logger, linker::new_move_program};
use serial_test::serial;

#[test]
#[serial]
pub fn test_string() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/string.polkavm",
        "../../examples/basic/sources/string.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "foo", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_string_index_of() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/string.polkavm",
        "../../examples/basic/sources/string.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "index_of", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_string_substring() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/string.polkavm",
        "../../examples/basic/sources/string.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<(), ()>(&mut allocator, "substring", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_serialize_string() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/serialize.polkavm",
        "../../examples/basic/sources/serialize.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "ser_string", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_append() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/string.polkavm",
        "../../examples/basic/sources/string.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "append", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_insert() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/string.polkavm",
        "../../examples/basic/sources/string.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "insert", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
