mod cmd;
mod common;
mod dotenvs;
mod postgres_url;

use clap::command;
pub use {cmd::Cmd, common::*, dotenvs::DotEnvs, postgres_url::PostgresUrl};

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
    /// Sets up the database for an existing project
    #[command(name = "init:db")]
    InitDb,
    /// Tests the backend, runs `cargo spec` to make sure the frontend is in sync and then tests the frontend. aliases = (test)
    #[command(name = "test:all", aliases=["test"])]
    TestAll,
    /// Gets the openapi spec from the api and generates the frontend typescript implementation and routes
    Spec,
    /// Exports the functions in export_fns to typescript. aliases = (fns)
    #[command(name = "update:fns", aliases=["fns"])]
    UpdateFns,
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
    /// Wrapper for 'cargo install garlic-cli'. aliases = (update)
    #[command(name = "update:self", aliases=["update"])]
    UpdateSelf {
        /// Pass in arguments for 'cargo install garlic-cli'
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
    /// Wrapper for cargo add <name> --package <package>. aliases = (crate)
    #[command(name = "add:crate", aliases=["crate"])]
    AddCrate {
        /// The name of the package to add
        name: String,
        /// The name of the target package in the workspace
        package: String,
        /// Pass in arguments for 'cargo add <name> --package <package>'
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
