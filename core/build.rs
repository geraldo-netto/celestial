//! Build script — no C compilation needed; the pure-Rust engine has no
//! external dependencies.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
