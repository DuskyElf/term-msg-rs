use std::net;
use pancurses as pc;
use std::io::prelude::*;
use std::io::BufReader;

pub fn start_client() {
    let window = pc::initscr();
    window.keypad(true);

    let stream = net::TcpStream::connect("127.0.0.1:6969")
        .expect("Couldn't connect to the server");

    loop {
        user_interface(stream.try_clone().unwrap(), &window)
    }

    /* let mut buff = String::new();
    let mut stream = BufReader::new(&stream);
    let length = stream.read_to_string(&mut buff).expect("Read fail");
    println!("{}", length);
    println!("{}", buff); */
}

fn user_interface(mut stream: net::TcpStream, window: &pc::Window) {
    let mut user_input = String::new();
    scan(window, &mut user_input);
    window.addch('\n');

    stream.write(user_input.as_bytes()).expect("Tcp Write Failed");
    stream.flush().unwrap();
    pc::endwin();
    // println!("client sent {} number of bytes", length);

    let mut buff = Vec::new();
    let mut stream = BufReader::new(&stream);
    stream.read_until(b'\n', &mut buff).unwrap();

    // println!("working!!");
    // println!("{}", String::from_utf8(buff).unwrap());
    window.addstr(String::from_utf8(buff).unwrap());
}

fn scan(window: &pc::Window, buffer: &mut String) {
    pc::noecho();
    loop {
        match window.getch().unwrap() {
            // Enter / Return
            pc::Input::Character('\n') => {
                buffer.push('\n');
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

