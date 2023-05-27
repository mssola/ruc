mod init;
mod object;
mod tree;

use clap::{arg, Command};
use std::path::PathBuf;

fn cli() -> Command {
    Command::new("ruc")
        .about("Git-like version control system for learning purposes")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("init")
                .about("Create an empty repository or reinitialize an existing one")
                .arg(
                    arg!([directory] "Directory location")
                        .value_parser(clap::value_parser!(PathBuf))
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("hash-object")
                .about("Hash a given file into the object database")
                .arg(
                    arg!(<file> "File to store into the object database")
                        .value_parser(clap::value_parser!(PathBuf))
                        .required(true),
                ),
        )
        .subcommand(
            // TODO: not really size
            Command::new("cat-file")
                .about("Provide content or type and size information for repository objects")
                .arg(
                    arg!(<object> "The name of the object to show")
                        .value_parser(clap::value_parser!(String))
                        .required(true),
                ),
        )
        .subcommand(Command::new("write-tree").about("Create a tree object from the current index"))
        .subcommand(
            Command::new("read-tree")
                .about("Reads tree information into the index")
                .arg(
                    arg!(<tree> "The name of the tree to read from")
                        .value_parser(clap::value_parser!(String))
                        .required(true),
                ),
        )
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", sm)) => {
            let cur = std::env::current_dir().unwrap();
            let dir = match sm.get_one::<PathBuf>("directory") {
                Some(name) => name,
                None => &cur,
            };
            init::init(dir);
        }
        Some(("hash-object", sm)) => {
            object::hash(
                sm.get_one::<PathBuf>("file").unwrap(),
                object::Kind::Blob,
                true,
            );
        }
        Some(("cat-file", sm)) => {
            object::cat(sm.get_one::<String>("object").unwrap());
        }
        Some(("write-tree", _sm)) => {
            let cur = std::env::current_dir().unwrap();
            tree::write_tree(cur.as_path());
        }
        Some(("read-tree", sm)) => {
            tree::read_tree(sm.get_one::<String>("tree").unwrap());
        }
        Some((command, _)) => {
            println!(
                "ruc: «{}» is not a valid command. See «ruc --help».",
                command
            )
        }
        _ => unreachable!(),
    }
}
