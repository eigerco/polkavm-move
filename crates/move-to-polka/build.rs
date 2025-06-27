use std::path::PathBuf;

// build.rs
use cargo_metadata::{MetadataCommand, Package};

fn main() -> anyhow::Result<()> {
    fetch_move_stdlib()?;
    build_move_native_lib()?;
    Ok(())
}

fn fetch_move_stdlib() -> anyhow::Result<()> {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("failed to fetch cargo metadata");

    let git_prefix = "git+https://github.com/joske/move-on-aptos.git";
    let dep_pkg: &Package = metadata
        .packages
        .iter()
        .find(|p| {
            p.name.as_str() == "move-stdlib"
                && p.source
                    .as_ref()
                    .map(|s| s.repr.starts_with(git_prefix))
                    .unwrap_or(false)
        })
        .expect("move-stdlib not found in cargo metadata");

    let dep_dir = dep_pkg
        .manifest_path
        .parent()
        .expect("manifest_path had no parent")
        .to_path_buf();

    println!("cargo:rustc-env=MOVE_STDLIB_PATH={dep_dir}");
    println!("cargo:rerun-if-changed={dep_dir}");
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
