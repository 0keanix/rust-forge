mod common;

use common::{TempDir, run, stdout, write};

fn manifest(name: &str, deps: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{deps}"
    )
}

#[test]
fn workspace_with_clap_and_bevy() {
    let dir = TempDir::new("domains-ws");
    let root = dir.path();
    write(
        &root.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"app\", \"game\"]\n",
    );
    write(
        &root.join("app/Cargo.toml"),
        &manifest("app", "clap = \"4\"\n"),
    );
    write(&root.join("app/src/main.rs"), "fn main() {}\n");
    write(
        &root.join("game/Cargo.toml"),
        &manifest("game", "bevy = \"0.17\"\n"),
    );
    write(&root.join("game/src/lib.rs"), "");
    write(
        &root.join("Cargo.lock"),
        "version = 4\n\n[[package]]\nname = \"bevy\"\nversion = \"0.17.2\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n[[package]]\nname = \"clap\"\nversion = \"4.5.40\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
    );
    let out = run(&["domains"], root, "");
    let text = stdout(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        text,
        "cli skill=rust-domain-cli crate=clap version=4.5.40\ngame skill=rust-domain-game crate=bevy version=0.17.2\n"
    );
}

#[test]
fn each_domain_from_its_key_crate() {
    let cases = [
        ("clap", "cli"),
        ("bevy", "game"),
        ("axum", "web"),
        ("tauri", "desktop"),
        ("leptos", "wasm"),
        ("yew", "wasm"),
        ("wasm-bindgen", "wasm"),
        ("uniffi", "ffi-mobile"),
        ("candle-core", "ml"),
        ("burn", "ml"),
        ("llama-cpp-2", "ml"),
        ("embassy-executor", "embedded"),
        ("cortex-m", "embedded"),
    ];
    for (krate, domain) in cases {
        let dir = TempDir::new("domains-one");
        write(
            &dir.path().join("Cargo.toml"),
            &manifest("probe", &format!("{krate} = \"1\"\n")),
        );
        write(&dir.path().join("src/lib.rs"), "");
        let out = run(&["domains"], dir.path(), "");
        assert_eq!(
            out.status.code(),
            Some(0),
            "{krate}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            stdout(&out),
            format!("{domain} skill=rust-domain-{domain} crate={krate} version=unknown\n"),
            "{krate}"
        );
    }
}
