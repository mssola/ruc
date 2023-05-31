use ruc::{commit, init, object, tree};

use anyhow::{bail, Result};

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
        .subcommand(
            Command::new("commit")
                .about("Record changes to the repository")
                .arg(
                    arg!(-m --message <message>)
                        .value_parser(clap::value_parser!(String))
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("log").about("Show commit logs").arg(
                arg!(-f --from <revision>)
                    .value_parser(clap::value_parser!(String))
                    .required(false),
            ),
        )
        .subcommand(
            // TODO: not branches, right now :)
            Command::new("checkout")
                .about("Switch branches or restore working tree files")
                .arg(
                    arg!(<commit> "The commit ID where to move")
                        .value_parser(clap::value_parser!(String))
                        .required(true),
                ),
        )
        .subcommand(
            // TODO: yeah, there's plenty to do here ;)
            Command::new("tag")
                .about("Create, list or delete a tag object")
                .subcommand(
                    Command::new("-a")
                        .about("Make an unsigned annotated tag")
                        .arg(
                            arg!(<name> "Name of the tag")
                                .value_parser(clap::value_parser!(String))
                                .required(true),
                        )
                        .arg(
                            arg!([commit] "Commit ID to tag")
                                .value_parser(clap::value_parser!(String))
                                .default_value("HEAD")
                                .required(false),
                        ),
                ),
        )
        .subcommand(Command::new("graph").about("Show a graph with the history of the repository"))
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", sm)) => {
            let cur = std::env::current_dir().unwrap();
            let dir = match sm.get_one::<PathBuf>("directory") {
                Some(name) => name,
                None => &cur,
            };
            init::init(dir)?;
        }
        Some(("hash-object", sm)) => {
            object::hash(
                sm.get_one::<PathBuf>("file").unwrap(),
                object::Kind::Blob,
                true,
            )?;
        }
        Some(("cat-file", sm)) => {
            let oid = commit::ref_to_oid(sm.get_one::<String>("object").unwrap())?;

            object::cat(&oid)?;
        }
        Some(("write-tree", _sm)) => {
            let cur = std::env::current_dir().unwrap();
            tree::write_tree(cur.as_path())?;
        }
        Some(("read-tree", sm)) => {
            let oid = commit::ref_to_oid(sm.get_one::<String>("tree").unwrap())?;

            tree::read_tree(&oid)?;
        }
        Some(("commit", sm)) => {
            let message = match sm.get_one::<String>("message") {
                Some(v) => v.to_owned(),
                None => commit::editor()?,
            };

            commit::commit(message)?;
        }
        Some(("log", sm)) => {
            let revision = match sm.get_one::<String>("from") {
                Some(v) => commit::ref_to_oid(v)?,
                None => {
                    let revision = commit::get_ref(&String::from("HEAD"))?;

                    if revision.is_empty() {
                        bail!("current branch has no commit yet");
                    }

                    revision
                }
            };

            commit::log(&revision)?;
        }
        Some(("checkout", sm)) => {
            let oid = commit::ref_to_oid(sm.get_one::<String>("commit").unwrap())?;

            commit::checkout(&oid)?;
        }
        Some(("tag", sub)) => match sub.subcommand() {
            Some(("-a", sm)) => {
                let commit = commit::ref_to_oid(sm.get_one::<String>("commit").unwrap())?;

                commit::create_tag(sm.get_one::<String>("name").unwrap(), &commit)?;
            }
            Some((command, _)) => {
                println!("unknown option {} for the 'tag' command", command);
                std::process::exit(1);
            }
            _ => unreachable!(),
        },
        Some(("graph", _sm)) => {
            commit::graph()?;
        }
        Some((command, _)) => {
            println!(
                "ruc: «{}» is not a valid command. See «ruc --help».",
                command
            )
        }
        _ => unreachable!(),
    }

    Ok(())
}
