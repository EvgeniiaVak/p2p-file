use libp2p::futures::StreamExt;
use libp2p::Transport;
use std::error::Error;

use libp2p::floodsub::{self, FloodsubEvent};
use libp2p::swarm::{keep_alive, NetworkBehaviour, SwarmEvent};
use libp2p::{identity, PeerId, Swarm};
use libp2p::{mplex, noise};

use crate::Command;

pub struct Node {
    swarm: Swarm<Behaviour>,
    floodsub_topic: floodsub::Topic,
}

impl Node {
    pub fn new() -> Result<Node, Box<dyn Error>> {
        let key_pair = identity::Keypair::generate_ed25519();
        let id = PeerId::from(key_pair.public());

        let transport = {
            use libp2p::core::upgrade;
            use libp2p::tcp::{tokio::Transport, Config};

            Transport::new(Config::default().nodelay(true))
                .upgrade(upgrade::Version::V1)
                .authenticate(
                    noise::NoiseAuthenticated::xx(&key_pair)
                        .expect("Signing libp2p-noise static DH keypair failed."),
                )
                .multiplex(mplex::MplexConfig::new())
                .boxed()
        };

        let floodsub_topic = floodsub::Topic::new("chat");
        let mut behaviour = Behaviour {
            keep_alive: keep_alive::Behaviour::default(),
            floodsub: floodsub::Floodsub::new(id.clone()),
        };
        behaviour.floodsub.subscribe(floodsub_topic.clone());
        let mut swarm = Swarm::with_tokio_executor(transport, behaviour, id);
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Node {
            swarm,
            floodsub_topic,
        })
    }

    pub async fn handle_event(&mut self) -> Result<String, Box<dyn Error>> {
        match self.swarm.select_next_some().await {
            SwarmEvent::Behaviour(BehaviourEvent::Floodsub(FloodsubEvent::Message(message))) => {
                let mut short_addr = message.source.to_string();
                short_addr.truncate(5);
                Ok(format!(
                    "{:?}...: {}",
                    short_addr,
                    String::from_utf8_lossy(&message.data)
                ))
            }
            SwarmEvent::Behaviour(BehaviourEvent::Floodsub(FloodsubEvent::Subscribed {
                peer_id,
                topic,
            })) => Ok(format!(
                "Peer {:?} subscribed to topic {:?}",
                peer_id, topic
            )),

            SwarmEvent::NewListenAddr { address, .. } => Ok(format!("Listening on: {address}")),

            SwarmEvent::IncomingConnection {
                local_addr,
                send_back_addr,
            } => Ok(format!(
                "Incoming connection to: {local_addr}\n\tfrom: {send_back_addr}"
            )),

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                self.swarm
                    .behaviour_mut()
                    .floodsub
                    .add_node_to_partial_view(peer_id);

                Ok(format!("Connected: {peer_id}"))
            }
            other => Ok(format!("Unhandled event:\n{other:?}")),
        }
    }

    pub fn handle_command(&mut self, command: Command) -> Result<Option<String>, Box<dyn Error>> {
        let result = match command {
            Command::Connect { remote } => {
                self.swarm.dial(remote)?;
                None
            }
            Command::Send { message } => {
                self.swarm
                    .behaviour_mut()
                    .floodsub
                    .publish(self.floodsub_topic.clone(), message.as_bytes());
                None
            }
            Command::Info => {
                let peer_id = self.swarm.local_peer_id().to_string();
                let address = self.swarm.listeners().last().unwrap().to_string();
                let address = format!("{address}/p2p/{peer_id}");

                let mut info = String::new();
                info.push_str("Other peers can send a message to you with:\n");
                info.push_str(&format!("  connect {address}\n"));
                info.push_str(&format!("  send MY_MESSAGE"));
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
    keep_alive: keep_alive::Behaviour,
    floodsub: floodsub::Floodsub,
}
