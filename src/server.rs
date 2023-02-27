use std::net;
use std::thread;
use std::sync::{Arc, Mutex};
use std::io::prelude::*;
use std::io::BufReader;

type Clients = Arc<Mutex<Vec<net::TcpStream>>>;

pub fn start_server() {
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    let listner = net::TcpListener::bind("127.0.0.1:6969")
        .expect("Wan't able to bind the TCP listner");

    println!("Listning for the client... ");

    for stream in listner.incoming() {
        if let Err(err) = stream {
            eprintln!("{err}");
            std::process::exit(1);
        }
        let stream = stream.unwrap();
        let clients = Arc::clone(&clients);
        handle_client(stream, clients);
    }
}

fn handle_client(stream: net::TcpStream, clients: Clients) {
    thread::spawn(move || {
        {
            let mut clients = clients.lock().unwrap();
            clients.push(stream.try_clone().unwrap());
        }

        let addr = stream.peer_addr().unwrap();
        println!("Connected to incoming client from: {addr}");

        loop {
            let mut buff = Vec::new();
            let mut stream = BufReader::new(&stream);
            let length = stream.read_until(b'\n', &mut buff)
                .expect("Tcp Read fail");
            if length == 0 {
                break;
            }
            println!("{}", String::from_utf8(buff.clone()).unwrap());

            {
                let clients = clients.lock().unwrap();
                for mut client in clients.iter() {
                    client.write(&buff).expect("Tcp Write Fail");
                    client.flush().unwrap();
                }
            }
        }
/*
        let length = stream.write("Hello world".as_bytes())
            .expect("Couldn't able to write on the network");
        println!("Successfully wrote {} chars", length);
*/
    });
}

