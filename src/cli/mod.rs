mod cmd;
mod common;
mod dotenvs;
mod postgres_url;
mod render_help;

use clap::command;
use strum::{Display, EnumIter};
pub use {
    cmd::Cmd, common::*, dotenvs::DotEnvs, postgres_url::PostgresUrl, render_help::print_info,
};

#[derive(clap::Parser)]
#[command(arg_required_else_help = true)]
#[command(disable_help_flag = true)]
#[command(disable_help_subcommand = true)]
#[command(author, version, about, long_about = None)]
pub struct GarlicParser {
    #[clap(subcommand)]
    pub command: Option<GarlicCommand>,
    /// Print help
    #[arg(short = 'h', long, global = true)]
    pub help: bool,
}

#[derive(clap::Subcommand, Display, EnumIter)]
pub enum GarlicCommand {
    /// Prints this message
    #[command(name = "help")]
    #[strum(serialize = "help")]
    Help,
    /// Detailed CLI information
    #[command(name = "info")]
    #[strum(serialize = "info")]
    Info,
    /// Sets up a new project
    #[command(name = "init")]
    #[strum(serialize = "init")]
    Init {
        /// If set, the relative location of the project
        location: Option<String>,
    },
    /// Sets up the database for an existing project
    #[command(name = "init:db")]
    #[strum(serialize = "init:db")]
    InitDb,
    /// Tests the backend, runs `cargo spec` to make sure the frontend is in sync and then tests the frontend
    #[command(name = "test:all", visible_aliases=["test"])]
    #[strum(serialize = "test:all")]
    TestAll,
    /// Gets the openapi spec from the api and generates the frontend typescript implementation and routes
    #[command(name = "spec")]
    #[strum(serialize = "spec")]
    Spec,
    /// Exports the functions in export_fns to typescript
    #[command(name = "update:fns", visible_aliases=["fns"])]
    #[strum(serialize = "update:fns")]
    UpdateFns,
    /// builds the frontend and backend.
    #[command(name = "build")]
    #[strum(serialize = "build")]
    Build,

    // -- Wrappers --
    /// <WRAPPER>Wrapper for 'cargo run'
    #[command(name = "run:backend", visible_aliases=["backend", "server", "run:server"])]
    #[strum(serialize = "run:backend")]
    RunBackend {
        /// Pass in arguments for 'cargo run'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// <WRAPPER>Wrapper for 'cargo install garlic-cli'
    #[command(name = "update:self", visible_aliases=["update"])]
    #[strum(serialize = "run:backend")]
    UpdateSelf {
        /// Pass in arguments for 'cargo install garlic-cli'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// <WRAPPER>Wrapper for 'bun x vite dev'
    #[command(name = "run:frontend", visible_aliases=["frontend", "dev"])]
    #[strum(serialize = "run:frontend")]
    RunFrontend {
        /// Pass in arguments for 'bun x vite dev'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// <WRAPPER>Wrapper for 'cargo add <name> --package <package>'
    #[command(name = "add:crate", visible_aliases=["crate"])]
    #[strum(serialize = "add:crate")]
    AddCrate {
        /// The name of the package to add
        name: String,
        /// The name of the target package in the workspace
        package: String,
        /// Pass in arguments for 'cargo add <name> --package <package>'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// <WRAPPER>Wrapper for 'cargo sqlx prepare --workspace'.
    #[command(name = "prepare")]
    #[strum(serialize = "prepare")]
    Prepare {
        /// Pass in arguments for 'cargo sqlx prepare --workspace'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// <WRAPPER>Wrapper for 'bun x vite preview'.
    #[command(name = "preview")]
    #[strum(serialize = "preview")]
    Preview {
        /// Pass in arguments for 'bun x vite preview'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// <WRAPPER>Wrapper for 'bun x vitest'
    #[command(name = "test:unit", visible_aliases=["vitest"])]
    #[strum(serialize = "test:unit")]
    TestUnit {
        /// Pass in arguments for 'bun x vitest'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
    /// <WRAPPER>Wrapper for 'cargo sqlx migrate'.
    #[command(name = "migrate")]
    #[strum(serialize = "migrate")]
    Migrate {
        /// Pass in arguments for 'cargo sqlx migrate'
        #[arg(long, allow_hyphen_values = true, num_args = 0..)]
        args: Vec<String>,
    },
}
