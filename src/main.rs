use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:7999").unwrap();

    //Iterate trough TcpStreams from listener
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut tcp_stream: TcpStream = stream;

                let mut buffer: [u8; 256] = [0; 256];
                tcp_stream.read(&mut buffer).unwrap();

                let headers = parse_request(&mut buffer);

                tcp_stream.write(headers.as_bytes()).unwrap();
                tcp_stream.flush().unwrap();                
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
    println!("Hello, world!");
}

fn parse_request(request: &mut [u8]) -> String {
    //TODO: check for other http methods (post, put, head, delete, patch, options)
    let get = b"GET / HTTP/1.1\r\n";

    let mut response: String = String::from("");
    if request.starts_with(get) {
        println!("We are working with a get request.");
        response = format!("{}{}", build_headers(), build_response());
    } else {
        println!("{}", String::from_utf8_lossy(&request[..]));
    }      

    return response;
}

fn build_response() -> String {
    let response: String = String::from("Yo this a body");
    let content_type: String = String::from("Content-Type: text/html\r\n");
    let content_length: String = format!("Content-Length: {}\r\n\r\n", response.len());


    return format!("{}{}{}", content_type, content_length, response);
}

fn build_headers() -> String {
    let status_line: String = String::from("HTTP/1.1 200\r\n");
    let header_line: String = String::from("Server: rust_test\r\n");

    return format!("{}{}", status_line, header_line);
}