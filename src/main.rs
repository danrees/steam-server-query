use std::net::UdpSocket;

use steam_server_query::query_info_cursor;

fn main() {
    println!("Hello, world!");
    let socket = UdpSocket::bind("0.0.0.0:8000").expect("could not bind socket");
    //query_info(&socket);
    let packet = query_info_cursor(&socket).expect("could not query info");
    println!("Info: {:?}", packet);
}
