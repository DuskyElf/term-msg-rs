use std::env;
use std::process::exit;

mod server;
mod client;

const SERVER_ARG: char = 's';
const CLIENT_ARG: char = 'c';

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: Expected argument '{SERVER_ARG}' or '{CLIENT_ARG}'");
        exit(1);
    }

    match args[1].chars().next().unwrap() {
        SERVER_ARG => server::start_server().await,
        CLIENT_ARG => client::start_client().await,
        _ => eprintln!("Error: Expected argument '{SERVER_ARG}' or '{CLIENT_ARG}'"),
    }
}
