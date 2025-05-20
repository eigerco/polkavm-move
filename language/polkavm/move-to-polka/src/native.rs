// check build.rs how native lib is actually being built
const MOVE_NATIVE_LIB_BYTES: &[u8] = include_bytes!(env!("MOVE_NATIVE_OBJECT_FILE"));

pub fn move_native_lib_content() -> &'static [u8] {
    MOVE_NATIVE_LIB_BYTES
}
