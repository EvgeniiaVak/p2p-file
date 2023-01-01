/// These tests assume there is a running instance
/// on the local network
const TEST_PEER: &str = "/ip4/192.168.64.3/tcp/58002";

use assert_cmd::prelude::*;
use rexpect::session::spawn_command;
use std::process::Command;

#[test]
fn ping() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.arg("-p").arg("58003");
    cmd.arg("-r").arg(TEST_PEER);
    let mut p = spawn_command(cmd, Some(30000))?;

    p.exp_string("Listening on")?;
    p.exp_string("Ping")?;
    p.exp_string("Pong")?;

    Ok(())
}
