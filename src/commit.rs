use crate::init;
use crate::object;
use crate::tree;

use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;
use std::process;

pub fn editor() -> Result<String, &'static str> {
    let program = match env::var("EDITOR") {
        Ok(v) => v,
        Err(_) => return Err("could not get the default EDITOR"),
    };

    let path = init::working_dir()
        .join(init::RUC_DIR)
        .join("COMMIT_EDITMSG");
    fs::File::create(&path).expect("could not create temporary file for editing the message");

    process::Command::new(program)
        .arg(&path)
        .status()
        .expect("editor exitted with bad value");

    let mut editable = String::new();
    fs::File::open(&path)
        .expect("could not open temporary file for editing the message")
        .read_to_string(&mut editable)
        .expect("could not read temporary file for editing the message");

    fs::remove_file(path).expect("could not remove temporary file for editing the message");

    Ok(editable.trim_end().to_owned())
}

pub fn commit(message: String) {
    let working_dir = init::working_dir();

    match tree::traverse_write_tree(&working_dir, &working_dir) {
        Ok(id) => {
            let parent_id = get_ref(&working_dir, &String::from("HEAD"));
            let contents = if parent_id.is_empty() {
                format!("tree {}\n\n{}", id, message)
            } else {
                format!("tree {}\nparent {}\n\n{}", id, parent_id, message)
            };
            let commit_id = object::hash_contents(&contents, object::Kind::Commit);
            update_ref(&working_dir, &String::from("HEAD"), &commit_id);
        }
        Err(e) => println!(
            "commit: fatal: left inconsistent because of the error: {}",
            e
        ),
    }
}

pub fn ref_to_oid(working_dir: &Path, name: &String) -> String {
    for path in &["", "refs/", "refs/tags/", "refs/heads/"] {
        let full_path = working_dir.join(init::RUC_DIR).join(path).join(name);

        if full_path.exists() {
            return get_ref(working_dir, &format!("{}{}", path, name));
        }
    }
    name.to_owned()
}

pub fn get_ref(working_dir: &Path, name: &String) -> String {
    let ref_file = working_dir.join(init::RUC_DIR).join(name);

    match std::fs::read_to_string(ref_file) {
        Ok(contents) => contents,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => String::new(),
            _ => {
                println!("fatal: could not get the current value for {}!", name);
                std::process::exit(1);
            }
        },
    }
}

pub fn update_ref(working_dir: &Path, name: &String, commit_id: &String) {
    let ref_file = working_dir.join(init::RUC_DIR).join(name);

    let mut file = fs::File::create(ref_file).unwrap_or_else(|e| {
        println!("could not save {} state: {}", name, e);
        std::process::exit(1);
    });
    file.write_all(commit_id.as_bytes()).unwrap_or_else(|e| {
        println!("could not save {} state: {}", name, e);
        std::process::exit(1);
    });
}

#[derive(Debug)]
pub struct Commit {
    pub id: String,
    pub tree: String,
    pub parent: Option<String>,
    pub contents: String,
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

fn get_commit(id: &String) -> Result<Commit, String> {
    match object::get(id) {
        Ok(obj) => {
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
        Err(e) => Err(format!("could not get commit {}: {}", id, e)),
    }
}

pub fn log(from: &String) {
    let mut id = from.to_owned();

    loop {
        if let Ok(commit) = get_commit(&id) {
            println!("commit {}\n\n{}", commit.id, commit.contents);

            match commit.parent {
                Some(parent) => {
                    id = parent.to_owned();
                    println!();
                }
                None => break,
            }
        }
    }
}

pub fn checkout(id: &String) {
    let working_dir = init::working_dir();
    let res = get_commit(id);

    match res {
        Ok(commit) => {
            tree::read_tree(&commit.tree);
            update_ref(&working_dir, &String::from("HEAD"), id);
        }
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }
}

pub fn create_tag(name: &String, id: &String) {
    let working_dir = init::working_dir();

    update_ref(&working_dir, &format!("refs/tags/{}", name), id);
}
