use std::{path::PathBuf, process::Command};

use anyhow::Context;
use log::{debug, error};

pub struct PlatformTools {
    rustc: PathBuf,
    cargo: PathBuf,
    lld: PathBuf,
}

impl PlatformTools {
    fn run_cargo(&self, target_dir: &PathBuf, args: &[&str]) -> anyhow::Result<()> {
        println!("running cargo in {:?} with args: {:?}", target_dir, args);
        let mut cmd = Command::new(&self.cargo);
        cmd.env_remove("RUSTUP_TOOLCHAIN");
        cmd.env_remove("RUSTC_WRAPPER");
        cmd.env_remove("RUSTC_WORKSPACE_WRAPPER");
        cmd.env("CARGO_TARGET_DIR", target_dir);
        cmd.env("CARGO", &self.cargo);
        cmd.env("RUSTC", &self.rustc);
        cmd.env("CARGO_PROFILE_DEV_PANIC", "abort");
        cmd.env("CARGO_PROFILE_RELEASE_PANIC", "abort");
        cmd.args(args);

        let status = cmd.status()?;
        if !status.success() {
            anyhow::bail!("running SBF cargo failed");
        }

        Ok(())
    }

    pub fn get_runtime(&self, out_path: &PathBuf) -> anyhow::Result<PathBuf> {
        debug!("building move-native runtime for polkavm in {out_path:?}");
        println!("building move-native runtime for polkavm in {out_path:?}");
        let archive_file = out_path
            .join("riscv32imac-unknown-none-elf")
            .join("release")
            .join("libmove_native.a");

        if archive_file.exists() {
            return Ok(archive_file);
        }

        let move_native = std::env::var("MOVE_NATIVE").expect("move native");
        let move_native = PathBuf::from(move_native);
        let move_native = move_native.join("Cargo.toml").to_string_lossy().to_string();

        // Using `cargo rustc` to compile move-native as a staticlib.
        // See move-native documentation on `no-std` compatibilty for explanation.
        // Release mode is required to eliminate large stack frames.
        let res = self.run_cargo(
            out_path,
            &[
                "rustc",
                "--crate-type=staticlib",
                "-p",
                "move-native",
                "--target",
                "riscv32imac-unknown-none-elf",
                "--manifest-path",
                &move_native,
                "--release",
                "--features",
                "polkavm",
                // "-q",
            ],
        );

        if let Err(e) = res {
            anyhow::bail!("{e}");
        }

        if !archive_file.exists() {
            anyhow::bail!("native runtime not found at {archive_file:?}. this is a bug");
        }

        Ok(archive_file)
    }

    pub fn merge_object_files(&self, sources: &[PathBuf], output: &PathBuf) -> anyhow::Result<()> {
        let output = Command::new(&self.lld)
            .arg("-r")
            .arg("-o")
            .arg(output)
            .args(sources)
            .output()?;
        let status = output.status;
        if !status.success() {
            error!("ld.ldd execution error:");
            error!("Stdout {}", String::from_utf8_lossy(&output.stdout));
            error!("Stderr {}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("lld failed: exit status: {}", status.code().unwrap())
        }
        Ok(())
    }
}

pub fn get_platform_tools() -> anyhow::Result<PlatformTools> {
    use which::{which, which_in};

    let tools = PlatformTools {
        rustc: which("rustc").context("no rustc in PATH")?,
        cargo: which("cargo").context("no cargo in PATH")?,
        lld: which("ld.lld")
            .or(which_in("ld.lld", Some("/opt/homebrew/bin"), "/"))
            .context("no ld.lld in PATH")?,
    };

    Ok(tools)
}
