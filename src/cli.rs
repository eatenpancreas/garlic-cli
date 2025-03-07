use std::{
    ffi::OsStr,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use clap::command;
use colored::Colorize;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: GarlicCommand,
}

#[derive(clap::Subcommand)]
pub enum GarlicCommand {
    Init {
        location: Option<String>,
    },
    /// Runs the backend. aliases = (backend, server)
    #[command(name = "run:backend", aliases=["backend", "server"])]
    RunBackend,
    /// Runs the frontend. aliases = (frontend, dev)
    #[command(name = "run:frontend", aliases=["frontend", "dev"])]
    RunFrontend,
    /// builds the frontend and backend.
    Build,
    Prepare,
    Preview,
    /// aliases = (vitest)
    #[command(name = "test:unit", aliases=["vitest"])]
    TestUnit,
    /// aliases = (test)
    #[command(name = "test:all", aliases=["test"])]
    TestAll,
    Spec,
    Migrate,
}

#[must_use]
pub struct Cmd(Command, String);
impl Cmd {
    pub fn run(command: &str) -> Self {
        let mut args = command.split(" ");
        let cmd = args.next().expect("Expected command to not be empty");
        let mut cmd = Command::new(cmd);
        cmd.args(args);

        Self(cmd, command.to_owned())
    }

    pub fn arg(mut self, arg: impl Display + AsRef<OsStr>) -> Self {
        self.1.extend(format!(" {arg}").chars());
        self.0.arg(arg);
        self
    }

    fn display(&self) {
        println!("{}: Running \"{}\"", "[garlic]".green(), self.1.cyan());
    }

    pub fn req(mut self) {
        self.display();
        match self.0.status() {
            Err(e) => error(e.kind(), e),
            Ok(o) if !o.success() => exit(o.code().unwrap_or(1)),
            _ => {}
        }
    }

    pub fn opt(mut self) {
        self.display();
        match self.0.status() {
            Err(e) => error_opt(e.kind(), e),
            _ => {}
        }
    }

    pub fn ok(mut self) -> bool {
        self.display();
        self.0.status().is_ok_and(|o| o.success())
    }
}

pub fn garlic_print(content: impl Display) {
    println!("{}: {}", "[garlic]".green(), content);
}

pub fn error(kind: impl Display, message: impl Display) {
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
        println!("process: {:?}", &working_path.strip_prefix(from));

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
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
                        println!("  copying {:?}", &path.strip_prefix(from),);
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
