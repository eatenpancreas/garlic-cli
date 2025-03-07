use std::{
    env::{self, current_dir},
    ffi::OsStr,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::{exit, Command as StdCommand},
};

use clap::command;
use colored::Colorize;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
#[command(author, version, about, long_about = None)]
pub struct GarlicParser {
    #[clap(subcommand)]
    pub command: GarlicCommand,
}

#[derive(clap::Subcommand)]
pub enum GarlicCommand {
    Init {
        location: Option<String>,
    },
    /// Tests the backend, runs `cargo spec` to make sure the frontend is in sync and then tests the frontend. aliases = (test)
    #[command(name = "test:all", aliases=["test"])]
    TestAll,
    /// Gets the openapi spec from the api and generates the frontend typescript implementation and routes
    Spec,
    /// builds the frontend and backend.
    #[command(next_help_heading = "WRAPPER COMMANDS")]
    Build,

    //--- wrappers
    /// Wrapper for 'cargo run'. aliases = (backend, server)
    #[command(name = "run:backend", aliases=["backend", "server"])]
    RunBackend {
        /// Pass in arguments for 'cargo run'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Wrapper for 'bun x vite dev'. aliases = (frontend, dev)
    #[command(name = "run:frontend", aliases=["frontend", "dev"])]
    RunFrontend {
        /// Pass in arguments for 'bun x vite dev'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Wrapper for 'cargo sqlx prepare --workspace'.
    Prepare {
        /// Pass in arguments for 'cargo sqlx prepare --workspace'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Wrapper for 'bun x vite preview'.
    Preview {
        /// Pass in arguments for 'bun x vite preview'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Wrapper for 'bun x vitest'. aliases = (vitest)
    #[command(name = "test:unit", aliases=["vitest"])]
    TestUnit {
        /// Pass in arguments for bun x vitest'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// Wrapper for 'cargo sqlx migrate'.
    Migrate {
        /// Pass in arguments for 'cargo sqlx migrate'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
}

#[must_use]
pub struct Cmd {
    inner: StdCommand,
    display: String,
    return_dir: PathBuf,
}

impl Cmd {
    pub fn run(command: impl AsRef<str>) -> Self {
        let command = command.as_ref();
        let mut args = command.split(" ");
        let return_dir = current_dir().expect("Expected to be in a valid directory");

        let cmd = args.next().expect("Expected command to not be empty");
        let dir = match find_dotgarlic_directory() {
            Some(dir) => dir,
            None => return_dir.clone(),
        };

        let mut cmd = StdCommand::new(cmd);
        cmd.args(args).current_dir(dir);

        Self {
            inner: cmd,
            display: command.to_owned(),
            return_dir,
        }
    }

    pub fn arg(mut self, arg: impl Display + AsRef<OsStr>) -> Self {
        self.display.extend(format!(" {arg}").chars());
        self.inner.arg(arg);
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Display + AsRef<OsStr>,
    {
        let args = args.into_iter().map(|arg| {
            self.display.extend(format!(" {arg}").chars());
            arg
        });

        self.inner.args(args);

        self
    }

    fn display(&self) {
        println!(
            "{}: Running \"{}\"",
            "[garlic]".green(),
            self.display.cyan(),
        );
    }

    pub fn app(mut self) -> Self {
        self.inner.current_dir(
            self.inner
                .get_current_dir()
                .expect("Expected to get a current working directory when moving to app")
                .join("app"),
        );
        self
    }

    pub fn req(mut self) {
        self.display();
        let status = self.inner.status();
        self.return_to_dir();

        match status {
            Err(e) => error(e.kind(), e),
            Ok(o) if !o.success() => exit(o.code().unwrap_or(1)),
            _ => {}
        }
    }

    pub fn opt(mut self) {
        self.display();
        match self.inner.status() {
            Err(e) => error_opt(e.kind(), e),
            _ => {}
        }

        self.return_to_dir();
    }

    pub fn ok(mut self) -> bool {
        self.display();
        let ok = self.inner.status().is_ok_and(|o| o.success());

        self.return_to_dir();
        ok
    }

    fn return_to_dir(&self) {
        match env::set_current_dir(&self.return_dir) {
            Ok(_) => {}
            Err(e) => panic!("Failed to change directory: {}", e),
        }
    }
}

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
