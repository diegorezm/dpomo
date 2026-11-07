//! Embeds `themes/*.toml` and `characters/<id>/*` into the binary, so adding
//! a built-in theme or character is just dropping files in those folders.

use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=themes");
    println!("cargo:rerun-if-changed=characters");

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut code = String::from("pub const BUILTIN: &[(&str, &str)] = &[\n");
    for path in files(&root.join("themes"), "toml") {
        let stem = path.file_stem().unwrap().to_string_lossy();
        writeln!(
            code,
            "    ({stem:?}, include_str!({:?})),",
            path.display().to_string()
        )
        .unwrap();
    }
    code += "];\n";
    fs::write(out.join("builtin_themes.rs"), code).unwrap();

    // (character id, [(file name, bytes)])
    let mut code = String::from(
        "/// (file name, contents)\npub type File = (&'static str, &'static [u8]);\n\
         pub const BUILTIN: &[(&str, &[File])] = &[\n",
    );
    let mut dirs: Vec<PathBuf> = fs::read_dir(root.join("characters"))
        .expect("characters/ folder is missing")
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for dir in dirs {
        let id = dir.file_name().unwrap().to_string_lossy();
        writeln!(code, "    ({id:?}, &[").unwrap();
        for path in files(&dir, "svg").into_iter().chain(files(&dir, "toml")) {
            let name = path.file_name().unwrap().to_string_lossy();
            let path = path.display().to_string();
            writeln!(code, "        ({name:?}, include_bytes!({path:?})),").unwrap();
        }
        code += "    ]),\n";
    }
    code += "];\n";
    fs::write(out.join("builtin_characters.rs"), code).unwrap();
}

fn files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("{} is missing", dir.display()))
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|e| e == ext))
        .collect();
    files.sort();
    files
}
