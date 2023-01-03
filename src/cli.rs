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
            ping_command if input.starts_with("ping") => {
                let remote = ping_command
                    .split_whitespace()
                    .nth(1)
                    .expect("Missing remote address to ping.");

                match remote.parse() {
                    Ok(remote) => Some(Command::Ping { remote }),
                    Err(_) => {
                        println!("Could not parse remote address: [{remote}]");
                        Self::print_help();
                        None
                    }
                }
            }
            send_command if input.starts_with("send") => {
                let mut split = send_command.split_whitespace();
                split.next();
                let remote = split
                    .next()
                    .expect("Missing remote address to send message to.");
                let remote = remote.parse().expect("Could not parse remote address");
                let message = split.next().expect("Missing message to send.").to_string();
                return Some(Command::Send { remote, message });
            }
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
        println!("  ping <remote>");
        println!("  send <remote> <message>");
        println!("  info");
    }
}
