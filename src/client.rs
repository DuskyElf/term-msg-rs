use std::net;
use std::io::{self, BufReader};
use std::io::prelude::*;

pub fn start_client() {
    let mut stream = net::TcpStream::connect("127.0.0.1:6969")
        .expect("Couldn't connect to the server");

    loop {
        user_interface(&mut stream)
    }

    /* let mut buff = String::new();
    let mut stream = BufReader::new(&stream);
    let length = stream.read_to_string(&mut buff).expect("Read fail");
    println!("{}", length);
    println!("{}", buff); */
}

fn user_interface(tcp_stream: &mut net::TcpStream) {
    let mut user_input = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut user_input).unwrap();

    tcp_stream.write(user_input.as_bytes()).expect("Tcp Write Failed");
    tcp_stream.flush().unwrap();

    let mut buff = [0; 1024];
    let mut stream = BufReader::new(tcp_stream);
    stream.read(&mut buff).expect("Tcp Read Failed");
    println!("{}", String::from_utf8(buff.to_vec()).unwrap());
}
