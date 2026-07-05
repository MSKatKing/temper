use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_source(assets_path: PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR missing"));
    let reports_dir = assets_path.join("generated").join("reports");

    let mut content = String::from("pub mod reports {\n");
    write_dir(&mut content, &reports_dir, 1);
    content.push_str("}\n");

    fs::write(out_dir.join("generated.rs"), content).expect("Failed to write generated source");
}

fn write_dir(content: &mut String, dir: &Path, depth: usize) {
    let mut entries = fs::read_dir(dir)
        .expect("Failed to read generated reports directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to read generated reports entry");

    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            write_indent(content, depth);
            content.push_str("pub mod ");
            content.push_str(&to_ident(&name, false));
            content.push_str(" {\n");

            write_dir(content, &path, depth + 1);

            write_indent(content, depth);
            content.push_str("}\n");
        } else {
            let stem = path
                .file_stem()
                .expect("Generated report file missing stem")
                .to_string_lossy();

            write_indent(content, depth);
            content.push_str("pub const ");
            content.push_str(&to_ident(&stem, true));
            content.push_str(": &str = include_str!(");
            content.push_str(&format!("{:?}", path.to_string_lossy()));
            content.push_str(");\n");
        }
    }
}

fn write_indent(content: &mut String, depth: usize) {
    for _ in 0..depth {
        content.push_str("    ");
    }
}

fn to_ident(name: &str, uppercase: bool) -> String {
    let mut ident = String::new();

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if uppercase {
                ident.push(ch.to_ascii_uppercase());
            } else {
                ident.push(ch.to_ascii_lowercase());
            }
        } else {
            ident.push('_');
        }
    }

    if ident
        .as_bytes()
        .first()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
        ident.insert(0, '_');
    }

    ident
}
