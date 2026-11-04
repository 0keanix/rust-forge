//! Shared helpers: fresh temp directories, fixture crates, running the binary.
#![allow(dead_code, reason = "each test binary uses a subset of the helpers")]
#![allow(clippy::unwrap_used, reason = "test helpers")]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};

/// A fresh directory under the system temp dir, removed on drop.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        static N: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "forge-tools-test-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

pub fn fixture(rel: &str) -> String {
    std::fs::read_to_string(fixtures().join(rel)).unwrap()
}

pub fn write(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, text).unwrap();
}

/// Write a dependency-free library crate into `dir`.
pub fn write_crate(dir: &Path, lib_rs: &str) {
    write(
        &dir.join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n",
    );
    write(&dir.join("src").join("lib.rs"), lib_rs);
}

pub const CLEAN_LIB: &str = "/// Adds one.\npub fn add_one(x: u32) -> u32 {\n    x + 1\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add_one(1), 2);\n    }\n}\n";

pub const BADLY_FORMATTED_LIB: &str = "pub fn add_one(x:u32)->u32{x+1}\n";

/// Run the forge-tools binary in `cwd` with `stdin`; cargo runs offline.
pub fn run(args: &[&str], cwd: &Path, stdin: &str) -> Output {
    run_env(args, cwd, stdin, &[])
}

pub fn run_env(args: &[&str], cwd: &Path, stdin: &str, env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_forge-tools"));
    cmd.args(args)
        .current_dir(cwd)
        .env("CARGO_NET_OFFLINE", "true")
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

pub fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
