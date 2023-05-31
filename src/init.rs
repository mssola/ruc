use std::path::Path;
use std::path::PathBuf;

pub const RUC_DIR: &str = ".ruc";

lazy_static! {
    // Lazily initialized static variable to be used in order to fetch the
    // current working directory. It is guaranteed to be initialized, otherwise
    // it will panic.
    pub static ref WORKING_DIR: PathBuf = {
        match std::env::current_dir() {
            Ok(dir) => match iterate_working_dirs(dir) {
                Ok(v) => v,
                Err(e) => {
                    println!("fatal: {}", e);
                    std::process::exit(1);
                }
            },
            Err(_) => {
                println!("fatal: could not get current directory!");
                std::process::exit(1);
            }
        }
    };
}

// Traverse from the current directory up to the home directory in order to
// check for the existence of the `RUC_DIR`. It will return the full path of the
// first match, otherwise it will return an error.
fn iterate_working_dirs(mut dir: PathBuf) -> Result<PathBuf, String> {
    let hd = home::home_dir().unwrap_or(dir.clone());

    while dir != hd && dir.as_os_str() != "/" {
        if dir.join(RUC_DIR).exists() {
            return Ok(dir);
        }
        dir = dir.parent().unwrap().to_path_buf();
    }

    Err(format!(
        "not a ruc repository (or any parent up to mount point {})",
        hd.display()
    ))
}

// Initialize the given directory to be a `ruc` project.
pub fn init(directory: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(directory.join(RUC_DIR))?;
    std::fs::create_dir_all(directory.join(RUC_DIR).join("objects"))?;
    std::fs::create_dir_all(directory.join(RUC_DIR).join("refs").join("tags"))?;
    std::fs::create_dir_all(directory.join(RUC_DIR).join("refs").join("heads"))?;

    println!(
        "Initialized empty Ruc repository in {}",
        directory.display()
    );

    Ok(())
}
