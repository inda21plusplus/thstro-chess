use std::error::Error as StdError;
use std::io::{ErrorKind, Read};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::{FromStr, Utf8Error};
use std::{fmt, str};

use bevy::prelude::*;
use chess_engine::{Board, Game, Move, PieceType, SquareSpec};

#[derive(Debug)]
pub struct MoveReceivedEvent(Move);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<MoveReceivedEvent>()
            .init_resource::<Listener>()
            .add_system(accept_connections.system())
            .add_system(read_packets.system());
    }
}

struct Listener(TcpListener);
struct ConnectedClient {
    stream: TcpStream,
    addr: SocketAddr,
    buffer: Vec<u8>,
    kind: ClientKind,
}
enum ClientKind {
    Playing,
    Spectating,
}

/// Indicates that something went wrong so that the connection should be dropped
#[derive(Debug)]
struct NetworkError;
impl StdError for NetworkError {}
impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Network error")
    }
}
impl From<Utf8Error> for NetworkError {
    fn from(_: Utf8Error) -> Self {
        Self
    }
}

impl FromWorld for Listener {
    fn from_world(_: &mut World) -> Self {
        let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
        listener.set_nonblocking(true).unwrap();
        Self(listener)
    }
}

enum NetworkEvent {
    MoveReceivedEvent(MoveReceivedEvent),
}

fn accept_connections(
    mut commands: Commands,
    listener: Res<Listener>,
    client_query: Query<(), With<ConnectedClient>>,
) {
    match listener.0.accept() {
        Ok((stream, addr)) => {
            stream.set_nodelay(true).unwrap(); // auto flush
            stream.set_nonblocking(true).unwrap();
            let kind = if client_query.single().ok().is_none() {
                ClientKind::Playing
            } else {
                ClientKind::Spectating
            };
            commands.spawn().insert(ConnectedClient {
                stream,
                addr,
                buffer: vec![],
                kind,
            });
        }
        Err(e) => eprintln!("{:?}", e),
    }
}

fn read_packets(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut ConnectedClient)>,
    game: Res<Game>,
) {
    let mut buffer = [0_u8; 1024];
    for (entity, mut client) in clients.iter_mut() {
        let n_bytes = match client.stream.read(&mut buffer) {
            Ok(n_bytes) => n_bytes,
            Err(err) if err.kind() == ErrorKind::WouldBlock => continue,
            Err(err) => {
                println!(
                    "Network error from {}: {}\nDropping connection",
                    client.addr, err
                );
                commands.entity(entity).despawn();
                continue;
            }
        };
        let buffer = &buffer[..n_bytes];
        for packet in buffer.split_inclusive(|&b| b == b';') {
            if !packet.ends_with(&[b';']) {
                client.buffer.extend_from_slice(packet);
                break;
            }
            match client.handle_packet(packet, game.current_board()) {
                Ok(_) => {}
                Err(_) => {
                    commands.entity(entity).despawn();
                    continue;
                }
            }
        }
    }
}

impl ConnectedClient {
    fn handle_packet(&mut self, packet: &[u8], board: &Board) -> Result<(), NetworkError> {
        let (key, value) = Self::split_packet(packet)?;
        match key {
            b"move" => self.handle_move_packet(value, board),
            _ => Err(NetworkError),
        }
    }
    fn split_packet(packet: &[u8]) -> Result<(&[u8], &[u8]), NetworkError> {
        let colon_index = match packet
            .iter()
            .enumerate()
            .find(|(_, &b)| b == b':')
            .map(|(i, _)| i)
        {
            Some(i) => i,
            None => return Err(NetworkError),
        };

        Ok(packet.split_at(colon_index))
    }
    fn handle_move_packet(
        &self,
        value: &[u8],
        board: &Board,
    ) -> Result<Option<NetworkEvent>, NetworkError> {
        let m: Move = parse_move(str::from_utf8(value)?, board).ok_or(NetworkError)?;

        Ok(Some(NetworkEvent::MoveReceivedEvent(MoveReceivedEvent(m))))
    }
}

fn parse_move(s: &str, board: &Board) -> Option<Move> {
    if s.len() != 5 {
        return None;
    }

    let from = SquareSpec::from_str(&s[0..2]).ok()?;
    let to = SquareSpec::from_str(&s[2..4]).ok()?;
    if let Ok(target) = PieceType::from_str(s[4..5].to_ascii_lowercase().as_str()) {
        Some(Move::Promotion { from, to, target })
    } else if let Some(piece) = board[from] {
        Move::new(piece, from, to)
    } else {
        None
    }
}
