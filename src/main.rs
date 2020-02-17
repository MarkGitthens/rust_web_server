use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::error::Error;

#[derive(Debug)]
enum RequestMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
    UNKNOWN,
}

#[derive(Debug)]
struct RequestLine {
    method: RequestMethod,
    target: String,
    version: String,
}

#[derive(Debug)]
struct ResponseLine {
    http_version: String,
    status_code: u8,
    reason_phrase: String,
}

#[derive(Debug)]
struct HttpHeaderInformation {
    request_line: Option<RequestLine>,
    response_line: Option<ResponseLine>,
    header_fields: HashMap<String, String>,
}

#[derive(Debug)]
struct HttpMessage {
    header_info: HttpHeaderInformation,
    payload: Option<String>,
}

/*TODO: 1) Add a thread pool for each incoming request.
        2) Add support for static HTML responses.
        3) Add support for function callback registrations for request types
        4) Seperate core http parsing logic into it's own library
        5) Add further support for http parsing/handling based off of the latest HTTP RFC standards
*/

fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:7999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut tcp_stream: TcpStream = stream;

                //TODO: Need to determine a good buffer size for general http messages
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
        let length: usize = result.1.header_fields.get("Content-Length").unwrap().parse().unwrap();
        let mut data: String = String::from(split_message[1]);
        data.truncate(length);
        
        HttpMessage {
            header_info: result.1,
            payload: Some(data),
        }
    } else {
        HttpMessage {
            header_info: result.1,
            payload: None,
        }
    };

    match message_object.payload {
        Some(payload) => {println!("{:?}\r\n\r\n{:?}", message_object.header_info, payload);},
        None => {},
    }
}

fn parse_header_information(headers: &str) -> (bool, HttpHeaderInformation) {
    //Reverse vector so that we can use it as a stack to simplify processing
    let mut header_lines: Vec<&str> = headers.split("\r\n").collect();    
    header_lines.reverse();

    let start: Vec<&str> = header_lines.pop().unwrap().split(" ").collect();

    //TODO: Probably don't need to initialize both of these
    let mut _request_line: RequestLine = RequestLine {
        method: RequestMethod::UNKNOWN,
        target: String::from(""),
        version: String::from(""),
    };

    let mut _response_line: ResponseLine = ResponseLine {
        http_version: String::from(""),
        status_code: 0,
        reason_phrase: String::from(""),
    };
 
    //TODO: Maybe setup a basic callback system that gets triggered here.
    match start[0] {
        "GET" => {_request_line.method = RequestMethod::GET;}
        "HEAD" => { _request_line.method = RequestMethod::HEAD;}
        "POST" => { _request_line.method = RequestMethod::POST;}
        "PUT" => { _request_line.method = RequestMethod::PUT;}
        "DELETE" => { _request_line.method = RequestMethod::DELETE;}
        "CONNECT" => { _request_line.method = RequestMethod::CONNECT;}
        "OPTIONS" => { _request_line.method = RequestMethod::OPTIONS;}
        "TRACE" => { _request_line.method = RequestMethod::TRACE;}
        "PATCH" => { _request_line.method = RequestMethod::PATCH;}
        _ => { /*Stub case*/ }
    };

    //TODO: Add sanity checking and return proper error code if invalid data
    //TODO: Add logic to determine if this is a request or a response message
    let mut result: HttpHeaderInformation = HttpHeaderInformation {
        request_line: None,
        response_line: None,
        header_fields: HashMap::new(),
    };

    match _request_line.method {
        RequestMethod::UNKNOWN => {/*TODO: We are a response, or invalid*/}
        _ => { 
            _request_line.target = String::from(start[1]);
            _request_line.version = String::from(start[2]); 
        
            result.request_line = Some(_request_line);
        }
    }

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

    match result.header_fields.get("Content-Length") {
        Some(_x) => return (true, result),
        None => return (false, result),
    }
}

//Temporary helper function for testing
fn parse_request(request: &mut [u8]) -> String {
    let get = b"GET / HTTP/1.1\r\n";
    let mut response: String = String::from("");
    
    if request.starts_with(get) {
        response = format!("{}{}", build_headers(), build_response());
    } 

    return response;
}

fn file_does_not_exist(response: &mut String){
    *response = String::from("<!DOCTYPE html><html><head><title>GET response</title></head><body>Couldn't find file</body></html>");
}

fn set_file(f1: File, file: &mut Option<File>) {
    *file = Some(f1);
}

//Temporary helper function for testing
//TODO: This seems really REALLY gross probably misusing match super hard here
fn build_response() -> String {
    //TODO: have this be configurable through a config file
    let path  = Path::new("/home/mark/html/static/example.html");

    let mut response: String = String::new();
    let file = File::open(&path);
    let mut f: Option<File> = None;
    
    match file {
        Err(_) => file_does_not_exist(&mut response),
        Ok(resp) => set_file(resp, &mut f),
    };

    if response.is_empty() {
        match f {
            Some(mut x) => match x.read_to_string(&mut response) {
                _ => ()
            },
            None => ()
        };
    }

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