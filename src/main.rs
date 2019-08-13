use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;

#[derive(Debug)]
struct RequestLine {
    method: String,
    target: String,
    version: String,
}

#[derive(Debug)]
struct HttpMessage {
    status_line: RequestLine,
    header_fields: HashMap<String, String>,
}

fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:7999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut tcp_stream: TcpStream = stream;

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

fn parse_http_message(request: &mut [u8]) {
    let request_body: String = format!("{}", String::from_utf8_lossy(&request[..]));
    let message_data: Vec<&str> = request_body.split("\r\n\r\n").collect::<Vec<&str>>();

    let result: (bool, HttpMessage) = parse_header_information(message_data[0]);

    if !result.0 {
        let message_object: HttpMessage = result.1;

        println!("{:?}\n\n{:?}", message_object.status_line, message_object.header_fields);
            //TODO: This will probably have to change for handling chunked data.
        //let _data: &str = message_data[1];
    }
}

//TODO: This should probably instead return whether the headers were valid or not.
//Parses the header information and returns true if we should parse the data payload.
fn parse_header_information(headers: &str) -> (bool, HttpMessage) {
    //TODO: Research to see if we need to split on a single CR or single LF
    //The http 1.1 std specifies that headers should be split using CRLF but I remember reading somewhere
    //that historically people have used just CR or just LF. Need to verify this

    //Reverse vector so that we can use it as a stack for simpler logic
    let mut header_lines: Vec<&str> = headers.split("\r\n").collect();    
    header_lines.reverse();

    let start: Vec<&str> = header_lines.pop().unwrap().split(" ").collect();

    let mut result: HttpMessage = HttpMessage {
        status_line: RequestLine { 
                        method: String::from(start[0]),
                        target: String::from(start[1]),
                        version: String::from(start[2]), },
        header_fields: HashMap::new(),
    };
    //TODO: Add sanity checking and return proper error code if invalid data
    //TODO: Add logic to determine if this is a request or a response message

    //println!("{:?}", req_line);

    while !header_lines.is_empty(){
        //TODO: do further parsing on headers here.
        let field: Option<&str> = header_lines.pop();
        match field {
            Some(res) => {
                //TODO: Can't split on : because some fields use this character in the field value
                //Fix probably involves recombining the result of split from [1..end] for a generic field
                //Need to determine if all fields can use : as a valid character or if only a select few can
                
                let split_field: Vec<&str> = res.split(":").collect();
                result.header_fields.insert(String::from(split_field[0]), String::from(split_field[1]));
            },
            None => { continue; },
        }
    }

    return (false, result);
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