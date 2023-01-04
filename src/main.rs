use std::{net::UdpSocket, time::Duration};

use steam_server_query::query_info_cursor;

fn main() {
    println!("Hello, world!");
    let socket = UdpSocket::bind("0.0.0.0:8000").expect("could not bind socket");
    //query_info(&socket);
    socket
        .set_read_timeout(Some(Duration::new(5, 0)))
        .expect("unable to set read timeout");
    let packet = query_info_cursor(&socket, None).expect("could not query info");
    println!("Info: {:?}", packet);
}
