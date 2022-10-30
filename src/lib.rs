use byteorder::{LittleEndian, ReadBytesExt};
use std::error::Error;
use std::fmt::Display;
use std::io::Cursor;
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

fn build_info_request() -> Vec<u8> {
    let neg_one: i32 = -1;
    let header = b'T';
    let payload = "Source Engine Query\0".as_bytes();
    let package: Vec<u8> = neg_one
        .to_le_bytes()
        .iter()
        .chain(vec![header].iter())
        .chain(payload.to_owned().iter())
        .map(|i| i.to_owned())
        .collect();
    package
}

#[derive(Debug)]
pub struct InfoPacket {
    protocol: char,
    name: String,
    map: String,
    folder: String,
    game: String,
    id: i16,
    players: u8,
    max_players: u8,
    bots: u8,
    server_type: char,
    environment: char,
    visibility: bool,
    vac: bool,
    version: String,
    edf: u8,
}

pub fn query_info_cursor(socket: &UdpSocket) -> Result<InfoPacket, anyhow::Error> {
    let to_send = build_info_request();
    socket.send_to(&to_send, "192.168.1.116:27015")?;
    let mut buffer: [u8; 1400] = [0x00; 1400];
    let (size, addr) = socket.recv_from(&mut buffer)?;

    let data = buffer[..size].to_vec();
    let mut cursor = Cursor::new(data);
    let header = cursor.read_i32::<LittleEndian>()?;

    match header {
        HEADER_SINGLE => println!("single header"),
        _ => println!("different header"),
    }
    println!("Header is: {}", header);
    let protocol: char = cursor.read_u8()?.try_into()?;
    println!("Protocol is: {}", protocol);
    let name = cursor.read_string()?;
    println!("Name is: {}", name);
    let map = cursor.read_string()?;
    println!("Map is: {}", map);
    let folder = cursor.read_string()?;

    Ok(InfoPacket {
        protocol: protocol,
        name: name,
        map: map,
        folder: folder,
        game: cursor.read_string()?,
        id: cursor.read_i16::<LittleEndian>()?,
        players: cursor.read_u8()?,
        max_players: cursor.read_u8()?,
        bots: cursor.read_u8()?,
        server_type: cursor.read_char()?,
        environment: cursor.read_char()?,
        visibility: cursor.read_bool()?,
        vac: cursor.read_bool()?,
        version: cursor.read_string()?,
        edf: cursor.read_u8()?,
    })
}

// fn extract_string_cursor(cursor: &mut Cursor<Vec<u8>>) -> String {
//     let mut buffer = Vec::with_capacity(256);
//     while let Ok(c) = cursor.read_u8() {
//         if c == 0x00 {
//             break;
//         }
//         buffer.push(c);
//     }

//     String::from_utf8_lossy(&buffer).into_owned()
// }
