use libp2p::futures::StreamExt;
use libp2p::swarm::{keep_alive, NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{identity, ping, tokio_development_transport, Multiaddr, PeerId};
use std::error::Error;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};

pub async fn run(local_addr: String, remote_addr: Option<String>) -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = tokio_development_transport(local_key)?;
    let mut swarm = Swarm::with_tokio_executor(transport, Behaviour::default(), local_peer_id);
    swarm.listen_on(local_addr.parse()?)?;

    if let Some(addr) = remote_addr {
        let remote: Multiaddr = addr.trim().parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

    let res = tokio::join!(handle_user_input(), handle_network_events(&mut swarm));
    if let (Ok(_), Ok(_)) = res {
        Ok(())
    } else {
        Err(format!("Result: {res:?}").into())
    }
}

async fn handle_user_input() -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stdin());

    loop {
        let mut buffer = String::new();
        let bytes = reader.read_line(&mut buffer).await?;
        let trimmed_input = buffer.trim();

        println!("Read [{bytes}] bytes");
        println!("Read [{trimmed_input}]");
    }
}

async fn handle_network_events(swarm: &mut Swarm<Behaviour>) -> Result<(), Box<dyn Error>> {
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
            _ => {}
        }
    }
}

#[derive(NetworkBehaviour, Default)]
struct Behaviour {
    keep_alive: keep_alive::Behaviour,
    ping: ping::Behaviour,
}
