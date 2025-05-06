pub mod common;

use common::{build_polka_from_move, parse_to_blob, BuildOptions};
use polkavm::{Config, Engine, Linker, Module};

#[test]
pub fn test_multiple_functions() -> anyhow::Result<()> {
    let build_options = BuildOptions::new("output/multiple_functions.polkavm")
        .source("../examples/basic/sources/multiple_functions.move");

    let move_byte_code = build_polka_from_move(build_options)?;

    let blob = parse_to_blob(&move_byte_code)?;

    let config = Config::from_env()?;
    let engine = Engine::new(&config)?;
    let module = Module::from_blob(&engine, &Default::default(), blob)?;

    let linker: Linker = Linker::new();

    // Link the host functions with the module.
    let instance_pre = linker.instantiate_pre(&module)?;

    // Instantiate the module.
    let mut instance = instance_pre.instantiate()?;

    let res: u64 = instance
        .call_typed_and_get_result(&mut (), "sum", (5u64, 6u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 11);

    let res: u64 = instance
        .call_typed_and_get_result(&mut (), "sum_plus_const_5", (5u64, 10u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 20);

    let res: u64 = instance
        .call_typed_and_get_result(&mut (), "sum_of_3", (1u64, 2u64, 3u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 6);

    let res: u64 = instance
        .call_typed_and_get_result(&mut (), "sum_plus_const_5", (5u64, 10u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 20);

    let res: u64 = instance
        .call_typed_and_get_result(&mut (), "sum_for_rich", (6u64, 7u64, 8u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 121);

    let res: u32 = instance
        .call_typed_and_get_result(&mut (), "sum_different_size_args", (1u32, 2u64, 3u32))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 6);

    let no_extras: u32 = 5; // polkaVM typed args don't support bool - any value bigger than 0 is true
    let res: u64 = instance
        .call_typed_and_get_result(&mut (), "sum_if_extras", (1u32, no_extras, 10u64))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

    assert_eq!(res, 11);

    Ok(())
}
