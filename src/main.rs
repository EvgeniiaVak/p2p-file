use clap::Parser;
use p2p_file::peer_ping;
use std::error::Error;

/// Ping a peer
#[derive(Parser)]
struct Args {
    /// Local port to listen on
    #[arg(short, long, default_value_t = 58002)]
    port: u32,

    /// Remote peer multiaddr to dial
    #[arg(short, long)]
    remote: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let port = args.port;

    let local_addr = format!("/ip4/0.0.0.0/tcp/{port:?}");
    let remote_addr = args.remote;

    peer_ping(local_addr, remote_addr).await
}
