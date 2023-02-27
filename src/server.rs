use std::collections::HashMap;
use std::net;
use std::net::SocketAddr;
use std::thread;
use std::sync::{Arc, Mutex};
use std::io::prelude::*;
use std::io::BufReader;

static mut GLOBAL_COUNTER: usize = 0;
type Clients = Arc<Mutex<HashMap<usize, net::TcpStream>>>;

pub fn start_server() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

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
        let client_id;
        {
            let mut clients = clients.lock().unwrap();
            unsafe {
                client_id = GLOBAL_COUNTER;
                GLOBAL_COUNTER += 1;
            };
            clients.insert(client_id, stream.try_clone().unwrap());
        }

        let addr = stream.peer_addr().unwrap();
        println!("Connected to incoming client from: {addr}");

        fn disconnect(clients: &Clients, client_id: &usize, addr: &SocketAddr) {
            let mut clients = clients.lock().unwrap();
            clients.remove(client_id);
            println!("disconnected from client: {}", addr);
        }

        loop {
            let mut buff = Vec::new();
            let mut stream = BufReader::new(&stream);
            let responce = stream.read_until(b'\n', &mut buff);
            if let Err(e) = responce {
                eprintln!("Error: {e}");
                disconnect(&clients, &client_id, &addr);
                break;
            } else {
                if responce.unwrap() == 0 {
                    disconnect(&clients, &client_id, &addr);
                    break;
                }
            }

            println!("{}", String::from_utf8(buff.clone()).unwrap());

            {
                let clients = clients.lock().unwrap();
                for mut client in clients.iter() {
                    if *client.0 != client_id {
                        client.1.write(&buff).expect("Tcp Write Fail");
                        client.1.flush().unwrap();
                    }
                    // println!("{:#?}", buff);
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

