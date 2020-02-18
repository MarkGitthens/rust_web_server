// Limiting the scope of this project. Planned features
/*  1) Bare minimum static web server.(With support for JS, png, jpeg, html, maybe more later)
        a) Support for locally hosted HTML 
        b) return image resources of png and jpeg types
        c) return locally hosted JS files
    2) HTTP error code responses for when something erroneous occurs (Accessing missing resource)
    3) Ideally have some basic threading support to allow for many simultaneous requests from users
*/

/* Things this project will NOT do
    1) Provide dynamic data (AKA don't expect to handle RESTful api calls)
    2) Provide any sort of user authentication
    3) Store any user data outside of whats required (ip, http requests,etc)
    4) Provide routing services to other applications/services
*/

/*TODO: 1) Add a thread pool for each incoming request.
        2) Add support for static HTML responses.
        3) Determine how much of the HTTP std we need to implement for a bare minimum static server
        4) Returning new Vec's all the time seems stupid inefficient. Particularly when responding with non text data
        5) path sanitation for resource files to mitigate risk of leaking non-server data
        6) Remove dirs dependency
*/

extern crate dirs;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::File;
use std::error::Error;
use std::str;

#[derive(Debug)]
enum FileType {
    HTML,
    HTM,
    CSS,
    JS,
    PNG,
    JPG,
    JPEG,
    UNKNOWN,
}

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
    payload: Option<Vec<u8>>,
}

fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:7999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut tcp_stream: TcpStream = stream;

                //TODO: Need to determine a good buffer size for general http messages
                let mut buffer: [u8; 512] = [0; 512];
                tcp_stream.read(&mut buffer).unwrap();

                let message: HttpMessage = parse_http_message(&mut buffer);
                let headers = parse_request(message);               

                tcp_stream.write(&headers[..]).unwrap();
                tcp_stream.flush().unwrap();                
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

//TODO: Do we have to worry about chunked data?
fn parse_http_message(request: &mut [u8]) -> HttpMessage {
    let http_message: String = format!("{}", String::from_utf8_lossy(&request[..]));
    let split_message: Vec<&str> = http_message.split("\r\n\r\n").collect::<Vec<&str>>();

    let result: (bool, HttpHeaderInformation) = parse_header_information(split_message[0]);

    let message_object: HttpMessage = if result.0 {
        let length: usize = result.1.header_fields.get("Content-Length").unwrap().parse().unwrap();
        let mut data: String = String::from(split_message[1]);
        data.truncate(length);
        
        HttpMessage {
            header_info: result.1,
            payload: Some(data.as_bytes().to_vec()),
        }
    } else {
        HttpMessage {
            header_info: result.1,
            payload: None,
        }
    };

    return message_object;
}

fn parse_header_information(headers: &str) -> (bool, HttpHeaderInformation) {
    let header_lines: Vec<&str> = headers.split("\r\n").collect();
    let start: Vec<&str> = header_lines[0].split(" ").collect();

    let mut _request_line: RequestLine = RequestLine {
        method: RequestMethod::UNKNOWN,
        target: String::from(""),
        version: String::from(""),
    };

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

    for x in 1..header_lines.len() {
        let field: &str = header_lines[x];

        let split_field: Vec<&str> = field.split(":").collect();
        let field_value: String = split_field[1..].join(":");

        result.header_fields.insert(String::from(split_field[0]), field_value.trim().to_string());
    }

    match result.header_fields.get("Content-Length") {
        Some(_x) => return (true, result),
        None => return (false, result),
    }
}

//Temporary helper function for testing
fn parse_request(request: HttpMessage) -> Vec<u8> {    
    let req_line = request.header_info.request_line.unwrap();

    let mut response: Vec<u8> = build_headers().as_bytes().to_vec();
    response.append(&mut build_get_response(req_line.target));
    return response;
}

//TODO: We should have this grab info from a user defined error file
fn file_does_not_exist(response: &mut String){
    *response = String::from("<!DOCTYPE html><html><head><title>GET response</title></head><body>Couldn't find file</body></html>");
}

fn set_file(f1: File, file: &mut Option<File>) {
    *file = Some(f1);
}

//TODO: A bit cleaner still needs more work
//TODO: Probably vulnerable to Path Traversal exploits
fn build_get_response(uri: String) -> Vec<u8> {
    //TODO: have this be configurable through a config file
    let mut path: PathBuf = match dirs::home_dir() {
        Some(x) => x,
        None => PathBuf::new()
    };

    //TODO: Need verification of URI and will need to probably normalize it
    let split_message: Vec<&str> = uri.split(".").collect::<Vec<&str>>();

    let mut file_type: FileType = FileType::UNKNOWN;
    //For now assume that the uri is sane
    if split_message.len() == 2 {
        match split_message[1] {
            "html" => file_type = FileType::HTML,
            "htm" => file_type = FileType::HTM,
            "css" => file_type = FileType::CSS,
            "jpeg" => file_type = FileType::JPEG,
            "jpg" => file_type = FileType::JPG,
            "png" => file_type = FileType::PNG,
            "js" => file_type = FileType::JS,
            _ => file_type = FileType::UNKNOWN
        };
    }

    path.push("html/static");

    //skip the first forward slash
    path.push(&uri[1..]);

    let mut response: Vec<u8> = Vec::new();

    let file = File::open(&path);
    let mut f: Option<File> = None;
    
    let mut text: String = String::new();
    match file {
        Err(_) => file_does_not_exist(&mut text),
        Ok(resp) => set_file(resp, &mut f),
    };

    let mut content_type: String = String::from("Content-Type: ");
    let mut con_len: usize = 0;

    match file_type {
        FileType::HTM => {content_type = format!("{}{}\r\n", content_type, "text/html");},
        FileType::HTML => {content_type = format!("{}{}\r\n", content_type, "text/html");},
        FileType::JPG => {content_type = format!("{}{}\r\n", content_type, "image/jpeg");},
        FileType::JPEG => {content_type = format!("{}{}\r\n", content_type, "image/jpeg");},
        FileType::PNG => {content_type = format!("{}{}\r\n", content_type, "image/png");},
        FileType::JS  => {content_type = format!("{}{}\r\n", content_type, "text/javascript");},
        FileType::CSS => {content_type = format!("{}{}\r\n", content_type, "text/css");},
        FileType::UNKNOWN => {content_type = format!("{}{}\r\n", content_type, "text/plain");}
    };

    //If file exists
    match f {
        Some(mut x) => {
            let mut buf: Vec<u8> = Vec::new();

            match x.read_to_end(&mut buf) {
                Ok(num_read) => con_len = num_read,
                Err(e) => {con_len = 0; println!("Read failed {}", e.description())}
            };
            response = buf;
        },
        None => println!("Couldn't open the requested file! {:?}", path)
    };

    let content_length: String = format!("Content-Length: {}\r\n\r\n", con_len);

    let mut result: Vec<u8> = content_type.as_bytes().to_vec();
    result.append(&mut content_length.as_bytes().to_vec());
    result.append(&mut response);
    return result;
}

//Temporary helper function for testing
fn build_headers() -> String {
    let status_line: String = String::from("HTTP/1.1 200 OK\r\n");
    let header_line: String = String::from("Server: rust_test\r\n");

    return format!("{}{}", status_line, header_line);
}