use crate::init::{RUC_DIR, WORKING_DIR};
use crate::object;
use crate::tree;

use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::Read;
use std::process;
use std::process::{Command, Stdio};

pub fn editor() -> Result<String> {
    let program = match env::var("EDITOR") {
        Ok(v) => v,
        Err(_) => bail!("could not get the default EDITOR"),
    };

    let path = WORKING_DIR.join(RUC_DIR).join("COMMIT_EDITMSG");
    fs::File::create(&path).expect("could not create temporary file for editing the message");

    process::Command::new(program).arg(&path).status()?;

    let mut editable = String::new();
    fs::File::open(&path)?.read_to_string(&mut editable)?;
    fs::remove_file(path)?;

    Ok(editable.trim_end().to_owned())
}

pub fn commit(message: String) -> Result<()> {
    let id = tree::traverse_write_tree(&WORKING_DIR)?;

    let parent_id = get_ref(&String::from("HEAD"))?;
    let contents = if parent_id.is_empty() {
        format!("tree {}\n\n{}", id, message)
    } else {
        format!("tree {}\nparent {}\n\n{}", id, parent_id, message)
    };

    let commit_id = object::hash_contents(&contents, object::Kind::Commit)?;
    update_ref(&String::from("HEAD"), &commit_id)?;

    Ok(())
}

pub fn ref_to_oid(name: &String) -> Result<String> {
    for path in &["", "refs/", "refs/tags/", "refs/heads/"] {
        let full_path = WORKING_DIR.join(RUC_DIR).join(path).join(name);

        if full_path.exists() {
            return get_ref(&format!("{}{}", path, name));
        }
    }

    Ok(name.to_owned())
}

pub fn get_ref(name: &String) -> Result<String> {
    let ref_file = WORKING_DIR.join(RUC_DIR).join(name);

    match std::fs::read_to_string(ref_file) {
        Ok(contents) => Ok(contents),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Ok(String::new()),
            _ => bail!(format!("could not get the current value for {}!", name)),
        },
    }
}

pub fn update_ref(name: &String, commit_id: &String) -> Result<()> {
    let ref_file = WORKING_DIR.join(RUC_DIR).join(name);

    let mut file =
        fs::File::create(ref_file).with_context(|| format!("could not save {} state", name))?;
    file.write_all(commit_id.as_bytes())
        .with_context(|| format!("could not save {} state", name))?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub tree: String,
    pub parent: Option<String>,
    pub contents: String,
}

impl Iterator for Commit {
    type Item = Commit;

    fn next(&mut self) -> Option<Commit> {
        match &self.parent {
            Some(parent) => match get_commit(parent) {
                Ok(commit) => {
                    *self = commit;
                    Some(self.to_owned())
                }
                Err(e) => {
                    println!("failed to fetch commit {}", e);
                    None
                }
            },
            None => None,
        }
    }
}

impl Commit {
    // Creates an empty commit with the given string as the ID of its parent.
    // This way it can be iterated through the Commit Iterator.
    fn iter_as_parent(from: &String) -> Commit {
        Commit {
            id: String::new(),
            tree: String::new(),
            parent: Some(from.to_owned()),
            contents: String::new(),
        }
    }
}

fn parse_header_element(element: Option<&str>, key: &str) -> Option<String> {
    match element {
        Some(el) => {
            let fields = el.split_whitespace().collect::<Vec<_>>();
            if fields.len() == 2 && fields[0] == key {
                Some(fields[1].to_owned())
            } else {
                None
            }
        }
        None => None,
    }
}

fn get_commit(id: &String) -> Result<Commit> {
    let obj = object::get(id).with_context(|| format!("while getting commit {}", id))?;

    let mut lines = obj.contents.lines();
    let tree = parse_header_element(lines.next(), "tree").unwrap();
    let parent = parse_header_element(lines.next(), "parent");

    if parent.is_some() {
        lines.next();
    }
    let contents = lines.fold(String::new(), |acc, x| format!("{}{}", acc, x));

    Ok(Commit {
        id: id.to_owned(),
        tree,
        parent,
        contents,
    })
}

pub fn log(from: &String) -> Result<()> {
    let mut first = true;

    for commit in Commit::iter_as_parent(from) {
        if !first {
            println!();
        }

        println!("commit {}\n\n{}", commit.id, commit.contents);

        first = false;
    }

    Ok(())
}

pub fn checkout(id: &String) -> Result<()> {
    let commit = get_commit(id)?;

    tree::read_tree(&commit.tree)?;

    update_ref(&String::from("HEAD"), id)?;
    Ok(())
}

pub fn create_tag(name: &String, id: &String) -> Result<()> {
    update_ref(&format!("refs/tags/{}", name), id)?;

    Ok(())
}

pub fn graph() -> Result<()> {
    let paths = fs::read_dir(WORKING_DIR.join(RUC_DIR).join("refs").join("tags")).unwrap();
    let mut dot = String::from("digraph commits {\n");
    let mut commits = vec![];

    // First of all, iterate over the different references that we have.
    for path in paths {
        let file = path.unwrap();
        let fname = file.file_name();
        let name = fname.to_str().unwrap();
        let rf = get_ref(&format!("refs/tags/{}", &name.to_string()))?;

        commits.push(rf.clone());

        dot.push_str(format!("\"{}\" [shape=note]\n", &name).as_str());
        dot.push_str(format!("\"{}\" -> \"{}\"\n", &name, rf).as_str());
    }

    for commit_id in commits {
        for commit in Commit::iter_as_parent(&commit_id) {
            let abbreved = commit.id.get(0..12).unwrap_or(&commit.id);
            dot.push_str(
                format!(
                    "\"{}\" [shape=box style=filled label=\"{}\"]\n",
                    commit.id, abbreved
                )
                .as_str(),
            );

            if let Some(parent) = commit.parent {
                dot.push_str(format!("\"{}\" -> \"{}\"", commit.id, parent).as_str());
            }
        }
    }

    dot.push('}');

    let dot_command = Command::new("dot")
        .args(["-Tgtk"])
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| "while running the `dot` command")?;
    write!(dot_command.stdin.unwrap(), "{}", dot)?;

    Ok(())
}
