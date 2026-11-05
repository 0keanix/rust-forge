#!/usr/bin/env bash
set -euo pipefail
mkdir -p src
cat > Cargo.toml <<'EOF'
[package]
name = "appconfig"
version = "0.1.0"
edition = "2024"

[dependencies]

[workspace]
EOF
cat > src/lib.rs <<'EOF'
/// Application configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub name: String,
    pub port: u16,
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "app".to_owned(),
            port: 8080,
            verbose: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port_is_8080() {
        assert_eq!(Config::default().port, 8080);
    }
}
EOF
printf 'target/\n' > .gitignore
git init -q
git add -A
git -c user.name=eval -c user.email=eval@example.invalid commit -q -m "initial fixture"
