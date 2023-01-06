use assert_cmd::prelude::*;
use rexpect::session::{spawn_command, PtySession};
use std::error::Error;
use std::process::Command;

struct Peer {
    cli: PtySession,
    address: String,
}

impl Peer {
    fn launch() -> Result<Peer, Box<dyn Error>> {
        let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let mut session = spawn_command(cmd, Some(30000))?;

        session.exp_string("Available commands:")?;
        session.exp_string("Listening on:")?;
        session.send_line("info")?;
        let mut address = session.exp_string("connect ")?;
        address = address.trim().to_string();

        println!("Launched peer with address: {}", address);
        Ok(Peer {
            cli: session,
            address,
        })
    }
}

#[test]
fn connect() -> Result<(), Box<dyn Error>> {
    let a = Peer::launch()?;
    let mut b = Peer::launch()?;

    let command = format!("connect {}", a.address.as_str());
    b.cli.send_line(&command)?;
    b.cli.exp_string("Connected")?;

    Ok(())
}

#[test]
fn send() -> Result<(), Box<dyn Error>> {
    let mut a = Peer::launch()?;
    let mut b = Peer::launch()?;

    let command = format!("connect {}", a.address.as_str());

    b.cli.send_line(&command)?;

    b.cli.exp_string("Connected")?;
    b.cli.send_line("send hello")?;
    a.cli.exp_string("hello")?;

    Ok(())
}
