#!/usr/bin/env bash
set -euo pipefail
mkdir -p src
cat > Cargo.toml <<'EOF'
[package]
name = "inventory"
version = "0.1.0"
edition = "2024"

[dependencies]

[workspace]
EOF
cat > src/lib.rs <<'EOF'
/// Total length of all names plus the number of names.
pub fn total_len(names: Vec<String>) -> usize {
    let owned = names;
    let count = names.len();
    count + owned.iter().map(String::len).sum::<usize>()
}

/// Append a copy of the first item with a `-copy` suffix.
pub fn add_copy_of_first(items: &mut Vec<String>) {
    let first = &items[0];
    items.push(String::new());
    let last = items.len() - 1;
    items[last] = format!("{first}-copy");
}

/// Swap the first two items.
pub fn swap_first_two(items: &mut [String]) {
    let a = &mut items[0];
    let b = &mut items[1];
    std::mem::swap(a, b);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["ab".to_owned(), "cde".to_owned()]
    }

    #[test]
    fn total_len_counts_names_and_bytes() {
        assert_eq!(total_len(names()), 7);
    }

    #[test]
    fn add_copy_of_first_appends_suffixed_copy() {
        let mut items = names();
        add_copy_of_first(&mut items);
        assert_eq!(items, ["ab", "cde", "ab-copy"]);
    }

    #[test]
    fn swap_first_two_swaps() {
        let mut items = names();
        swap_first_two(&mut items);
        assert_eq!(items, ["cde", "ab"]);
    }
}
EOF
printf 'target/\n' > .gitignore
git init -q
git add -A
git -c user.name=eval -c user.email=eval@example.invalid commit -q -m "initial fixture"
