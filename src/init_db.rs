use crate::{
    error_opt, find_dotgarlic_directory, garlic_print, random_jwt_secret, DotEnvs, PostgresUrl,
};
use dialoguer::{Confirm, Input};
use postgres::{Client, NoTls};
use std::{env::current_dir, fs::copy};

pub fn init_db_inner() {
    // setup
    let current_dir =
        find_dotgarlic_directory().unwrap_or(current_dir().expect("Expected a current directory"));

    if !current_dir.join(".env").exists() {
        copy(current_dir.join(".env.example"), current_dir.join(".env"))
            .expect("Expected to be able to copy to .env");
    }

    let mut envs = DotEnvs::load(current_dir.join(".env")).expect("Expected envs");
    let example_db_string = DotEnvs::load(current_dir.join(".env.example"))
        .expect("Expected envs")
        .remove("DATABASE_URL")
        .expect("Expected DATABASE_URL in .env.example");

    if envs
        .get("JWT_SECRET")
        .is_none_or(|secret| secret.is_empty())
    {
        envs.insert("JWT_SECRET".to_owned(), random_jwt_secret());
    }

    envs.save(current_dir.join(".env"))
        .expect("Expected to be able to save");

    let mut database_url = envs
        .remove("DATABASE_URL")
        .unwrap_or(example_db_string.clone());

    if database_url.is_empty() {
        database_url = example_db_string.clone()
    }

    let mut url = PostgresUrl::parse_env_connection_string(&database_url);
    if database_url == example_db_string
        || Confirm::new()
            .with_prompt("Connection string is already present. Redo?")
            .interact()
            .expect("Expected interaction")
    {
        url.username = Input::new()
            .with_prompt("Username?")
            .default("postgres".to_owned())
            .interact_text()
            .expect("Expected interaction");

        url.password = Input::new()
            .with_prompt("Password?")
            .default("".to_owned())
            .interact_text()
            .ok();

        url.host = Input::new()
            .with_prompt("Hostname?")
            .default("localhost".to_owned())
            .interact_text()
            .expect("Expected interaction");

        url.port = Input::new()
            .with_prompt("Port?")
            .default(5432)
            .interact_text()
            .expect("Expected interaction");

        url.port = Input::new()
            .with_prompt("Database name?")
            .interact_text()
            .expect("Expected interaction");

        envs.insert("DATABASE_URL".to_owned(), url.to_env_connection_string());

        envs.save(current_dir.join(".env"))
            .expect("Expected to be able to save");
    }

    if !setup_postgres_database(url) {
        garlic_print("You'll have to set-up postgres first and then run this command again.");
        garlic_print("\x1b]8;;https://www.postgresql.org/download\x1b\\https://www.postgresql.org/download\x1b]8;;\x1b\\");
    }
}

pub fn setup_postgres_database(url: PostgresUrl) -> bool {
    if url.is_running() {
        match Client::connect(&url.to_connection_params(), NoTls) {
            Ok(mut client) => {
                let rows = client
                    .query_opt(
                        "SELECT 1 FROM pg_database WHERE datname = $1",
                        &[&url.database],
                    )
                    .expect("expected to be able to run query");

                if rows.is_none() {
                    if Confirm::new()
                        .with_prompt(format!(
                            "Database {} is not present. Create it?",
                            url.database
                        ))
                        .interact()
                        .expect("Expected interaction")
                    {
                        let create_db_query = format!("CREATE DATABASE {}", url.database);
                        client
                            .execute(&create_db_query, &[])
                            .expect("Expected to be able to create non-existing database");

                        garlic_print(format!("Created {}", url.database));
                        true
                    } else {
                        garlic_print(format!("You'll have to manually create the database."));
                        true
                    }
                } else {
                    garlic_print(format!("Database {} already exists.", url.database));
                    true
                }
            }
            Err(e) => {
                error_opt("no_connection", format!("Couldn't connect to client: {e}"));
                false
            }
        }
    } else {
        error_opt(
            "not_detected",
            format!("Postgres not detected at {}:{}.", url.host, url.port),
        );
        false
    }
}
