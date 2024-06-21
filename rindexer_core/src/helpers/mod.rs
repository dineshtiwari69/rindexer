use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Converts a CamelCase string to snake_case.
pub fn camel_to_snake(s: &str) -> String {
    let mut snake_case = String::new();
    let mut previous_was_uppercase = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_alphanumeric() || c == '_' {
            if c.is_uppercase() {
                // Insert an underscore if it's not the first character and the previous character wasn't uppercase
                if i > 0
                    && (!previous_was_uppercase
                    || (i + 1 < s.len() && s.chars().nth(i + 1).unwrap().is_lowercase()))
                {
                    snake_case.push('_');
                }
                snake_case.push(c.to_ascii_lowercase());
                previous_was_uppercase = true;
            } else {
                snake_case.push(c);
                previous_was_uppercase = false;
            }
        }
    }

    snake_case
}

/// Formats all Rust source files in the given folder using `cargo fmt`.
pub fn format_all_files_for_project<P: AsRef<Path>>(project_path: P) {
    let manifest_path = project_path.as_ref().join("Cargo.toml");

    let status = Command::new("cargo")
        .arg("fmt")
        .arg("--manifest-path")
        .arg(manifest_path)
        .status()
        .expect("Failed to execute cargo fmt.");

    if !status.success() {
        panic!("cargo fmt failed with status: {:?}", status);
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WriteFileError {
    #[error("Could not create dir: {0}")]
    CouldNotCreateDir(std::io::Error),

    #[error("Could not convert string to bytes: {0}")]
    CouldNotConvertToBytes(std::io::Error),

    #[error("Could not create the file: {0}")]
    CouldNotCreateFile(std::io::Error),
}

/// Writes contents to a file, creating directories as needed, and formats the file.
pub fn write_file(path: &Path, contents: &str) -> Result<(), WriteFileError> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(WriteFileError::CouldNotCreateDir)?
    }

    let mut file = File::create(path).map_err(WriteFileError::CouldNotCreateFile)?;
    file.write_all(contents.as_bytes())
        .map_err(WriteFileError::CouldNotConvertToBytes)?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum CreateModFileError {
    #[error("Could not read path: {0}")]
    ReadPath(std::io::Error),

    #[error("Could not read dir entries: {0}")]
    ReadDirEntries(std::io::Error),

    #[error("Could not create file: {0}")]
    CreateFile(std::io::Error),

    #[error("Could not write extra lines to file: {0}")]
    WriteExtraLines(std::io::Error),
}

/// Creates a `mod.rs` file for a given directory, including submodules for all Rust files and directories.
pub fn create_mod_file(
    path: &Path,
    code_generated_comment: bool,
) -> Result<(), CreateModFileError> {
    let entries = fs::read_dir(path).map_err(CreateModFileError::ReadPath)?;

    let mut mods = Vec::new();
    let mut dirs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(CreateModFileError::ReadDirEntries)?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                dirs.push(dir_name.to_owned());
                create_mod_file(&path, code_generated_comment)?;
            }
        } else if let Some(ext) = path.extension() {
            if ext == "rs" && path.file_stem().map_or(true, |s| s != "mod") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    mods.push(file_stem.to_owned());
                }
            }
        }
    }

    if !mods.is_empty() || !dirs.is_empty() {
        let mod_path = path.join("mod.rs");
        let mut mod_file = File::create(mod_path).map_err(CreateModFileError::CreateFile)?;

        writeln!(mod_file, "#![allow(dead_code, unused)]")
            .map_err(CreateModFileError::WriteExtraLines)?;

        if code_generated_comment {
            write!(
                mod_file,
                r#"
            /// THIS IS A GENERATED FILE. DO NOT MODIFY MANUALLY.
            ///
            /// This file was auto generated by rindexer - https://github.com/joshstevens19/rindexer
            /// Any manual changes to this file will be overwritten.
            "#
            )
            .map_err(CreateModFileError::WriteExtraLines)?;
        }

        for item in mods.iter().chain(dirs.iter()) {
            if item.contains("_abi_gen") {
                writeln!(mod_file, "mod {};", item).map_err(CreateModFileError::WriteExtraLines)?;
            } else {
                writeln!(mod_file, "pub mod {};", item)
                    .map_err(CreateModFileError::WriteExtraLines)?;
            }
        }
    }

    Ok(())
}

pub fn generate_random_id(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("CamelCase"), "camel_case");
        assert_eq!(camel_to_snake("Camel-Case"), "camel_case");
        assert_eq!(camel_to_snake("camelCase"), "camel_case");
        assert_eq!(camel_to_snake("camel_case"), "camel_case");
        assert_eq!(camel_to_snake("Camel"), "camel");
        assert_eq!(camel_to_snake("camel"), "camel");
        assert_eq!(camel_to_snake("collectNFTId"), "collect_nft_id");
        assert_eq!(camel_to_snake("ERC20"), "erc20");
    }
}
