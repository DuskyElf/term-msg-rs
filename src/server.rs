use ron;
use std::io;
use tokio::{net, task};
use std::io::prelude::*;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use term_msg_rs::*;

const MAX_PASSWORD_TRIES: u8 = 5;

pub async fn start_server() {
    let server_name = prompt("Server Name: ");
    let password = prompt("Password (leave empty for no password): ");
    let is_protected = password.len() != 0;

    let server_info = ServerInfo {
        server_name, is_protected
    };

    let listner = net::TcpListener::bind("0.0.0.0:6969").await
        .expect("Wasn't able to bind the TCP listner");

    println!("Listning for clients ...\n");

    loop {
        let (socket, _) = listner.accept().await
            .expect("Wasn't able to listen for clients");
        new_client(socket, server_info.clone(), password.clone()).await;
    }
}

fn prompt(prompt: &str) -> String {
    let mut user_input = String::new();
    print!("{prompt}");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut user_input).unwrap();
    user_input.trim_end().to_string()
}

async fn new_client(mut socket: net::TcpStream, server_info: ServerInfo, password: String) {
    task::spawn(async {
        let addr = socket.peer_addr().unwrap();
        println!("connected to incoming client from: {addr}");

        let result = handle_client(&mut socket, server_info, password).await;
        if let Err(Error::InvalidEvent) = result {
            eprintln!("Client Error: Client sent an Invalid event");
            disconnect_client(socket, addr).await;
            return;
        } else if let Err(Error::IoError(err)) = result {
            eprintln!("{err}");
            disconnect_client(socket, addr).await;
            return;
        }
    }); 
}

async fn handle_client(socket: &mut net::TcpStream, server_info: ServerInfo, password: String) -> Result<(), Error> {
    handshake(socket, server_info.clone()).await?;
    if server_info.is_protected {
        verify(socket, password).await?;
    }
    Ok(())
}

async fn verify(socket: &mut net::TcpStream, password: String) -> Result<(), Error> {
    let mut wrong_pass_count = 1;
    loop {
        let verification = receive_event(socket).await?
            .expected(NetworkPhase::Verify)?;
        let res_password =
            if let ClientToServerEvent::Verify { password } = verification {
                password
        } else { unreachable!() };

        if res_password == password {
            break;
        } else {
            if wrong_pass_count < MAX_PASSWORD_TRIES {
                send_event(socket, ServerToClientEvent::VerificationFailed).await?;
                wrong_pass_count += 1;
            } else {
                return Err(Error::IoError("Client Error: Max password tries exceeded".to_string()))
            }
        }
    }
    send_event(socket, ServerToClientEvent::VerificationSuccessful).await?;
    Ok(())
}

async fn handshake(socket: &mut net::TcpStream, server_info: ServerInfo) -> Result<(), Error> {
    let ping = receive_event(socket).await?
        .expected(NetworkPhase::Ping)?;

    let identifier = if let ClientToServerEvent::Ping { identifier } = ping {
        identifier
    } else { unreachable!() };

    send_event(socket, ServerToClientEvent::PingResponce {
        identifier, server_info
    }).await?;
    Ok(())
}

async fn disconnect_client(mut socket: net::TcpStream, addr: SocketAddr) {
    send_event(&mut socket, ServerToClientEvent::Disconnect).await
        // We don't care if this function call fails
        .unwrap_or(()); // Just to avoid annoying warnings
    println!("disconnected from client: {}", addr);
}

async fn send_event(socket: &mut net::TcpStream, event: ServerToClientEvent) -> Result<(), Error> {
    let mut line = ron::to_string(&event).unwrap();
    line.push('\n'); // '\n' is the ending indicator

    let network_result = socket.write_all(line.as_bytes()).await;
    if let Err(err) = network_result {
        return Err(Error::IoError(format!("Client Error: {err}")))
    }
    Ok(())
}

async fn receive_event(socket: &mut net::TcpStream) -> Result<ClientToServerEvent, Error> {
    let (reader, _writer) = socket.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();

    let network_result = reader.read_line(&mut line).await;
    if let Err(err) = network_result {
        return Err(Error::IoError(format!("Client Error: {err}")))
    }

    let result = ron::from_str(&line);
    match result {
        Err(_) => Err(Error::InvalidEvent),
        Ok(value) => Ok(value)
    }
}

