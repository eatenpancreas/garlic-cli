use super::{error, error_opt, find_dotgarlic_directory};
use colored::Colorize;
use std::{
    env::{self, current_dir},
    ffi::OsStr,
    fmt::Display,
    path::PathBuf,
    process::{exit, Command as StdCommand},
};

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

    pub fn app(self) -> Self {
        self.push_path("app")
    }

    pub fn export_fns(self) -> Self {
        self.push_path("export_fns")
    }

    fn push_path(mut self, dir: &str) -> Self {
        self.inner.current_dir(
            self.inner
                .get_current_dir()
                .expect("Expected to get a current working directory when moving to app")
                .join(dir),
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
