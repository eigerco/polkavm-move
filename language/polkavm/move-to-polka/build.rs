// build.rs
use cargo_metadata::{MetadataCommand, Package};

fn main() {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("failed to fetch cargo metadata");

    let git_prefix = "git+https://github.com/move-language/move-on-aptos.git";
    let dep_pkg: &Package = metadata
        .packages
        .iter()
        .find(|p| {
            p.name == "move-stdlib"
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

    println!("cargo:rustc-env=MOVE_STDLIB_PATH={}", dep_dir);
    println!("cargo:rerun-if-changed={}", dep_dir);
}
