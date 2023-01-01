use assert_cmd::prelude::*;
use rexpect::session::{spawn_command, PtySession};
use std::error::Error;
use std::process::Command;

struct Peer {
    session: PtySession,
    address: String,
}

impl Peer {
    fn launch(remote: Option<String>) -> Result<Peer, Box<dyn Error>> {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        if let Some(r) = remote {
            cmd.arg("-r").arg(r);
        }
        let mut session = spawn_command(cmd, Some(30000))?;

        session.exp_string("PeerId")?;
        session.exp_string("Listening on")?;
        let mut address = session.exp_string("Listening on")?;
        address = address.replace("\"", "");
        Ok(Peer { session, address })
    }
}

#[test]
fn ping() -> Result<(), Box<dyn Error>> {
    let listening_peer = Peer::launch(None)?;
    let mut pinning_peer = Peer::launch(Some(listening_peer.address))?;

    pinning_peer.session.exp_string("Ping")?;
    pinning_peer.session.exp_string("Pong")?;

    Ok(())
}

#[test]
fn trim_remote() -> Result<(), Box<dyn Error>> {
    let listening_peer = Peer::launch(None)?;

    let addr = format!("\n{} ", listening_peer.address);
    let mut pinning_peer = Peer::launch(Some(addr))?;

    pinning_peer.session.exp_string("Ping")?;
    pinning_peer.session.exp_string("Pong")?;

    Ok(())
}
