mod server;
mod thread_pool;

use server::HttpServer;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Missing server address!");
    } else if args.len() > 2 {
        panic!(format!("Unknown args: {}...", args[2]));
    }

    let server = HttpServer::new(args[1].as_str());
    server.serve();
}
