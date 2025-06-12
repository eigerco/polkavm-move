use move_to_polka::{initialize_logger, linker::new_move_program};

#[test]
pub fn test_multiple_functions() -> anyhow::Result<()> {
    initialize_logger();

    let (mut instance, mut allocator) = new_move_program(
        "output/multiple_functions.polkavm",
        "../../examples/basic/sources/multiple_functions.move",
        vec![],
    )?;
    let res: u64 = instance
        .call_typed_and_get_result(&mut allocator, "sum", (5u64, 6u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 11);

    let res: u64 = instance
        .call_typed_and_get_result(&mut allocator, "sum_plus_const_5", (5u64, 10u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 20);

    let res: u64 = instance
        .call_typed_and_get_result(&mut allocator, "sum_of_3", (1u64, 2u64, 3u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 6);

    let res: u64 = instance
        .call_typed_and_get_result(&mut allocator, "sum_plus_const_5", (5u64, 10u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 20);

    let res: u64 = instance
        .call_typed_and_get_result(&mut allocator, "sum_for_rich", (6u64, 7u64, 8u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 121);

    let res: u32 = instance
        .call_typed_and_get_result(
            &mut allocator,
            "sum_different_size_args",
            (1u32, 2u64, 3u32),
        )
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 6);

    let no_extras: u32 = 5; // polkaVM typed args don't support bool - any value bigger than 0 is true
    let res: u64 = instance
        .call_typed_and_get_result(&mut allocator, "sum_if_extras", (1u32, no_extras, 10u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 11);

    Ok(())
}
