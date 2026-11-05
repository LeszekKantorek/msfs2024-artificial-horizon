use std::{env, io::{Read, Write}, net::TcpListener};
fn main() {
    let args: Vec<_> = env::args().collect();
    let port = args.iter().position(|arg| arg == "--port").unwrap();
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args[port + 1])).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut bytes = [0; 1024];
        let _ = stream.read(&mut bytes);
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
    }
}
