use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

struct RequestLine {
    method: String,
    target: String,
    version: String,
}

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
                
                parse_http_message(&mut buffer);

                tcp_stream.write(headers.as_bytes()).unwrap();
                tcp_stream.flush().unwrap();                
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn parse_http_message(request: &mut [u8]) {
    let request_body: String = format!("{}", String::from_utf8_lossy(&request[..]));
    let message_data: Vec<&str> = request_body.split("\r\n\r\n").collect::<Vec<&str>>();

    //TODO: This will probably have to change for handling chunked data.
    if parse_header_information(message_data[0]) {
        let _data: &str = message_data[1];
    }
}

//TODO: This should probably instead return whether the headers were valid or not.
//Parses the header information and returns true if we should parse the data payload.
fn parse_header_information(headers: &str) -> bool {    
    let header_lines: Vec<&str> = headers.split("\n").collect();
    let start_line: Vec<&str> = header_lines[0].split(" ").collect();

    //TODO: Add sanity checking and return proper error code if invalid data
    let req_line: RequestLine = RequestLine { 
                                    method: String::from(start_line[0]),
                                    target: String::from(start_line[1]),
                                    version: String::from(start_line[2]), };
    return false;
}

//Method used for testing receiving and responding to a request
fn parse_request(request: &mut [u8]) -> String {
    //TODO: check for other http methods (post, put, head, delete, patch, options)
    let get = b"GET / HTTP/1.1\r\n";

    let mut response: String = String::from("");
    if request.starts_with(get) {
        response = format!("{}{}", build_headers(), build_response());
    } else {
        println!("{}", String::from_utf8_lossy(&request[..]));
    }      

    return response;
}

fn build_response() -> String {
    let response: String = String::from("<!DOCTYPE html><html><head><title>GET response</title></head><body>Yo this a body</body></html>");
    let content_type: String = String::from("Content-Type: text/html\r\n");
    let content_length: String = format!("Content-Length: {}\r\n\r\n", response.len());

    return format!("{}{}{}", content_type, content_length, response);
}

fn build_headers() -> String {
    let status_line: String = String::from("HTTP/1.1 200 OK\r\n");
    let header_line: String = String::from("Server: rust_test\r\n");

    return format!("{}{}", status_line, header_line);
}