use crate::init::WORKING_DIR;
use crate::object;

use anyhow::{bail, Context, Result};
use std::fs;
use std::fs::{DirEntry, File};
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

lazy_static! {
    // TODO: properly initialize this.
    static ref IGNORED_ENTRIES: Vec<&'static str> = vec![".ruc", ".git", "target", "tests"];
}

fn should_be_ignored(entry: &DirEntry) -> bool {
    let path = entry.path();

    match path.strip_prefix(WORKING_DIR.to_owned()) {
        Ok(en) => {
            if let Some(base) = en.components().next() {
                if let Some(raw) = base.as_os_str().to_str() {
                    for ignored in IGNORED_ENTRIES.iter().copied() {
                        if raw == ignored {
                            return true;
                        }
                    }
                }
            }

            false
        }
        Err(_) => false,
    }
}

#[derive(Debug)]
pub struct TreeEntry {
    id: String,
    kind: object::Kind,
    path: String,
}

pub fn traverse_write_tree(path: &Path) -> Result<String> {
    let mut entries: Vec<TreeEntry> = vec![];

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if should_be_ignored(&entry) {
            continue;
        }

        // Try to get the path relative to the working directory. This way it's
        // easier to migrate from different databases.
        let entry_path_full = entry.path();
        let entry_path = match entry_path_full.strip_prefix(WORKING_DIR.to_owned()) {
            Ok(ep) => ep,
            Err(_) => entry_path_full.as_path(),
        }
        .to_str();

        // If it's a directory, then we have to traverse the tree one level
        // below, otherwise we can just hash the file.
        if file_type.is_dir() {
            entries.push(TreeEntry {
                id: traverse_write_tree(&entry.path())?,
                kind: object::Kind::Tree,
                path: entry_path.unwrap().to_string(),
            });
        } else {
            entries.push(TreeEntry {
                id: object::hash(&entry.path(), object::Kind::Blob, false)?,
                kind: object::Kind::Blob,
                path: entry_path.unwrap().to_string(),
            });
        }
    }

    // Bundle all the entries that have been found (both trees and blobs), and
    // store it into a tree kind. The return value on success for this function
    // will be the ID for this newly generated tree file.
    let contents = entries.iter().fold(String::new(), |a, b| {
        a + &format!("{} {} {}", b.kind, b.id, b.path) + "\n"
    });

    object::hash_contents(&contents, object::Kind::Tree)
}

pub fn write_tree(path: &Path) -> Result<()> {
    traverse_write_tree(path)?;

    println!("Stored tree from {}", path.display());

    Ok(())
}

fn get_entries(contents: &str) -> Result<Vec<TreeEntry>> {
    let mut error = false;

    let res = contents
        .lines()
        .map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() != 3 {
                error = true;
            }

            TreeEntry {
                kind: object::Kind::from_str(fields[0]).unwrap(),
                id: fields[1].to_string(),
                path: fields[2].to_string(),
            }
        })
        .collect::<Vec<_>>();

    if error {
        bail!("badly formatted tree!");
    }

    Ok(res)
}

pub fn read_blob(blob: &TreeEntry) -> Result<()> {
    let obj = object::get(&blob.id)?;

    if let Some(dir) = Path::new(&blob.path).parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }

    let mut file = File::create(&blob.path)?;
    file.write_all(obj.contents.as_bytes())?;

    Ok(())
}

fn empty_directory(dir: &PathBuf) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if should_be_ignored(&entry) {
            continue;
        }

        if file_type.is_dir() {
            empty_directory(&entry.path()).ok();
        } else {
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}

pub fn traverse_read_tree(tree: &String) -> Result<()> {
    let obj = object::get(tree)?;
    if obj.kind != object::Kind::Tree {
        bail!("object '{}' is not a tree!", tree);
    }

    let entries = get_entries(&obj.contents)
        .with_context(|| format!("while fetching entries for tree '{}'", tree))?;

    for parsed in entries {
        match parsed.kind {
            object::Kind::Tree => traverse_read_tree(&parsed.id)?,
            object::Kind::Blob => read_blob(&parsed).with_context(|| "while reading blob")?,
            _ => bail!("unknown error!"),
        };
    }

    Ok(())
}

pub fn read_tree(tree: &String) -> Result<()> {
    empty_directory(&WORKING_DIR).ok();

    traverse_read_tree(tree)?;

    Ok(())
}
