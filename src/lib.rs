use byteorder::{LittleEndian, ReadBytesExt};
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::io::{Cursor, Read};
use std::net::UdpSocket;

const HEADER_SINGLE: i32 = -1;
//const HEADER_MULTIPLEL: i32 = -2;

trait ReadPacketString {
    fn read_string(&mut self) -> Result<String, std::io::Error>;
    fn read_char(&mut self) -> Result<char, anyhow::Error>;
    fn read_bool(&mut self) -> Result<bool, anyhow::Error>;
}

impl ReadPacketString for Cursor<Vec<u8>> {
    fn read_string(&mut self) -> Result<String, std::io::Error> {
        let mut buffer = Vec::with_capacity(256);
        while let Ok(c) = self.read_u8() {
            if c == 0x00 {
                break;
            }
            buffer.push(c);
        }

        Ok(String::from_utf8_lossy(&buffer).into_owned())
    }

    fn read_char(&mut self) -> Result<char, anyhow::Error> {
        let c: char = self.read_u8()?.try_into()?;
        Ok(c)
    }

    fn read_bool(&mut self) -> Result<bool, anyhow::Error> {
        let b = self.read_u8()?;
        if b == 0 {
            return Ok(true);
        }
        Ok(false)
    }
}

#[derive(Debug)]
struct QueryError(String);

impl Error for QueryError {}
impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn build_info_request(challenge: Option<&[u8]>) -> Vec<u8> {
    let neg_one: i32 = -1;
    let header = b'T';
    let payload = "Source Engine Query\0".as_bytes();
    let mut resp: Vec<u8> = neg_one
        .to_le_bytes()
        .iter()
        .chain(vec![header].iter())
        .chain(payload.to_owned().iter())
        .map(|i| i.to_owned())
        .collect();
    if let Some(c) = challenge {
        resp.extend_from_slice(c);
    }
    resp
}

#[derive(Debug)]
pub struct InfoPacket {
    pub protocol: char,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub id: i16,
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: char,
    pub environment: char,
    pub visibility: bool,
    pub vac: bool,
    pub version: String,
    pub edf: u8,
}

pub fn query_info_cursor(
    socket: &UdpSocket,
    challenge: Option<&[u8]>,
) -> Result<InfoPacket, anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        return Err(anyhow::anyhow!(
            "you must provide the host:port as the first argument"
        ));
    }
    let addr = &args[1];
    let to_send = build_info_request(challenge);
    dbg!(&to_send);
    socket.send_to(&to_send, addr)?;
    let mut buffer: [u8; 1400] = [0x00; 1400];
    let (size, _) = socket.recv_from(&mut buffer)?;

    let data = buffer[..size].to_vec();
    dbg!(&data);
    let mut cursor = Cursor::new(data);
    let header = cursor.read_i32::<LittleEndian>()?;

    match header {
        HEADER_SINGLE => println!("single header"),
        _ => println!("different header"),
    }

    let protocol: char = cursor.read_u8()?.try_into()?;
    dbg!(&protocol);
    if protocol == 'A' {
        // then this is a challenge request
        let mut buff = [0_u8; 4];
        cursor.read_exact(&mut buff)?;
        return query_info_cursor(socket, Some(&buff));
    }
    let name = cursor.read_string()?;
    dbg!(&name);
    let map = cursor.read_string()?;
    dbg!(&map);
    let folder = cursor.read_string()?;
    dbg!(&folder);
    let game = cursor.read_string()?;
    dbg!(&game);
    let id = cursor.read_i16::<LittleEndian>()?;
    dbg!(&id);
    let players = cursor.read_u8()?;
    let max_players = cursor.read_u8()?;
    let bots = cursor.read_u8()?;
    let server_type = cursor.read_char()?;
    let environment = cursor.read_char()?;
    let visibility = cursor.read_bool()?;
    let vac = cursor.read_bool()?;
    let version = cursor.read_string()?;
    let edf = cursor.read_u8()?;

    Ok(InfoPacket {
        protocol: protocol,
        name: name,
        map: map,
        folder: folder,
        game: game,
        id: id,
        players: players,
        max_players: max_players,
        bots: bots,
        server_type: server_type,
        environment: environment,
        visibility: visibility,
        vac: vac,
        version: version,
        edf: edf,
    })
}

fn query_players(socket: &UdpSocket) -> Result<(), anyhow::Error> {
    Ok(())
}
