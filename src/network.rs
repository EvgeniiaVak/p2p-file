use libp2p::futures::StreamExt;
use libp2p::{request_response, Transport};

use futures::{io, AsyncRead, AsyncWrite, AsyncWriteExt};
use std::error::Error;
use std::iter;

use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::request_response::{ProtocolName, ProtocolSupport};
use libp2p::swarm::{keep_alive, NetworkBehaviour};
use libp2p::{identity, PeerId, Swarm};
use libp2p::{mplex, noise};

use crate::Command;

use async_trait::async_trait;

pub struct Node {
    swarm: Swarm<Behaviour>,
    connected_peer: Option<PeerId>,
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

        let behaviour = Behaviour {
            keep_alive: keep_alive::Behaviour::default(),
            request_response: request_response::RequestResponse::new(
                FileExchangeCodec(),
                iter::once((FileExchangeProtocol(), ProtocolSupport::Full)),
                request_response::RequestResponseConfig::default(),
            ),
        };
        let mut swarm = Swarm::with_tokio_executor(transport, behaviour, id);
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Node {
            swarm,
            connected_peer: None,
        })
    }

    pub async fn handle_event(&mut self) -> Result<String, Box<dyn Error>> {
        use libp2p::request_response::RequestResponseEvent::*;
        use libp2p::request_response::RequestResponseMessage::*;
        use libp2p::swarm::SwarmEvent::*;

        match self.swarm.select_next_some().await {
            Behaviour(BehaviourEvent::RequestResponse(Message {
                peer,
                message:
                    Request {
                        request_id,
                        request,
                        channel,
                    },
            })) => {
                // TODO: wrap in an async?
                // TODO: provide an actual file
                let response: Vec<u8> = vec![1, 2, 3, 4, 5];
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, FileResponse(response))
                    .expect("Failed to send response");
                Ok(format!(
                    "Sent file {request:?} \n\tto {peer}\n\trequest_id: {request_id}"
                ))
            }

            Behaviour(BehaviourEvent::RequestResponse(Message {
                peer,
                message: Response { response, .. },
            })) => {
                // TODO: wrap in an async?
                // TODO: save the file
                let file = response.0;

                Ok(format!("Received file {file:?} \n\tfrom {peer}"))
            }

            NewListenAddr { address, .. } => Ok(format!("Listening on: {address}")),

            IncomingConnection {
                local_addr,
                send_back_addr,
            } => Ok(format!(
                "Incoming connection to: {local_addr}\n\tfrom: {send_back_addr}"
            )),

            ConnectionEstablished { peer_id, .. } => {
                self.connected_peer = Some(peer_id);
                Ok(format!("Connected: {peer_id}"))
            }
            other => Ok(format!("Unhandled event:\n{other:?}")),
        }
    }

    pub fn handle_command(&mut self, command: Command) -> Result<Option<String>, Box<dyn Error>> {
        match command {
            Command::Connect { remote } => {
                self.swarm.dial(remote)?;
                Ok(None)
            }

            Command::Info => {
                let peer_id = self.swarm.local_peer_id().to_string();
                let address = self.swarm.listeners().last().unwrap().to_string();
                let address = format!("{address}/p2p/{peer_id}");

                let mut info = String::new();
                info.push_str("Other peers can connect and request a file:\n");
                info.push_str(&format!("connect {address}\n"));
                info.push_str("request /path/to/file");
                Ok(Some(info))
            }

            Command::Request { file_path } => match self.connected_peer {
                Some(peer) => {
                    let request_id = self
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer, FileRequest(file_path));
                    Ok(Some(format!("Sent request {request_id:?} to {peer}")))
                }
                None => Err("No peer connected".into()),
            },
        }
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    keep_alive: keep_alive::Behaviour,
    request_response: request_response::RequestResponse<FileExchangeCodec>,
}

// TODO: remove empty tuples
#[derive(Debug, Clone)]
struct FileExchangeProtocol();
#[derive(Clone)]
struct FileExchangeCodec();
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResponse(Vec<u8>);

impl ProtocolName for FileExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/file-exchange/1".as_bytes()
    }
}

#[async_trait]
impl request_response::RequestResponseCodec for FileExchangeCodec {
    type Protocol = FileExchangeProtocol;
    type Request = FileRequest;
    type Response = FileResponse;

    async fn read_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileRequest(String::from_utf8(vec).unwrap()))
    }

    async fn read_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 500_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileResponse(vec))
    }

    async fn write_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileRequest(data): FileRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileResponse(data): FileResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
