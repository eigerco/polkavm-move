use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    build_move_native_lib()?;
    Ok(())
}

fn build_move_native_lib() -> anyhow::Result<()> {
    let tools = build_tools::NativeBuildTools::try_init()?;

    let move_native_crate =
        std::env::var("MOVE_NATIVE_CRATE").unwrap_or("../polkavm-move-native".to_string());
    println!("cargo:rerun-if-changed={move_native_crate}");
    let move_native_crate = PathBuf::from(move_native_crate).canonicalize()?;

    let out_path = PathBuf::from(std::env::var("OUT_DIR")?).join("move-native-lib-build");
    std::fs::create_dir_all(&out_path)?;

    let object_file = tools.build_native_move_lib(&move_native_crate, &out_path)?;
    println!(
        "cargo:rustc-env=MOVE_NATIVE_OBJECT_FILE={}",
        object_file.canonicalize()?.to_string_lossy()
    );

    Ok(())
}
