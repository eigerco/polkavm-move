use std::collections::HashSet;

use move_to_polka::{
    initialize_logger,
    linker::{create_blob, create_instance_from_pre, create_instance_pre},
};
use once_cell::sync::OnceCell;
use polkavm::{Instance, InstancePre};
use polkavm_move_native::host::{ProgramError, Runtime};

static INSTANCE_PRE: OnceCell<InstancePre<Runtime, ProgramError>> = OnceCell::new();

fn get_instance_pre() -> &'static InstancePre<Runtime, ProgramError> {
    INSTANCE_PRE.get_or_init(|| {
        initialize_logger();
        let blob = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                create_blob(
                    "output/ring_deque/ring_deque.polkavm",
                    "../../examples/ring_deque/",
                    HashSet::new(),
                )
                .expect("Failed to compile Move source to PolkaVM bytecode")
            })
            .expect("Failed to spawn compilation thread")
            .join()
            .expect("Compilation thread panicked");
        create_instance_pre(blob).expect("Failed to create InstancePre")
    })
}

fn new_instance() -> anyhow::Result<(Instance<Runtime, ProgramError>, Runtime)> {
    create_instance_from_pre(get_instance_pre())
}

macro_rules! ring_deque_test {
    ($name:ident) => {
        #[test]
        pub fn $name() -> anyhow::Result<()> {
            let (mut instance, mut runtime) = new_instance()?;
            let result = instance
                .call_typed_and_get_result::<(), ()>(&mut runtime, stringify!($name), ())
                .map_err(|e| anyhow::anyhow!("{e:?}"));
            assert!(
                result.is_ok(),
                "{} failed: {:?}",
                stringify!($name),
                result.err()
            );
            Ok(())
        }
    };
}

ring_deque_test!(test_end_to_end);
ring_deque_test!(test_single_element);
ring_deque_test!(test_wrap_around);
ring_deque_test!(test_large_capacity);
