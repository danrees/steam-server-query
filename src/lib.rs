use std::error::Error;
use std::fmt::Display;
use std::net::UdpSocket;
use std::str::{self, from_utf8};

const HEADER_SINGLE: i32 = -1;
const HEADER_MULTIPLEL: i32 = -2;

#[derive(Debug)]
struct QueryError(String);

impl Error for QueryError {}
impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn query_info() {
    let buffer = bufferize();
    let socket = UdpSocket::bind("0.0.0.0:8000").expect("could not bind to address");

    socket
        .send_to(&buffer, "192.168.1.116:27015")
        .expect("could not send to address");
    let mut buff: [u8; 1400] = [0; 1400];
    socket
        .recv_from(&mut buff)
        .expect("could not recieve reply");

    println!("{:x?}", buff);
    let response_header = i32::from_le_bytes(
        buff[0..4]
            .try_into()
            .expect("couldn't get first long from response"),
    );
    println!("Found response header: {}", response_header);
    let protocol: char = buff[4].try_into().expect("couldn't get protocol");
    println!("found {} as the protocol", protocol);
    let mut start_from = 5;

    let (name, next) = extract_string(&buff[start_from..], 0x00);
    start_from = start_from + next + 1;

    println!(
        "Server Name: {}",
        name.expect("could not get name of server")
    );

    let (server, next) = extract_string(&buff[start_from..], 0x00);
    start_from = start_from + next + 1;

    println!(
        "Map name is: {}",
        server.expect("unable to get server name")
    );

    let (folder, next) = extract_string(&buff[start_from..], 0x00);
    start_from = start_from + next + 1;

    println!(
        "Folder name is: {}",
        folder.expect("could not get server folder")
    );

    let (game, next) = extract_string(&buff[start_from..], 0x00);
    start_from = start_from + next + 1;
    println!("Game name is: {}", game.expect("could not get game name"));
    //println!("Debug: {:x?}", &buff[start_from..]);
    let id = i16::from_le_bytes(
        buff[start_from..(start_from + 2)]
            .try_into()
            .expect("could not get game id"),
    );
    start_from += 2;
    println!("Game id is: {}", id);
    let num_players = buff[start_from];
    start_from += 1;
    let max_players = buff[start_from];
    println!("Players: {}/{}", num_players, max_players);
    start_from += 1;
    let bots = buff[start_from];
    start_from += 1;
    let server_type: char = buff[start_from]
        .try_into()
        .expect("could not get server type");
    start_from += 1;
    println!("server type: {}", server_type);
    let env: char = buff[start_from]
        .try_into()
        .expect("could not get environment");
    start_from += 1;
    println!("env: {}", env);
    let visibility = buff[start_from];
    start_from += 1;
    println!("vis: {}", visibility);
    let vac = buff[start_from];
    start_from += 1;
    println!("vac: {}", vac);
    let (version, next) = extract_string(&buff[start_from..], 0x00);
    start_from = start_from + next + 1;
    println!("version: {}", version.expect("could not get version"));
    let edf = buff[start_from];
    start_from += 1;
    println!("EDF: {:x?}", edf);
    println!("remaining");
    println!("{:x?}", &buff[start_from..]);
    //let is_port = edf & 0x80;
    let flags: Vec<u8> = vec![0x80, 0x10, 0x40, 0x20, 0x01];
    println!("Debug 0xb1 & 0x80 = {:x?}", 0xb1 & 0x80);
    for flag in flags.iter() {
        let result = edf & flag;
        println!("{:x?} {:x?} {:x?}", edf, flag, result);
        println!("flag ({:x?}): {:x?}", flag, result);
    }
    if edf > 0 {
        if (edf & 0x80) > 0 {
            let port = u16::from_le_bytes(
                buff[start_from..start_from + 2]
                    .try_into()
                    .expect("could not get game port"),
            );
            println!("game port is: {}", port);
            start_from += 2;
        }
        if (edf & 0x10) > 0 {
            println!("{:x?}", &buff[start_from..(start_from + 8)]);
            let id_bytes: Option<[u8; 8]> = buff[start_from..(start_from + 8)].try_into().ok();
            start_from += 8;
            if let Some(id) = id_bytes {
                let steam_id = u64::from_le_bytes(id);
                println!("steam id is {}", steam_id);
            } else {
                println!("couldn't extract steam id even though edf said it should be there")
            }
        }

        if (edf & 0x40) > 0 {
            println!("should be spectator stuff");
            // short
            start_from += 2;
            // string
            let (_, next) = extract_string(&buff, 0x00);
            start_from = start_from + next + 1;
        }
        if (edf & 0x20) > 0 {
            //println!("{:x?}", &buff[start_from..]);
            let (keywords, next) = extract_string(&buff, 0x00);
            start_from = start_from + next + 1;
            println!("Keywords: {}", keywords.expect("could not get keywords"));
        }
    }

    ()
}

fn bufferize() -> Vec<u8> {
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

fn extract_string(bytes: &[u8], term: u8) -> (Result<String, QueryError>, usize) {
    let mut val_bytes = Vec::new();
    let mut index = 0;
    for (i, val) in bytes.iter().enumerate() {
        index = i;
        if *val == term {
            break;
        }
        val_bytes.push(*val);
    }
    match str::from_utf8(&val_bytes) {
        Ok(val) => (Ok(String::from(val)), index),
        Err(e) => (Err(QueryError(format!("{}", e))), index),
    }
}
