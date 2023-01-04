use libp2p::futures::StreamExt;
use std::error::Error;

use libp2p::swarm::{keep_alive, NetworkBehaviour, SwarmEvent};
use libp2p::{floodsub, identity, ping, tokio_development_transport, PeerId, Swarm};

use crate::Command;

pub struct Node {
    swarm: Swarm<Behaviour>,
}

impl Node {
    pub fn new() -> Result<Node, Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519();
        let id = PeerId::from(local_key.public());

        let transport = tokio_development_transport(local_key)?;
        let behaviour = Behaviour {
            keep_alive: keep_alive::Behaviour::default(),
            ping: ping::Behaviour::default(),
            floodsub: floodsub::Floodsub::new(id.clone()),
        };
        let mut swarm = Swarm::with_tokio_executor(transport, behaviour, id);
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Node { swarm })
    }

    pub async fn handle_event(&mut self) -> Result<String, Box<dyn Error>> {
        match self.swarm.select_next_some().await {
            SwarmEvent::Behaviour(event) => Ok(format!("Behaviour event:\n{event:?}")),

            SwarmEvent::NewListenAddr { address, .. } => Ok(format!("Listening on: {address}")),

            other => Ok(format!("Unhandled event:\n{other:?}")),
        }
    }

    pub fn handle_command(&mut self, command: Command) -> Result<Option<String>, Box<dyn Error>> {
        let result = match command {
            Command::Ping { remote } => {
                self.swarm.dial(remote)?;
                None
            }
            Command::Send { remote, message } => {
                todo!()
            }
            Command::Info => {
                let peer_id = self.swarm.local_peer_id().to_string();
                let address = self.swarm.listeners().last().unwrap().to_string();
                let address = format!("{address}/p2p/{peer_id}");

                let mut info = String::new();
                info.push_str(&format!("Address: {address}\n"));
                info.push_str("You can connect to other peers with:\n");
                info.push_str(&format!("  ping {address}\n"));
                info.push_str(&format!("  send {address} hello there!\n"));
                Some(info)
            }
            Command::Accept => {
                todo!()
            }
        };

        Ok(result)
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    // TODO: substitute keeping alive with something more sensible
    keep_alive: keep_alive::Behaviour,
    ping: ping::Behaviour,
    floodsub: floodsub::Floodsub,
}
