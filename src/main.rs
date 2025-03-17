mod cli;
mod init_db;
#[cfg(test)]
mod tests;

use clap::Parser;
pub use cli::*;
use dialoguer::{Confirm, Input};
use std::{
    fs::{copy, remove_dir_all},
    path::Path,
};
use tempdir::TempDir;
pub use GarlicCommand as Cc;

fn main() {
    let garlic = GarlicParser::parse();

    match &garlic.command {
        Cc::Init { .. } => { /* init command, we don't expect a .garlic at this point */ }
        _ => {
            if find_dotgarlic_directory().is_none() {
                error_opt("no_dotgarlic", ".garlic File not found. This file is used as an anchor for projects so you can run garlic commands in subdirectories.");
            }
        }
    }

    match garlic.command {
        Cc::Init { location } => {
            let location = Path::new(location.as_deref().unwrap_or("."));
            init_inner();
            if !folder_empty(location)
                && !Confirm::new()
                    .with_prompt("Folder is not empty. Continue?")
                    .interact()
                    .expect("Expected interaction")
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

            copy(location.join(".env.example"), location.join(".env"))
                .expect("Expected to be able to copy to .env");

            let mut envs = DotEnvs::load(location.join(".env")).expect("Expected envs");

            let app_name = Input::new()
                .with_prompt("OpenAPI name?")
                .default("MyGarlic".to_owned())
                .interact()
                .expect("Expected interaction");

            *envs
                .get_mut("OPENAPI_TITLE")
                .expect("Expected OPENAPI_TITLE in .env") = app_name;

            envs.save(location.join(".env"))
                .expect("Expected to be able to save");

            if Confirm::new()
                .with_prompt(
                    "Set up database now? (Requires that postgres is running & can be done later)",
                )
                .interact()
                .expect("Expected interaction")
            {
                init_db::init_db_inner();
            } else {
                envs.insert("JWT_SECRET".to_owned(), random_jwt_secret());

                envs.save(location.join(".env"))
                    .expect("Expected to be able to save");

                garlic_print("Make sure to setup your postgres instance");
                garlic_print("And set your .env's DATABASE_URL manually OR run garlic init:db");
            }

            garlic_print("🧄 Done!");

            println!();
            garlic_print("Run `garlic server` to start the server!");
            garlic_print("Run `garlic dev --open` to run and open the site!");
        }
        Cc::InitDb => {
            init_db::init_db_inner();
        }
        Cc::RunBackend { args } => Cmd::run("cargo run").args(args).req(),
        Cc::UpdateSelf { args } => Cmd::run("cargo install garlic-cli").args(args).req(),
        Cc::RunFrontend { args } => Cmd::run("bun x vite dev").app().args(args).req(),
        Cc::Build => {
            Cmd::run("bun x vite build").app().req();
            Cmd::run("cargo build --release").req()
        }
        Cc::AddCrate {
            name,
            package,
            args,
        } => Cmd::run("cargo add")
            .arg(name)
            .arg("--package")
            .arg(package)
            .args(args)
            .req(),
        Cc::Prepare { args } => Cmd::run("cargo sqlx prepare --workspace").args(args).req(),
        Cc::Preview { args } => Cmd::run("bun x vite preview").app().args(args).req(),
        Cc::TestUnit { args } => Cmd::run("bun x vitest").app().args(args).req(),
        Cc::TestAll => {
            Cmd::run("cargo test").req();
            export_fns();
            spec_get();
            Cmd::run("bun x vitest --run").app().req()
        }
        Cc::UpdateFns => export_fns(),
        Cc::Spec => {
            Cmd::run("cargo set-version --bump patch --package server").req();

            Cmd::run("cargo test test_load_spec").req();
            export_fns();
            spec_get();
            Cmd::run("bun x vitest spec --run").app().req()
        }
        Cc::Migrate { args } => Cmd::run("cargo sqlx migrate").args(args).req(),
    }
}

fn spec_get() {
    Cmd::run("bun x openapi-zod-client ./spec.yml -o ./app/src/lib/gen/client.ts").req();
}

fn export_fns() {
    Cmd::run("wasm-pack build -d ../app/src/lib/gen/export_fns --no-pack")
        .export_fns()
        .req();
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
