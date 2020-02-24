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
        4) path sanitation for resource files to mitigate risk of leaking non-server data
*/

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::str;
use std::io::BufWriter;

#[derive(Debug)]
enum FileType {
    HTML,
    HTM,
    CSS,
    JS,
    PNG,
    JPG,
    JPEG,
    ICO,
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

fn read_config() -> HashMap<String,String>{
    let mut map: HashMap<String, String> = HashMap::new();
    let config_path = Path::new("config.txt");
    let mut config_file = match File::open(config_path) {
        Ok(contents) => contents,
        Err(e) => panic!("Couldn't open config file! {}", e.description())
    };

    let mut file_contents: String = String::new();
    match config_file.read_to_string(&mut file_contents) {
        Ok(_bytes_read) => (),
        Err(e) => panic!("Couldn't read file contents! {}", e.description())
    };

    let split_lines: Vec<&str> = file_contents.split("\n").collect::<Vec<&str>>();
    
    for x in 0..split_lines.len() {
        let field: &str = split_lines[x];

        let split_field: Vec<&str> = field.split("=").collect();
        let field_value: String = split_field[1..].join("=");

        map.insert(String::from(split_field[0].trim()), field_value.trim().to_string());
    }

    //For now we only require one config entry. 
    //If we need more required config options we should pull this into it's own function
    match map.contains_key("server_directory") {
        false => panic!("server_directory not defined in config file!"),
        _ => ()
    };

    return map;
}

fn main() {
    let config_data: HashMap<String, String> = read_config();

    let listener: TcpListener = TcpListener::bind("localhost:8080").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                //TODO: Need to determine a good buffer size for general http messages
                let mut read_buffer: [u8; 512] = [0; 512];
                stream.read(&mut read_buffer).unwrap();

                match parse_http_message(&mut read_buffer) {
                    Some(msg) => {
                        let mut buffer = BufWriter::new(stream);
                        build_get_response(&config_data, &mut buffer, msg);
                        buffer.flush().unwrap();  
                    },
                    //TODO: Send error message
                    None => println!("Couldn't parse http message"),
                };                                            
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

//TODO: Do we have to worry about chunked data?
fn parse_http_message(request: &mut [u8]) -> Option<HttpMessage> {
    let header_info: HttpHeaderInformation;

    let http_message: String = format!("{}", String::from_utf8_lossy(&request[..]));
    let split_message: Vec<&str> = http_message.split("\r\n\r\n").collect::<Vec<&str>>();
    
    match parse_header_information(split_message[0]) {
        Some(x) => header_info = x,
        None => {
            println!("Couldn't parse header information");
            return None;
        }
    };

    let mut result = HttpMessage {
        header_info: header_info,
        payload: None
    };

    match result.header_info.header_fields.get("Content-Length") {
        Some(x) => {
            let mut data: String = String::from(split_message[1]);
            data.truncate(x.parse().unwrap());
            result.payload = Some(data.as_bytes().to_vec());
        },
        None => result.payload = None
    };

    return Some(result);
}

//TODO: Verify request line
fn parse_header_information(headers: &str) -> Option<HttpHeaderInformation> {
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

    return Some(result);
}

//TODO: We should have this grab info from a user defined error file
fn file_does_not_exist() -> Vec<u8>{
    return String::from("<!DOCTYPE html><html><head><title>GET response</title></head><body>Couldn't find file</body></html>").as_bytes().to_vec();
}

//TODO: A bit cleaner still needs more work
//TODO: Probably vulnerable to Path Traversal exploits
fn build_get_response(config_map: &HashMap<String,String>, buffer: &mut BufWriter<TcpStream>, request: HttpMessage) {
    let mut path: PathBuf = PathBuf::from((*config_map).get("server_directory").unwrap());
    
    //TODO: Need to sanitize paths to remove . and ..
    let uri: String;
    match request.header_info.request_line {
        Some(x) => uri = x.target,
        None => panic!("Couldn't get request line!") 
    };
    
    //skip the first forward slash
    path.push(&uri[1..]);

    let file_type: FileType;

    match path.extension() {
       Some(ext) => {
            match ext.to_str().unwrap() {
                "html" => file_type = FileType::HTML,
                "htm" => file_type = FileType::HTM,
                "css" => file_type = FileType::CSS,
                "jpeg" => file_type = FileType::JPEG,
                "jpg" => file_type = FileType::JPG,
                "png" => file_type = FileType::PNG,
                "js" => file_type = FileType::JS,
                "ico" => file_type = FileType::ICO,
                _ => file_type = FileType::UNKNOWN
            };
        },
        None => file_type = FileType::UNKNOWN
    };

    (*buffer).write(&build_headers()[..]).unwrap();

    let mut content_type: String = String::from("Content-Type: ");

    match file_type {
        FileType::HTM => {content_type = format!("{}{}\r\n", content_type, "text/html");},
        FileType::HTML => {content_type = format!("{}{}\r\n", content_type, "text/html");},
        FileType::JPG => {content_type = format!("{}{}\r\n", content_type, "image/jpeg");},
        FileType::JPEG => {content_type = format!("{}{}\r\n", content_type, "image/jpeg");},
        FileType::PNG => {content_type = format!("{}{}\r\n", content_type, "image/png");},
        FileType::JS  => {content_type = format!("{}{}\r\n", content_type, "text/javascript");},
        FileType::CSS => {content_type = format!("{}{}\r\n", content_type, "text/css");},
        FileType::ICO => {content_type = format!("{}{}\r\n", content_type, "image/x-icon");},
        FileType::UNKNOWN => {content_type = format!("{}{}\r\n", content_type, "text/html");}
    };

    (*buffer).write(content_type.as_bytes()).unwrap();

    let mut file = match File::open(&path) {
        Err(_) => {(*buffer).write(&file_does_not_exist()[..]).unwrap(); return;},
        Ok(resp) => resp,
    };

    let mut buf: Vec<u8> = Vec::new();
    let con_len: usize;

    match file.read_to_end(&mut buf) {
        Ok(num_read) => con_len = num_read,
        Err(e) => {con_len = 0; println!("Read failed {}", e.description())}
    };

    let content_length: String = format!("Content-Length: {}\r\n\r\n", con_len);
    (*buffer).write(content_length.as_bytes()).unwrap();
    (*buffer).write(&buf[..]).unwrap();
}

//Temporary helper function for testing
fn build_headers() -> Vec<u8> {
    let status_line: String = String::from("HTTP/1.1 200 OK\r\n");
    let header_line: String = String::from("Server: rust_test\r\n");

    return format!("{}{}", status_line, header_line).as_bytes().to_vec();
}