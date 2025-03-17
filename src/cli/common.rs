use colored::Colorize;
use rand::{distr::Alphanumeric, rng, Rng};
use std::{
    env::{self},
    fmt::Display,
    fs::{self},
    path::{Path, PathBuf},
    process::exit,
};

pub fn garlic_print(content: impl Display) {
    println!("{}: {}", "[garlic]".green(), content);
}

pub fn error(kind: impl Display, message: impl Display) -> ! {
    println!(
        "{} (type {}): {}",
        "Error".red(),
        kind.to_string().cyan(),
        message.to_string().red()
    );

    exit(1)
}

pub fn error_opt(kind: impl Display, message: impl Display) {
    println!(
        "{} (type {}): {}",
        "Error".red(),
        kind.to_string().cyan(),
        message.to_string().red()
    );
}

pub fn folder_empty<S: AsRef<Path>>(location: S) -> bool {
    if !location.as_ref().is_dir() {
        return false;
    }

    let entries = fs::read_dir(location).expect("Expected a folder");
    return entries.count() == 0;
}

pub fn random_jwt_secret() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub fn copy_dir_contents(
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    let from = from.as_ref();
    stack.push(PathBuf::from(from));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        garlic_print(format!("copying {:?}", &path.strip_prefix(from).unwrap()));
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn find_dotgarlic_directory() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().ok()?;

    loop {
        let garlic_path = current_dir.join(".garlic");
        if garlic_path.exists() {
            return Some(current_dir);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            return None;
        }
    }
}
