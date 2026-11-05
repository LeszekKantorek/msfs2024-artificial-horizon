use std::{env, io::{BufRead, Read, Write}, net::TcpListener, thread, time::Duration};
fn main() {
    let args: Vec<_> = env::args().collect();
    let port = args.iter().position(|arg| arg == "--port").unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    thread::spawn(move || { for line in std::io::stdin().lock().lines() { if line.unwrap() == "stop" { let _ = sender.send(()); break; } } });
    let listener = TcpListener::bind(format!("127.0.0.1:{}", args[port + 1])).unwrap();
    listener.set_nonblocking(true).unwrap();
    while receiver.try_recv().is_err() {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut bytes = [0; 8192];
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let count = stream.read(&mut bytes).unwrap();
                if count == 0 { break; }
                request.extend_from_slice(&bytes[..count]);
            }
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
        }
        thread::sleep(Duration::from_millis(5));
    }
    std::io::stdout().write_all(&vec![b'x'; 8 * 1024 * 1024]).unwrap();
    std::io::stderr().write_all(&vec![b'y'; 8 * 1024 * 1024]).unwrap();
}
