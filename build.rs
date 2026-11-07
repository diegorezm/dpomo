//! Embeds every `themes/*.toml` into the binary, so adding a built-in theme
//! is just dropping a file in that folder.

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=themes");

    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("themes");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("themes/ folder is missing")
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();

    let mut code = String::from("pub const BUILTIN: &[(&str, &str)] = &[\n");
    for path in files {
        let stem = path.file_stem().unwrap().to_string_lossy();
        code += &format!(
            "    ({stem:?}, include_str!({:?})),\n",
            path.display().to_string()
        );
    }
    code += "];\n";

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("builtin_themes.rs");
    fs::write(out, code).unwrap();
}
