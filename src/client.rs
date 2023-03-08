use ron;
use rand;
use std::time;
use tokio::net;
use std::thread;
use pancurses as pc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use term_msg_rs::*;

pub async fn start_client() {
    let window = pc::initscr();
    window.keypad(true);

    let result = ui(&window).await;

    if let Err(err) = result {
        pc::endwin();
        match err {
            Error::InvalidEvent => {
                eprintln!("Server Error: Server sent an Invalid event")
            }
            Error::IoError(err) => {
                eprintln!("{err}")
            }
        }
    } else {
        pc::endwin();
    }

}

async fn ui(window: &pc::Window) -> Result<(), Error> {
    let mut socket = connect_to_server(&window).await;
    let server_info = handshake(&window, &mut socket).await?;
    if server_info.is_protected {
        verify(&window, &mut socket).await?;
    }

    Ok(())
}

async fn connect_to_server(window: &pc::Window) -> net::TcpStream {
    loop {
        let mut server_ip = prompt(&window, "Server's IP address:");
        server_ip.push_str(&format!(":{PORT}"));
        let connection = net::TcpStream::connect(server_ip).await;

        match connection {
            Ok(tcp_stream) => break tcp_stream,
            Err(err) => tell_message(window, &format!("Couldn't connect to the server: {err}")),
        }
    }
}

async fn handshake(window: &pc::Window, socket: &mut net::TcpStream) -> Result<ServerInfo, Error> {
    let identifier = rand::random();
    send_event(socket, ClientToServerEvent::Ping { identifier }).await?;

    let ping_responce = receive_event(socket).await?
        .expected(NetworkPhase::Ping)?;
    let (res_identifier, server_info) =
        if let ServerToClientEvent::PingResponce { identifier: res_identifier, server_info }
            = ping_responce {
                (res_identifier, server_info)
    } else { unreachable!() };

    if res_identifier != identifier {
        return Err(Error::IoError("Max Password tries exceeded".to_string()));
    }

    tell_message(
        window,
        &format!("{server_name}, Welcomes you!", server_name=server_info.server_name
    ));

    Ok(server_info)
}

async fn verify(window: &pc::Window, socket: &mut net::TcpStream) -> Result<(), Error> {
    loop {
        let password = prompt(&window, "Server password: ");
        send_event(socket, ClientToServerEvent::Verify { password }).await?;
        let verify_responce = receive_event(socket).await?;

        if let ServerToClientEvent::Disconnect = verify_responce {
            return Err(Error::IoError("Couldn't authenticate, try next time!".to_string()));
        }

        match verify_responce.expected(NetworkPhase::Verify)? {
            ServerToClientEvent::VerificationSuccessful => break,
            ServerToClientEvent::VerificationFailed =>
                tell_message(window, &format!("Wrong Password, Please try again!")),
            _ => unreachable!()
        }
    }
    Ok(())
}

async fn send_event(socket: &mut net::TcpStream, event: ClientToServerEvent) -> Result<(), Error> {
    let mut line = ron::to_string(&event).unwrap();
    line.push('\n'); // '\n' is the ending indicator

    let network_result = socket.write_all(line.as_bytes()).await;
    if let Err(err) = network_result {
        return Err(Error::IoError(format!("Server Error: {err}")))
    }
    Ok(())
}

async fn receive_event(socket: &mut net::TcpStream) -> Result<ServerToClientEvent, Error> {
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

fn prompt(window: &pc::Window, question: &str) -> String {
    animated_message(window, question);
    window.addstr("\n\n> ");

    let mut responce = String::new();
    scan(window, &mut responce);
    responce
}

fn tell_message(window: &pc::Window, info: &str) {
    animated_message(window, info);
    window.addstr("\n\nPress any key to continue");
    window.refresh();

    pc::curs_set(0);
    pc::noecho();
    window.getch();
    pc::echo();
    pc::curs_set(1);
}

fn animated_message(window: &pc::Window, info: &str) {
    window.clear();
    window.mv(0, 0);

    for letter in info.chars() {
        window.addch(letter as u32);
        thread::sleep(time::Duration::from_millis(15));
        window.refresh();
    }
}

fn scan(window: &pc::Window, buffer: &mut String) {
    pc::noecho();
    loop {
        match window.getch().unwrap() {
            pc::Input::Character('\n') => {
                pc::echo();
                break;
            },

            pc::Input::KeyBackspace => {
                if buffer.len() != 0 {
                    buffer.pop();
                    window.mv(window.get_cur_y(), window.get_cur_x() - 1);
                    window.delch();
                }
                continue;
            },

            pc::Input::Character(read) => {
                window.addch(read);
                buffer.push(read);
            },

            _ => (),
        }
    }
}
