use std::path::Path;
use std::path::PathBuf;

pub const RUC_DIR: &str = ".ruc";

fn iterate_working_dirs(mut dir: PathBuf) -> PathBuf {
    let hd = home::home_dir().unwrap_or(dir.clone());

    while dir != hd && dir.as_os_str() != "/" {
        if dir.join(RUC_DIR).exists() {
            return dir;
        }
        dir = dir.parent().unwrap().to_path_buf();
    }

    println!(
        "fatal: not a ruc repository (or any parent up to mount point {})",
        hd.display()
    );
    std::process::exit(1);
}

// Returns the current working directory, or exits if it does not exist.
pub fn working_dir() -> PathBuf {
    match std::env::current_dir() {
        Ok(dir) => iterate_working_dirs(dir),
        Err(_) => {
            println!("ERROR");
            std::process::exit(1);
        }
    }
}

// Initialize the given directory to be a `ruc` project.
pub fn init(directory: &Path) {
    std::fs::create_dir_all(directory.join(RUC_DIR))
        .unwrap_or_else(|e| panic!("Could not create directory: {}", e));
    std::fs::create_dir_all(directory.join(RUC_DIR).join("objects"))
        .unwrap_or_else(|e| panic!("Could not create directory: {}", e));

    println!(
        "Initialized empty Ruc repository in {}",
        directory.display()
    );
}
