use std::{
    io,
    process::{exit, Command, ExitStatus},
};

use clap::command;
use colored::Colorize;

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Cmd,
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    Init,
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

pub fn command(command: &str) {
    let mut args = command.split(" ");
    let cmd = args.next().expect("Expected command to not be empty");

    println!("{}: Running \"{}\"", "[garlic]".green(), command.cyan());

    let mut cmd = Command::new(cmd);
    cmd.args(args);

    print_status(cmd.status());
}

fn print_status(status: io::Result<ExitStatus>) {
    match status {
        Err(e) => {
            println!(
                "{} (type {}): {}",
                "Error".red(),
                e.kind().to_string().cyan(),
                e.to_string().red()
            );

            exit(1)
        }
        Ok(o) if !o.success() => exit(o.code().unwrap_or(1)),
        _ => {}
    }
}
