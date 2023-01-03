use assert_cmd::prelude::*;
use rexpect::session::{spawn_command, PtySession};
use std::error::Error;
use std::process::Command;

struct Peer {
    session: PtySession,
    address: String,
}

impl Peer {
    fn launch() -> Result<Peer, Box<dyn Error>> {
        let cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        let mut session = spawn_command(cmd, Some(30000))?;

        session.exp_string("PeerId")?;
        session.exp_string("Listening on")?;
        let mut address = session.exp_string("Listening on")?;
        address = address.replace("\"", "").trim().to_string();

        println!("Launched peer with address: {}", address);
        Ok(Peer { session, address })
    }
}

#[test]
fn ping() -> Result<(), Box<dyn Error>> {
    let listening_peer = Peer::launch()?;
    let mut pinning_peer = Peer::launch()?;

    let command = format!("ping {}", listening_peer.address.as_str());

    pinning_peer.session.send_line(&command)?;

    pinning_peer.session.exp_string("Ping")?;
    pinning_peer.session.exp_string("Pong")?;

    Ok(())
}
