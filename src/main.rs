use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;

//TODO: Determine what fields should be nullable and update to be of type Option<T>
#[derive(Debug)]
struct RequestLine {
    method: String,
    target: String,
    version: String,
}

#[derive(Debug)]
struct HttpHeaderInformation {
    status_line: RequestLine,
    header_fields: HashMap<String, String>,
}

#[derive(Debug)]
struct HttpMessage {
    header_info: HttpHeaderInformation,
    payload: Option<String>,
}

//TODO: Need to benchmark practically everything here and will most likely need to make some optimizations
fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:7999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut tcp_stream: TcpStream = stream;

                //TODO: Need to determine a good buffer size for general http messages
                //TODO: Need to plan for potentially needing multiple buffers worth of data for a single http messages
                let mut buffer: [u8; 512] = [0; 512];
                tcp_stream.read(&mut buffer).unwrap();

                parse_http_message(&mut buffer);
                let headers = parse_request(&mut buffer);               

                tcp_stream.write(headers.as_bytes()).unwrap();
                tcp_stream.flush().unwrap();                
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

//TODO: This will probably have to change for handling chunked data.
fn parse_http_message(request: &mut [u8]) {
    let http_message: String = format!("{}", String::from_utf8_lossy(&request[..]));
    let split_message: Vec<&str> = http_message.split("\r\n\r\n").collect::<Vec<&str>>();

    let result: (bool, HttpHeaderInformation) = parse_header_information(split_message[0]);

    let message_object: HttpMessage = if result.0 {
        HttpMessage {
            header_info: result.1,
            payload: Some(String::from(split_message[1])),
        }
    } else {
        HttpMessage {
            header_info: result.1,
            payload: None,
        }
    };

    match message_object.payload {
        Some(payload) => {println!("{:?}\r\n\r\n{:?}", message_object.header_info, payload);},
        None => {println!("{:?}", message_object.header_info);},
    }
}

fn parse_header_information(headers: &str) -> (bool, HttpHeaderInformation) {

    //Reverse vector so that we can use it as a stack to simplify processing
    let mut header_lines: Vec<&str> = headers.split("\r\n").collect();    
    header_lines.reverse();

    let start: Vec<&str> = header_lines.pop().unwrap().split(" ").collect();

    //TODO: Add sanity checking and return proper error code if invalid data
    //TODO: Add logic to determine if this is a request or a response message
    let mut result: HttpHeaderInformation = HttpHeaderInformation {
        status_line: RequestLine { 
            method: String::from(start[0]),
            target: String::from(start[1]),
            version: String::from(start[2]), },
        header_fields: HashMap::new(),
    };

    while !header_lines.is_empty(){
        let field: Option<&str> = header_lines.pop();
        match field {
            Some(res) => {
                //TODO: Should I provide better parsing for field values or leave that up to the user?
                let split_field: Vec<&str> = res.split(":").collect();
                let field_value: String = split_field[1..].join(":");

                result.header_fields.insert(String::from(split_field[0]), field_value.trim().to_string());
            },
            None => { continue; },
        }
    }

    //Default to return false since we aren't yet checking if we need to process a payload or not
    return (false, result);
}

//Temporary helper function for testing
fn parse_request(request: &mut [u8]) -> String {
    let get = b"GET / HTTP/1.1\r\n";

    let mut response: String = String::from("");
    if request.starts_with(get) {
        response = format!("{}{}", build_headers(), build_response());
    } else {
        println!("{}", String::from_utf8_lossy(&request[..]));
    }      

    return response;
}

//Temporary helper function for testing
fn build_response() -> String {
    let response: String = String::from("<!DOCTYPE html><html><head><title>GET response</title></head><body>Yo this a body</body></html>");
    let content_type: String = String::from("Content-Type: text/html\r\n");
    let content_length: String = format!("Content-Length: {}\r\n\r\n", response.len());

    return format!("{}{}{}", content_type, content_length, response);
}

//Temporary helper function for testing
fn build_headers() -> String {
    let status_line: String = String::from("HTTP/1.1 200 OK\r\n");
    let header_line: String = String::from("Server: rust_test\r\n");

    return format!("{}{}", status_line, header_line);
}