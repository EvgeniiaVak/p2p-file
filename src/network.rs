use libp2p::futures::StreamExt;
use std::error::Error;

use libp2p::{Swarm,  ping, identity, PeerId, tokio_development_transport};
use libp2p::swarm::{NetworkBehaviour, keep_alive, SwarmEvent};

use crate::Command;

pub struct Node {
    swarm: Swarm<Behaviour>,
}

impl Node {
    pub fn new() -> Result<Node, Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let id = PeerId::from(local_key.public());
        println!("Local peer id: {id:?}");

        let transport = tokio_development_transport(local_key)?;
        let mut swarm = Swarm::with_tokio_executor(transport, Behaviour::default(), id);
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Node { swarm })
    }

    pub async fn handle_event(&mut self) -> Result<(), Box<dyn Error>> {
        match self.swarm.select_next_some().await {
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
    
            other => {
                println!("Unhandled event: [{other:?}]");
                Ok(())
            },
        }
    }

    pub fn handle_command(&mut self, command: Command) -> Result<(), Box<dyn Error>> {
        match command {
            Command::Ping { remote } => {
                self.swarm.dial(remote)?;
            }
            Command::Send { remote, message } => {
                todo!()
            }
            Command::Info => {
                let peer_id = self.swarm.local_peer_id().to_string();
                let addrs = self.swarm.listeners().collect::<Vec<_>>();
                println!("Local peer id: {peer_id:?} at {addrs:?}");
            }
            Command::Accept => {
                todo!()
            }
        }
        Ok(())
    }
}

#[derive(NetworkBehaviour, Default)]
struct Behaviour {
    // TODO: substitute keeping alive with something more sensible
    keep_alive: keep_alive::Behaviour,
    ping: ping::Behaviour,
}
