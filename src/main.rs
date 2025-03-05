mod cli;

use clap::Parser;
pub use cli::*;

fn main() {
    match Cli::parse().command {
        Cmd::Init => command("cargo install cargo-edit"),
        Cmd::RunBackend => command("cargo run"),
        Cmd::RunFrontend => command("bun x vite dev"),
        Cmd::Build => {
            command("bun x vite build");
            command("cargo build --release")
        }
        Cmd::Prepare => command("cargo sqlx prepare --workspace"),
        Cmd::Preview => command("bun x vite preview"),
        Cmd::TestUnit => command("bun x vitest"),
        Cmd::TestAll => {
            command("cargo test");
            command(SPEC_GET);
            command("bun x vitest --run")
        }
        Cmd::Spec => {
            command("cargo set-version --bump patch --package server");

            command("cargo test test_load_spec");
            command(SPEC_GET);
            command("vitest spec --run")
        }
        Cmd::Migrate => command("cargo sqlx migrate"),
    }
}

const SPEC_GET: &str = "bun openapi-zod-client spec.yml -o ./src/lib/gen/client.ts";
