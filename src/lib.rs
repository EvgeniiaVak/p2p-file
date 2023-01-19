use libp2p::Multiaddr;
use std::error::Error;
use tokio::select;

mod cli;
mod network;

pub enum Command {
    Connect { remote: Multiaddr },
    Info,
    Request { file_path: String },
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let mut node = network::Node::new()?;
    let mut commands = cli::CommandParser::default();

    cli::CommandParser::print_help();

    loop {
        select! {
            command = commands.next_command() => {
                let output = node.handle_command(command?)?;
                if let Some(output) = output {
                    cli::show_output(output);
                }
            },
            output = node.handle_event() => { cli::show_output(output?) },
        }
    }
}
