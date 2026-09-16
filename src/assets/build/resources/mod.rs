mod banner_pattern;

use proc_macro2::TokenStream;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::fs;
use std::fs::ReadDir;
use std::io::Error;
use std::path::{Path, PathBuf};

type BakeEntry = (&'static str, &'static str, fn(ReadDir) -> Result<TokenStream, BakeError>);

#[derive(Debug)]
enum BakeError {
    Io(Error),
    Deserialize(serde_json::Error),
}

pub fn bake_resources(out_dir: impl AsRef<Path>, generated_data_dir: impl AsRef<Path>) {
    const BAKE_FUNCTIONS: &[BakeEntry] = &[
        ("banner_patterns.rs", "banner_pattern", banner_pattern::bake_banner_patterns),
    ];

    let out_dir = PathBuf::from(out_dir.as_ref())
        .join("resources");
    let generated_data_dir = PathBuf::from(generated_data_dir.as_ref())
        .join("data")
        .join("minecraft");

    match fs::exists(&generated_data_dir) {
        Ok(exists) => if !exists {
            quit(format!("Generated data directory does not exist: '{}'", generated_data_dir.display()));
        },
        Err(err) => quit(format!("Failed to load generated data directory: {err}")),
    }

    for (output_file, input_path, bake_function) in BAKE_FUNCTIONS {
        let input_path = generated_data_dir.join(input_path);
        let directory = match fs::read_dir(&input_path) {
            Ok(directory) => directory,
            Err(err) => quit(format!("Failed to read input directory '{}' while baking 'resources/{}': {}", input_path.display(), output_file, err)),
        };

        let file = match bake_function(directory) {
            Ok(file) => file,
            Err(err) => quit(format!("Failed to bake resource file 'resources/{}': {:?}", output_file, err))
        };

        match fs::write(out_dir.join(output_file), file.to_string()) {
            Ok(_) => (),
            Err(err) => quit(format!("Failed to write output file 'resources/{}': {}", output_file, err))
        }
    }
}

fn quit(msg: impl Into<String>) -> ! {
    println!("cargo::error={}", msg.into());
    std::process::exit(-1);
}

fn gather_files<T: DeserializeOwned>(root: String, dir: ReadDir, map: &mut BTreeMap<String, T>) -> Result<(), BakeError> {
    for dir in dir {
        let dir = dir?;

        if dir.metadata()?.is_dir() {
            gather_files(
                format!(
                    "{}{}",
                    if root.is_empty() { "".to_string() } else { format!("{root}/") },
                    dir.file_name().display()),
                fs::read_dir(dir.path())?,
                map
            )?;
        }

        let file_stem = dir.path().file_stem().expect("should have name").display().to_string();
        let name = if root.is_empty() {
            file_stem
        } else {
            format!("{root}/{file_stem}")
        };

        let file = fs::read_to_string(&dir.path())?;
        let data = serde_json::from_str::<T>(&file)?;

        map.insert(name, data);
    }

    Ok(())
}

impl From<Error> for BakeError {
    fn from(value: Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for BakeError {
    fn from(value: serde_json::Error) -> Self {
        Self::Deserialize(value)
    }
}
