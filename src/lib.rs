use libp2p::Multiaddr;
use std::error::Error;
use tokio::select;

mod cli;
mod network;

pub enum Command {
    Ping { remote: Multiaddr },
    Send { remote: Multiaddr, message: String },
    Info,
    Accept,
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let mut node = network::Node::new()?;
    let mut commands = cli::CommandParser::default();

    cli::CommandParser::print_help();

    loop {
        select! {
            command = commands.next_command() => { node.handle_command(command?)? },
            _ = node.handle_event() => {},
        }
    }
}
