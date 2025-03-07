mod cli;

use std::fs::{copy, remove_dir_all, File};

use clap::Parser;
pub use cli::*;
use dialoguer::Confirm;
use tempdir::TempDir;
pub use GarlicCommand as Cc;

const SPEC_GET: &str = "bun x openapi-zod-client spec.yml -o ./app/src/lib/gen/client.ts";
const SQLX_PREPARE: &str = "cargo sqlx prepare --workspace";

fn main() {
    match GarlicParser::parse().command {
        Cc::Init { location } => {
            let location = location.as_deref().unwrap_or(".");
            init_inner();
            if !folder_empty(location)
                && !Confirm::new()
                    .with_prompt("Folder is not empty. Continue?")
                    .interact()
                    .unwrap_or(false)
            {
                error("not_empty", "Folder is not empty. Exiting");
            }
            let tempdir = TempDir::new("garlic-init")
                .expect("Expected to be able to create temporary directory");

            let temp_str = tempdir.path().to_str().expect("expected utf-8 path");

            Cmd::run("git clone https://github.com/eatenpancreas/garlic.git")
                .arg(temp_str)
                .req();

            remove_dir_all(format!("{temp_str}/.git")).expect("Expected to remove original .git");

            copy_dir_contents(temp_str, location).expect("Expected to be able to clone directory");

            Cmd::run("git init").req();
            Cmd::run("bun install").app().req();

            copy(
                format!("{location}/.env.example"),
                format!("{location}/.env"),
            )
            .expect("Expected to be able to copy to .env");

            File::create(format!("{location}/.garlic"))
                .expect("Expected to be able to create a .garlic file");

            garlic_print("🧄 Done!");
            garlic_print("Setup your database and set your .env's DATABASE_URL");
            garlic_print("And make sure to set the JWT_SECRET as well");
            println!();
            garlic_print("Then...");
            garlic_print("Run `garlic server` to start the server!");
            garlic_print("Run `garlic dev --open` to run and open the site!");
        }
        Cc::RunBackend { args } => Cmd::run("cargo run").args(args).req(),
        Cc::RunFrontend { args } => Cmd::run("bun x vite dev").app().args(args).req(),
        Cc::Build => {
            Cmd::run("bun x vite build").app().req();
            Cmd::run("cargo build --release").req()
        }
        Cc::Prepare { args } => Cmd::run(SQLX_PREPARE).args(args).req(),
        Cc::Preview { args } => Cmd::run("bun x vite preview").app().args(args).req(),
        Cc::TestUnit { args } => Cmd::run("bun x vitest").app().args(args).req(),
        Cc::TestAll => {
            Cmd::run("cargo test").req();
            Cmd::run(SPEC_GET).req();
            Cmd::run("bun x vitest --run").app().req()
        }
        Cc::Spec => {
            Cmd::run("cargo set-version --bump patch --package server").req();

            Cmd::run("cargo test test_load_spec").req();
            Cmd::run(SPEC_GET).req();
            Cmd::run("bun x vitest spec --run").app().req()
        }
        Cc::Migrate { args } => Cmd::run("cargo sqlx migrate").args(args).req(),
    }
}

fn init_inner() {
    if !Cmd::run("bun --version").ok() {
        error(
            "missing_install",
            "Bun is not installed or is not in your environment",
        );
    }
    if !Cmd::run("git --version").ok() {
        error(
            "missing_install",
            "Git is not installed or is not in your environment",
        );
    }
    if !Cmd::run("cargo --version").ok() {
        error(
            "missing_install",
            "Cargo is not installed or is not in your environment",
        );
    }

    if !Cmd::run("cargo set-version --version").ok() {
        Cmd::run("cargo install cargo-edit").opt();
    }

    if !Cmd::run("cargo sqlx --version").ok() {
        Cmd::run("cargo install sqlx-cli --features postgres").opt();
    }
}
