use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

// Response headers.
const HEADER_200: &str = "HTTP/1.1 200 OK\r\n\r\n";
const HEADER_404: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const HEADER_405: &str = "HTTP/1.1 405 METHOD NOT ALLOWED\r\nAllow: GET\r\n\r\n";
const HEADER_500: &str = "Http/1.1 500 INTERNAL ERROR\r\n\r\n";

// Main http server struct.
pub struct HttpServer {
    listener: TcpListener,
    read_timeout: Option<Duration>,
    router: HashMap<String, String>,
}

impl HttpServer {
    // Create a http server listening at addr.
    pub fn new(addr: &str) -> HttpServer {
        let mut router = HashMap::new();
        router.insert(String::from("/"), String::from("assets/index.html"));
        HttpServer {
            listener: TcpListener::bind(addr).unwrap(),
            read_timeout: Some(Duration::new(0, 500_000_000)),
            router: router,
        }
    }

    // Start listening to requests.
    pub fn serve(&self) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            println!("Accept connection from: {:?}", stream.peer_addr().unwrap());
            self.handle_request(stream);
        }
    }

    // Handle a connection.
    fn handle_request(&self, mut stream: TcpStream) {
        let request = self.parse_request(&mut stream);

        // Only GET is allowed.
        if request.method != "GET" {
            stream.write(HEADER_405.as_bytes()).unwrap();
            return;
        }

        let request_url: &str = request.url.as_str();
        let file_path: &str = match self.router.get(request_url) {
            Some(path) => path,
            None => {
                stream.write(HEADER_404.as_bytes()).unwrap();
                return;
            }
        };

        match fs::read_to_string(file_path) {
            Ok(content) => {
                stream.write(HEADER_200.as_bytes()).unwrap();
                stream.write(content.as_bytes()).unwrap();
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    stream.write(HEADER_404.as_bytes()).unwrap();
                }
                _ => {
                    stream.write(HEADER_500.as_bytes()).unwrap();
                }
            },
        }
    }

    fn parse_request(&self, stream: &mut TcpStream) -> HttpRequest {
        // Set read timeout.
        stream.set_read_timeout(self.read_timeout).unwrap();

        // Read request contents.
        let mut buffer: Vec<u8> = Vec::new();
        match stream.read_to_end(&mut buffer) {
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => {}
                std::io::ErrorKind::TimedOut => {}
                _ => {
                    panic!(e);
                }
            },
            Ok(_) => {}
        }

        HttpRequest::parse(&buffer)
    }
}

// Http request struct.
struct HttpRequest {
    method: String,
    url: String,
    #[used]
    version: String,
    #[used]
    headers: HashMap<String, String>,
}

impl HttpRequest {
    // Create a request struct from string.
    pub fn parse(buffer: &[u8]) -> HttpRequest {
        let request_str = String::from_utf8_lossy(buffer);

        let mut headers = HashMap::new();
        let request: Vec<&str> = request_str.split("\r\n").collect();
        let request_infos: Vec<&str> = request[0].split(" ").collect();
        for i in 1..request.len() - 1 {
            let header: Vec<&str> = request[i].split(": ").collect();
            if header.len() > 1 {
                // println!("{}: {}", header[0], header[1]);
                headers.insert(String::from(header[0]), String::from(header[1]));
            }
        }

        HttpRequest {
            method: String::from(request_infos[0]),
            url: String::from(request_infos[1]),
            version: String::from(request_infos[2]),
            headers: headers,
        }
    }
}
