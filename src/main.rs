use std::env;
use std::process::exit;

mod server;
mod client;

const SERVER_ARG: char = 's';
const CLIENT_ARG: char = 'c';

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: Expected argument for server / client");
        exit(1);
    }

    match args[1].chars().next().unwrap() {
        SERVER_ARG => server::start_server(),
        CLIENT_ARG => client::start_client(),
        _ => exit(1)
    }
}
