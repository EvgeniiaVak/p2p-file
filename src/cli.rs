use std::error::Error;
use tokio::io::{stdin, AsyncBufReadExt, BufReader, Lines, Stdin};

use crate::Command;

pub struct CommandParser {
    reader: Lines<BufReader<Stdin>>,
}

impl Default for CommandParser {
    fn default() -> Self {
        Self {
            reader: BufReader::new(stdin()).lines(),
        }
    }
}

impl CommandParser {
    pub async fn next_command(&mut self) -> Result<Command, Box<dyn Error>> {
        loop {
            if let Some(input) = self.reader.next_line().await? {
                if let Some(command) = Self::parse(input) {
                    return Ok(command);
                }
            }
        }
    }

    fn parse(input: String) -> Option<Command> {
        match input.trim() {
            connect_command if input.starts_with("connect") => {
                let remote = connect_command
                    .split_whitespace()
                    .nth(1)
                    .expect("Missing remote address to ping.");

                match remote.parse() {
                    Ok(remote) => Some(Command::Connect { remote }),
                    Err(_) => {
                        println!("Could not parse remote address: [{remote}]");
                        Self::print_help();
                        None
                    }
                }
            }

            request_command if input.starts_with("request") => {
                let file_path = request_command
                    .split_once(' ')
                    .expect("Missing file path")
                    .1;

                Some(Command::Request {
                    file_path: file_path.to_string(),
                })
            }

            "info" => Some(Command::Info),

            unknown => {
                println!("\nUnknown command: {unknown}");
                Self::print_help();
                None
            }
        }
    }

    // TODO: make a single source of truth for commands somewhere
    pub fn print_help() {
        println!("Available commands:");
        println!("  connect <remote>");
        println!("  request <message>");
        println!("  info");
    }
}

pub fn show_output(output: impl std::fmt::Display) {
    println!("{output}");
}
