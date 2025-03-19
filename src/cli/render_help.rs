use parser::*;
use std::{env, process::exit};

use clap::CommandFactory;
use colored::Colorize;

use super::{GarlicCommand, GarlicParser};

pub fn print_info() {
    println!(
        "{} v{}",
        env!("CARGO_PKG_NAME").bold().green(),
        env!("CARGO_PKG_VERSION")
    );

    println!();
    println!("{}", "--PACKAGE--".blue());
    println!("Authors: {}", env!("CARGO_PKG_AUTHORS").blue());
    println!("License: {}", env!("CARGO_PKG_LICENSE").blue());
    println!("Description: {}", env!("CARGO_PKG_DESCRIPTION").blue());
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY").blue());

    println!();
    println!("{}", "--BUILD--".blue());
    println!(
        "Location: {}",
        env::current_exe()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .blue()
    );
    println!(
        "Profile: {}",
        if cfg!(debug_assertions) {
            "debug".blue()
        } else {
            "release".blue()
        }
    );
    println!("Target architecture: {}", std::env::consts::ARCH.blue());
    println!("Target OS: {}", std::env::consts::OS.blue());
}

impl GarlicParser {
    pub fn render_help(command: Option<GarlicCommand>) -> ! {
        match command {
            Some(command) => {
                let mut app = GarlicParser::command();
                let command_name = command.to_string();
                let subcmd = app
                    .find_subcommand_mut(&command_name)
                    .expect("Expected subcommand");

                let help = subcmd.render_help().to_string();
                let (name, details) = help.split_once("\n").unzip();

                println!(
                    "{} v{}: {}",
                    env!("CARGO_PKG_NAME").bold().green(),
                    env!("CARGO_PKG_VERSION"),
                    command_name.blue()
                );

                let name = name.unwrap();
                parse_command(&name.replace("<WRAPPER>", ""), true);

                println!();

                let details = details
                    .unwrap()
                    .replace("Usage:", &format!("Usage: {}", env!("CARGO_BIN_NAME")));

                let mut parser = Parser::new(&details);
                parse_lines(&mut parser);

                let aliases: Vec<_> = subcmd.get_visible_aliases().collect();
                if aliases.len() > 0 {
                    println!();
                    print!("{} ", "Aliases:".bold().underline());

                    let mut aliases = aliases.iter();
                    parse_command_alias(aliases.next().unwrap());

                    for alias in aliases {
                        print!(", ");
                        parse_command_alias(alias);
                    }
                    println!();
                }

                exit(0);
            }
            None => {
                let help = GarlicParser::command()
                    .render_help()
                    .to_string()
                    .replace(env!("CARGO_PKG_NAME"), env!("CARGO_BIN_NAME"));

                let (cli_description, details) = help.split_once("\n").unzip();

                println!(
                    "{} v{}",
                    env!("CARGO_PKG_NAME").bold().green(),
                    env!("CARGO_PKG_VERSION")
                );
                parse_command(cli_description.unwrap(), true);
                println!();
                let mut parser = Parser::new(details.unwrap());

                parse_lines(&mut parser);
                // GarlicParser::command().print_help().unwrap();
                exit(0);
            }
        }
    }
}

fn parse_lines(parser: &mut Parser) {
    let mut current = String::new();

    while let Some(ch) = parser.next() {
        match ch {
            ':' if parser.peek().is_some_and(|p| p.is_whitespace()) => {
                current.push(ch);
                match &*current {
                    "Commands:" => {
                        print!("{}", current.bold().underline());
                        current.clear();
                        parse_commands(parser);
                    }
                    "Options:" => {
                        print!("{}", current.bold().underline());
                        current.clear();
                        parse_options(parser);
                    }
                    "Arguments:" => {
                        print!("{}", current.bold().underline());
                        current.clear();
                        parse_arguments(parser);
                    }
                    "Usage:" => {
                        print!("{}", current.bold().underline());
                        current.clear();
                        parse_usage(parser);
                    }
                    _ => {}
                }
            }
            '\n' => {
                current.push(ch);
                print!("{current}");
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    print!("{current}")
}

fn parse_arguments(parser: &mut Parser) {
    let mut line = String::new();
    print!("{}", parser.next().unwrap_or('\n'));

    while let Some(ch) = parser.next() {
        line.push(ch);
        if ch == '\n' {
            if line.chars().all(|c| c.is_whitespace()) {
                break;
            }

            parse_command(&line, false);
            line.clear();
        }
    }

    println!();
}

fn parse_commands(parser: &mut Parser) {
    let mut commands = vec![];

    let mut line = String::new();
    print!("{}", parser.next().unwrap_or('\n'));

    while let Some(ch) = parser.next() {
        line.push(ch);
        if ch == '\n' {
            if line.chars().all(|c| c.is_whitespace()) {
                break;
            }
            commands.push(line.clone());
            line.clear();
        }
    }

    let (non_wrappers, wrappers) = (
        commands.iter().filter(|comm| !comm.contains("<WRAPPER>")),
        commands
            .iter()
            .filter(|comm| comm.contains("<WRAPPER>"))
            .collect::<Vec<_>>(),
    );

    for command in non_wrappers {
        parse_command(command, false);
    }

    if !wrappers.is_empty() {
        println!();
        println!("{}", "Wrapper commands:".bold().underline());

        for command in wrappers {
            parse_command(&command.replace("<WRAPPER>", ""), false);
        }
    }

    print!("{line}");
}

fn parse_command(command: &str, description_only: bool) {
    let mut parser = Parser::new(command);

    let mut current = String::new();
    let mut whitespace = 0;
    let mut is_string = false;
    let mut aliases = false;

    while let Some(ch) = parser.next() {
        if ch == '\'' && !is_string {
            print!("{current}");
            current.clear();
            is_string = true;
            current.push(ch);
            continue;
        }
        current.push(ch);

        if is_string {
            if ch == '\'' {
                print!("{}", current.bold().cyan());
                current.clear();
                is_string = false;
            }
            continue;
        }

        if current.contains("[aliases: ") {
            print!("{current}");
            current.clear();
            aliases = true;
            continue;
        }

        if aliases {
            if current.ends_with(",") {
                _ = current.pop();
                parse_command_alias(&current);
                print!(", ");
                current.clear();
                parser.advance();
            } else if current.ends_with("]") {
                _ = current.pop();
                parse_command_alias(&current);
                print!("]");
                current.clear();
                aliases = false;
            }
            continue;
        }

        if !description_only
            && ch.is_whitespace()
            && parser.peek().is_some_and(|c| !c.is_whitespace())
        {
            whitespace += 1;

            if whitespace == 2 {
                print!("{}", current.blue());
                current.clear();
            }
        }

        if !description_only && whitespace < 2 && parser.peek().is_some_and(|c| c == ':') {
            print!("{}", current.blue());
            current.clear();
            parser.advance();
            print!("{}", ":".bold());
        }

        if current.contains("[<ARGS>...]") {
            print!("{}", current.split("[<ARGS>...]").next().unwrap_or(""));
            print!("[{}]", "<ARGS>...".yellow());
            current.clear();
            continue;
        }
    }

    print!("{current}");
}

fn parse_command_alias(alias: &str) {
    let mut alias_split = alias.split(":");

    print!("{}", alias_split.next().unwrap_or("").blue());

    for split in alias_split {
        print!("{}{}", ":".bold(), split.blue());
    }
}

fn parse_options(parser: &mut Parser) {
    let mut current = String::new();
    let mut second_part = false;

    while let Some(ch) = parser.next() {
        current.push(ch);
        if !second_part
            && parser.peek().is_some_and(|p| p.is_whitespace())
            && current.contains("--")
        {
            print!("{}", current.bold().green());
            current.clear();
            second_part = true;
        }

        if ch == '\n' {
            parse_command(&current, true);
            current.clear();
            second_part = false;
        }
    }

    parse_command(&current, true);
}

fn parse_usage(parser: &mut Parser) {
    let mut current = String::new();

    while let Some(ch) = parser.next() {
        current.push(ch);

        if ch == '\n' {
            break;
        }

        if ch == '[' {
            print!("{current}");
            current.clear();

            while let Some(ch) = parser.next() {
                if ch == ']' {
                    if current == "OPTIONS" {
                        print!("{}", current.green().bold());
                    } else {
                        print!("{}", current.blue());
                    }
                    current.clear();
                    current.push(ch);
                    break;
                }
                current.push(ch);
            }
        } else if ch.is_whitespace() {
            parse_command_alias(&current);
            current.clear();
        } else if current.trim() == env!("CARGO_BIN_NAME") {
            print!("{}", current.bold().green());
            current.clear();
        }
    }

    print!("{current}");
}

mod parser {
    pub struct Parser<'p> {
        string: &'p str,
        cursor: usize,
    }

    impl<'p> Parser<'p> {
        pub fn new(string: &'p str) -> Self {
            Parser { string, cursor: 0 }
        }

        pub fn peek(&self) -> Option<char> {
            self.string.chars().nth(self.cursor)
        }

        pub fn advance(&mut self) {
            self.cursor += 1
        }
    }

    impl<'p> Iterator for Parser<'p> {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            let char = self.peek();
            self.advance();
            char
        }
    }
}
