use crate::init::{RUC_DIR, WORKING_DIR};
use anyhow::{bail, Context, Result};
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
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::None => write!(f, "none"),
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
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
            "commit" => Ok(Kind::Commit),
            _ => Ok(Kind::None),
        }
    }
}

#[derive(Debug)]
pub struct Object {
    pub kind: Kind,
    pub contents: String,
}

pub fn hash_contents(contents: &String, kind: Kind) -> Result<String> {
    let text = kind.to_string() + std::str::from_utf8(&[b'\x00']).unwrap() + contents;

    // Hash it with SHA1 as in Git.
    let mut hasher = Sha1::new();
    hasher.update(&text);
    let hashed = format!("{:x}", hasher.finalize());

    // And store it in plain text, no compressing nor fancy splitting like Git does.
    let op = WORKING_DIR.join(RUC_DIR).join("objects").join(&hashed);
    let mut file =
        File::create(op).with_context(|| format!("while creating object {} in store", &hashed))?;
    file.write_all(text.as_bytes())
        .with_context(|| format!("while saving object {} in store", &hashed))?;

    Ok(hashed)
}

pub fn hash(path: &Path, kind: Kind, verbose: bool) -> Result<String> {
    // NOTE: The object will be saved in plain text out of simplicity, but there
    // is a header beforehand to store stuff like the type of the object. The
    // header is then delimited by a \x00 byte.
    let contents = &std::fs::read_to_string(path)?;

    let res = hash_contents(contents, kind)?;
    if verbose {
        println!(
            "stored given file {} into the object database",
            path.display()
        );
    }

    Ok(res)
}

pub fn get(object: &String) -> Result<Object> {
    let path = WORKING_DIR.join(RUC_DIR).join("objects").join(object);
    let contents =
        std::fs::read_to_string(path).context(format!("while reading 'objects/{}'", object))?;

    let v: Vec<&str> = contents
        .splitn(2, std::str::from_utf8(&[b'\x00']).unwrap())
        .collect();

    if v.len() != 2 {
        bail!("bad format for object {}", object);
    }

    Ok(Object {
        kind: Kind::from_str(v[0]).unwrap(),
        contents: v[1].to_string(),
    })
}

pub fn cat(object: &String) -> Result<()> {
    let res = get(object)?;

    print!("Kind: {}\nContents:\n{}\n", res.kind, res.contents);

    Ok(())
}
