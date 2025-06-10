use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;
use itertools::Itertools;
use log::{debug, error};
use which::{which, which_in};

pub struct Lld(PathBuf);

impl Lld {
    pub fn try_init() -> anyhow::Result<Self> {
        let path = which("ld.lld")
            .or(which_in("ld.lld", Some("/opt/homebrew/bin"), "/"))
            .context("no ld.lld in PATH")?;
        Ok(Self(path))
    }

    pub fn merge_object_files(
        &self,
        sources: &[&PathBuf],
        output: &PathBuf,
        gc_sections: bool,
    ) -> anyhow::Result<()> {
        let mut cmd = Command::new(&self.0);
        // this flag is essential as it strips all unused symbols AFTER we merge native lib with actual move program code
        // otherwise there are lot of bits (like atomics) included by rust compiler which result in undefined symbols
        // during polka linking phase.
        if gc_sections {
            cmd.arg("--gc-sections");
        }
        let status = cmd.arg("-r").arg("-o").arg(output).args(sources).status()?;
        if !status.success() {
            error!("ld.lld execution error:");
            anyhow::bail!("lld failed: exit status: {}", status.code().unwrap())
        }
        Ok(())
    }
}

pub struct NativeBuildTools {
    cargo: PathBuf,
    lld: Lld,
    llvm_ar: PathBuf,
}

impl NativeBuildTools {
    pub fn try_init() -> anyhow::Result<Self> {
        Ok(Self {
            cargo: which("cargo").context("no cargo in PATH")?,
            lld: Lld::try_init()?,
            llvm_ar: which("llvm-ar")
                .or(which_in(
                    "llvm-ar",
                    Some("/opt/homebrew/opt/llvm/bin/"),
                    "/",
                ))
                .context("no llvm-ar in PATH")?,
        })
    }

    fn run_cargo(
        &self,
        crate_dir: &PathBuf,
        target_dir: &PathBuf,
        args: &[&str],
    ) -> anyhow::Result<()> {
        debug!(
            "running {:?} in {:?} with args: {:?}",
            self.cargo, crate_dir, args
        );
        debug!("target output dir: {target_dir:?}");
        let mut cmd = Command::new(&self.cargo);
        cmd.current_dir(crate_dir);
        cmd.env_remove("RUSTC_WRAPPER");
        cmd.env_remove("RUSTC_WORKSPACE_WRAPPER");
        cmd.env_remove("CARGO");
        cmd.env_remove("RUSTUP_TOOLCHAIN");
        cmd.env_remove("RUSTC");
        cmd.env("CARGO_TARGET_DIR", target_dir);
        //cmd.env("CARGO", &self.cargo);
        cmd.env("CARGO_PROFILE_DEV_PANIC", "abort");
        cmd.env("CARGO_PROFILE_RELEASE_PANIC", "abort");
        cmd.args(args);

        let status = cmd.status()?;
        if !status.success() {
            anyhow::bail!("cargo failed: exit status: {}", status.code().unwrap())
        }

        Ok(())
    }

    fn extract_lib_archive(
        &self,
        target_dir: &PathBuf,
        lib_archive: &PathBuf,
    ) -> anyhow::Result<()> {
        let mut cmd = Command::new(&self.llvm_ar);
        let status = cmd
            .current_dir(target_dir)
            .arg("x")
            .arg(lib_archive)
            .status()?;
        if !status.success() {
            anyhow::bail!("llvm-ar failed with: {}", status.code().unwrap())
        }
        Ok(())
    }

    pub fn build_native_move_lib(
        &self,
        crate_path: &Path,
        out_path: &PathBuf,
    ) -> anyhow::Result<PathBuf> {
        debug!("building move-native runtime for polkavm in {out_path:?}");
        let final_object_file = out_path.join("polkavm_native_final.o");

        let target_json = "riscv32emac-unknown-none-polkavm.json".to_string();

        // Using `cargo rustc` to compile move-native as a staticlib.
        // See move-native documentation on `no-std` compatibilty for explanation.
        // Release mode is required to eliminate large stack frames.
        self.run_cargo(
            &crate_path.canonicalize()?,
            &out_path.canonicalize()?,
            &[
                "rustc",
                "--crate-type=staticlib",
                "-Z",
                "build-std=core,alloc",
                "--target",
                &target_json,
                "--release",
                "--features",
                "polkavm",
                "--verbose", // for build process debuging purposes
                "--",
                // following are direct rustc flags
                // create one object in static library - we need this for object merge invocation later (and probably embedding too)
                "-C",
                "codegen-units=1",
                // optimize for binary size, but also respect performance
                "-C",
                "opt-level=s",
            ],
        )?;

        let archive_file = out_path
            .join("riscv32emac-unknown-none-polkavm")
            .join("release")
            .join("libpolkavm_move_native.a");

        if !archive_file.exists() {
            anyhow::bail!("native runtime not found at {archive_file:?}. this is a bug");
        }

        let extracted_content = out_path.join("archive_contents");
        // cleanup any possible leftovers
        if extracted_content.exists() {
            std::fs::remove_dir_all(&extracted_content)?;
        }
        std::fs::create_dir_all(&extracted_content)?;
        self.extract_lib_archive(&extracted_content, &archive_file.canonicalize()?)?;

        let mut object_files = vec![];
        // collect all extracted files
        for entry in std::fs::read_dir(extracted_content)? {
            let path: PathBuf = entry?.path();
            if path.extension().is_some_and(|ext| ext == "o") {
                object_files.push(path);
            }
        }

        self.lld.merge_object_files(
            &object_files.iter().collect_vec(),
            &final_object_file,
            false,
        )?;

        Ok(final_object_file)
    }
}
