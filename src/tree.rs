use crate::init;
use crate::object;

use std::fs;
use std::fs::{DirEntry, File};
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

fn should_be_ignored(entry: &DirEntry, working_dir: &Path) -> bool {
    // TODO: pass along an "option" struct that is passed through all commands.
    let ignored_entries = vec![".ruc", ".git", "target", "tests"];
    let path = entry.path();

    match path.strip_prefix(working_dir) {
        Ok(en) => {
            if let Some(base) = en.components().next() {
                if let Some(raw) = base.as_os_str().to_str() {
                    for ignored in ignored_entries {
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

pub fn traverse_write_tree(path: &Path, working_dir: &Path) -> std::io::Result<String> {
    let mut entries: Vec<TreeEntry> = vec![];

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if should_be_ignored(&entry, working_dir) {
            continue;
        }

        // Try to get the path relative to the working directory. This way it's
        // easier to migrate from different databases.
        let entry_path_full = entry.path();
        let entry_path = match entry_path_full.strip_prefix(working_dir) {
            Ok(ep) => ep,
            Err(_) => entry_path_full.as_path(),
        }
        .to_str();

        // If it's a directory, then we have to traverse the tree one level
        // below, otherwise we can just hash the file.
        if file_type.is_dir() {
            entries.push(TreeEntry {
                id: traverse_write_tree(&entry.path(), working_dir)?,
                kind: object::Kind::Tree,
                path: entry_path.unwrap().to_string(),
            });
        } else {
            entries.push(TreeEntry {
                id: object::hash(&entry.path(), object::Kind::Blob, false),
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

    Ok(object::hash_contents(&contents, object::Kind::Tree))
}

pub fn write_tree(path: &Path) {
    let wd = init::working_dir();

    match traverse_write_tree(path, &wd) {
        Ok(_) => println!("Stored tree from {}", path.display()),
        Err(e) => println!(
            "write-tree: fatal: left inconsistent because of the error: {}",
            e
        ),
    }
}

fn get_entries(contents: &str) -> Result<Vec<TreeEntry>, std::io::Error> {
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
        return Err(Error::new(ErrorKind::Other, "badly formatted tree!"));
    }

    Ok(res)
}

pub fn read_blob(blob: &TreeEntry) -> std::io::Result<()> {
    // TODO: properly handle the error...
    let obj = object::get(&blob.id).unwrap_or_else(|e| {
        println!("read-tree: failed to read blob: {}", e);
        std::process::exit(1);
    });

    if let Some(dir) = Path::new(&blob.path).parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }

    let mut file = File::create(&blob.path)?;
    file.write_all(obj.contents.as_bytes())?;

    Ok(())
}

fn empty_directory(dir: &PathBuf, wd: &PathBuf) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if should_be_ignored(&entry, wd) {
            continue;
        }

        if file_type.is_dir() {
            empty_directory(&entry.path(), wd).ok();
        } else {
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}

pub fn traverse_read_tree(tree: &String) {
    match object::get(tree) {
        Ok(obj) => {
            if obj.kind != object::Kind::Tree {
                println!("read-tree: wrong object for {}", tree);
                std::process::exit(1);
            }

            match get_entries(&obj.contents) {
                Ok(entries) => {
                    for parsed in entries {
                        match parsed.kind {
                            object::Kind::Tree => traverse_read_tree(&parsed.id),
                            object::Kind::Blob => read_blob(&parsed).unwrap_or_else(|e| {
                                println!("read-tree: failed to read blob {}: {}", parsed.id, e);
                                std::process::exit(1);
                            }),
                            _ => println!("SHOULD NOT HAPPEN!"),
                        };
                    }
                }
                Err(e) => println!("read-tree failed: {}", e),
            }
        }
        Err(e) => println!("read-tree failed: {}", e),
    }
}

pub fn read_tree(tree: &String) {
    let working_dir = init::working_dir();
    empty_directory(&working_dir, &working_dir).ok();

    traverse_read_tree(tree);
}
