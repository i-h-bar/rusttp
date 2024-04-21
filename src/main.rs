use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind listener");
    let pool = ThreadPool::new(16);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| handle_stream(stream));
    }
}

fn handle_stream(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, response_file) =
        if request_line == "GET / HTTP/1.1" {
            ("HTTP/1.1 200 OK", "hello.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        };

    reply(stream, status_line, response_file);
}

fn reply(mut stream: TcpStream, status: &str, response_file: &str) {
    let body = fs::read_to_string(response_file).unwrap();
    let content_len = body.len();
    let headers = format!("Content-Length: {content_len}");

    let response = format!("{status}\r\n{headers}\r\n\r\n{body}");
    stream.write_all(response.as_bytes()).unwrap()
}