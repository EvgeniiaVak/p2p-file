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
            ping_command if input.starts_with("connect") => {
                let remote = ping_command
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
            send_command if input.starts_with("send") => match send_command.split_once(" ") {
                Some((_, message)) => Some(Command::Send {
                    message: message.to_string(),
                }),
                None => {
                    println!("Unexpected command.");
                    Self::print_help();
                    return None;
                }
            },
            "info" => {
                return Some(Command::Info);
            }
            "y" => {
                // TODO: do not send accept command without a pending request
                return Some(Command::Accept);
            }

            unknown => {
                println!("\nUnknown command: {unknown}");
                Self::print_help();
                return None;
            }
        }
    }

    pub fn print_help() {
        println!("Available commands:");
        println!("  connect <remote>");
        println!("  send <message>");
        println!("  info");
    }
}

pub fn show_output(output: impl std::fmt::Display) {
    println!("{output}");
}
