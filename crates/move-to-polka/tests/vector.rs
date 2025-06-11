use move_to_polka::{initialize_logger, linker::new_move_program};
use serial_test::serial;

#[test]
#[serial]
pub fn test_vector_new() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u64, ()>(&mut allocator, "vecnew", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 2);

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_isempty() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<i32, ()>(&mut allocator, "vecisempty", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 0);

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_cmp() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    let result = instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "veccmp", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    assert_eq!(result, 1);

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_singleton() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "singleton", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_reverse() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "reverse", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_contains() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "contains", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_swapremove() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "swapremove", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_remove() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "remove", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_indexof() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "indexof", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_foreach() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "foreach", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_foreachref() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "foreachref", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_fold() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "fold", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_map() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "map", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}

#[test]
#[serial]
pub fn test_vector_filter() -> anyhow::Result<()> {
    initialize_logger();
    let (mut instance, mut allocator) = new_move_program(
        "output/vector.polkavm",
        "../../examples/basic/sources/vector.move",
        vec![],
    )?;
    instance
        .call_typed_and_get_result::<u32, ()>(&mut allocator, "filter", ())
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    Ok(())
}
