use clap::Parser;
use p2p_file::run;
use std::error::Error;

/// Ping a peer
#[derive(Parser)]
struct Args {
    /// Local interface to listen on
    #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/0")]
    local: String,

    /// Remote peer multiaddr to ping
    /// e.g. /ip4/192.168.64.3/tcp/58002
    #[arg(short, long)]
    remote: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    run(args.local, args.remote).await
}
