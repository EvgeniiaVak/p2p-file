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
        println!("Dialed {addr}");
    }

    loop {
        tokio::select! {
            _ = handle_user_input() => {},
            _ = handle_network_events(&mut swarm) => {},
        }
    }
}

async fn handle_user_input() -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stdin());

    let mut buffer = String::new();
    let bytes = reader.read_line(&mut buffer).await?;
    let trimmed_input = buffer.trim();

    println!("Read [{bytes}] bytes");
    println!("Read [{trimmed_input}]");

    Ok(())
}

async fn handle_network_events(swarm: &mut Swarm<Behaviour>) -> Result<(), Box<dyn Error>> {
    match swarm.select_next_some().await {
        SwarmEvent::Behaviour(event) => {
            println!("{event:?}");
            Ok(())
        }

        SwarmEvent::NewListenAddr { address, .. } => {
            println!("Listening on {address:?}");
            Ok(())
        }

        SwarmEvent::IncomingConnection {
            local_addr,
            send_back_addr,
        } => {
            println!("Incoming connection from {send_back_addr} at {local_addr}");
            Ok(())
        }

        SwarmEvent::ConnectionEstablished {
            peer_id,
            endpoint,
            num_established,
            concurrent_dial_errors,
        } => {
            println!(
                "Connection established with {peer_id} at {endpoint:?} ({num_established} connections)"

            );
            if let Some(errs) = concurrent_dial_errors {
                println!(" ({errs:?} concurrent dial errors)");
            }
            Ok(())
        }

        other => unimplemented!("Unhandled event: [{other:?}]"),
    }
}

#[derive(NetworkBehaviour, Default)]
struct Behaviour {
    // TODO: substitute keeping alive with something more sensible
    keep_alive: keep_alive::Behaviour,
    ping: ping::Behaviour,
}
