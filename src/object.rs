use crate::init;
use sha1::{Digest, Sha1};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Kind {
    None,
    Blob,
    Tree,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::None => write!(f, "none"),
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
        }
    }
}

impl std::str::FromStr for Kind {
    type Err = ();

    fn from_str(input: &str) -> Result<Kind, Self::Err> {
        match input.to_lowercase().as_str() {
            "none" => Ok(Kind::None),
            "blob" => Ok(Kind::Blob),
            "tree" => Ok(Kind::Tree),
            _ => Ok(Kind::None),
        }
    }
}

#[derive(Debug)]
pub struct Object {
    pub kind: Kind,
    pub contents: String,
}

pub fn hash_contents(contents: &String, kind: Kind) -> String {
    let working_dir = init::working_dir();
    let text = kind.to_string() + std::str::from_utf8(&[b'\x00']).unwrap() + contents;

    // Hash it with SHA1 as in Git.
    let mut hasher = Sha1::new();
    hasher.update(&text);
    let hashed = format!("{:x}", hasher.finalize());

    // And store it in plain text, no compressing nor fancy splitting like Git does.
    let op = working_dir
        .join(init::RUC_DIR)
        .join("objects")
        .join(&hashed);
    let mut file = File::create(op).unwrap_or_else(|e| {
        println!("Could not store object: {}", e);
        std::process::exit(1);
    });
    file.write_all(text.as_bytes()).unwrap_or_else(|e| {
        println!("Could not store object: {}", e);
        std::process::exit(1);
    });

    hashed
}

pub fn hash(path: &Path, kind: Kind, verbose: bool) -> String {
    // The object will be saved in plain text out of simplicity, but there is a
    // header beforehand to store stuff like the type of the object. The header
    // is then delimited by a \x00 byte.
    let contents = &std::fs::read_to_string(path).unwrap_or_else(|e| {
        println!("Could not read file: {}", e);
        std::process::exit(1);
    });

    let res = hash_contents(contents, kind);
    if verbose {
        println!(
            "Stored given file {} into the object database",
            path.display()
        );
    }

    res
}

pub fn get(object: &String) -> Result<Object, String> {
    let path = init::working_dir()
        .join(init::RUC_DIR)
        .join("objects")
        .join(object);

    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let v: Vec<&str> = contents
                .splitn(2, std::str::from_utf8(&[b'\x00']).unwrap())
                .collect();

            if v.len() != 2 {
                return Err(format!("Bad format for object {}", object));
            }

            Ok(Object {
                kind: Kind::from_str(v[0]).unwrap(),
                contents: v[1].to_string(),
            })
        }
        Err(e) => Err(format!("Failed to read given object: {}", e)),
    }
}

pub fn cat(object: &String) {
    match get(object) {
        Ok(res) => print!("Kind: {}\nContents:\n{}", res.kind, res.contents),
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
}
