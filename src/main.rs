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

use std::path::{PathBuf, Path};
use std::io::prelude::*;
use std::net::TcpListener;
use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use percent_encoding::{percent_decode_str};

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

enum RequestMethod {
    GET,
    HEAD,
    POST,
    CONNECT,
    OPTIONS,
    TRACE,
    UNKNOWN,
}

struct RequestLine {
    method: RequestMethod,
    target: String,
    version: String,
}

struct ResponseLine {
    http_version: String,
    status_code: u16,
    reason_phrase: String,
}

struct RequestHeader {
    request_line: RequestLine,
    header_fields: HashMap<String, String>,
}

struct ResponseHeader {
    response_line: ResponseLine,
    header_fields: HashMap<String, String>,
}

struct HttpRequest {
    header_info: RequestHeader,
    payload: Option<Vec<u8>>,
}

struct HttpResponse {
    header_info: ResponseHeader,
    payload: Option<Vec<u8>>,
}

impl HttpResponse {
    fn serialize(&mut self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let response_line: String = format!("{} {} {}\r\n",
            self.header_info.response_line.http_version,
            self.header_info.response_line.status_code,
            self.header_info.response_line.reason_phrase);
        result.extend_from_slice(response_line.as_bytes());

        for i in self.header_info.header_fields.iter() {
            result.extend_from_slice(format!("{}: {}\r\n", i.0, i.1).as_bytes());
        }

        result.extend_from_slice(String::from("\r\n").as_bytes());
        match &self.payload {
            Some(x) => result.extend_from_slice(&x[..]),
            None => ()
        }

        result
    }
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

    let split_lines: Vec<&str> = file_contents.split('\n').collect::<Vec<&str>>();
    
    for iter in split_lines.iter() {
        let field: &str = iter;

        let split_field: Vec<&str> = field.split('=').collect();
        let field_value: String = split_field[1..].join("=");

        map.insert(String::from(split_field[0].trim()), field_value.trim().to_string());
    }

    //For now we only require one config entry. 
    //If we need more required config options we should pull this into it's own function
    if !map.contains_key("server_directory") {
        panic!("server_directory not defined in config file!");
    }

    map
}

fn main() {
    let config_data: HashMap<String, String> = read_config();

    let listener: TcpListener = TcpListener::bind("0.0.0.0:8000").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut read_buffer: [u8; 512] = [0; 512];
                stream.read(&mut read_buffer).unwrap();

                match parse_http_message(&mut read_buffer) {
                    Some(msg) => {
                        let mut response: HttpResponse = build_get_response(&config_data, msg);
                        stream.write_all(&response.serialize()[..]).unwrap();  
                    },
                    None => {
                        let mut result = HttpResponse {
                            header_info: ResponseHeader {
                                response_line:  ResponseLine {
                                    http_version: String::from("HTTP/1.1"),
                                    status_code: 400,
                                    reason_phrase: String::from("Invalid Request")
                                },
                                header_fields: HashMap::new(),
                            },
                            payload: None
                        };

                        stream.write_all(&result.serialize()[..]).unwrap();
                    }
                };                                            
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn parse_http_message(request: &mut [u8]) -> Option<HttpRequest> {
    let http_message: String = format!("{}", String::from_utf8_lossy(&request[..]));
    let split_message: Vec<&str> = http_message.split("\r\n\r\n").collect::<Vec<&str>>();
    
    match parse_header_information(split_message[0]) {
        Some(headers) => {
            let mut result = HttpRequest {
                header_info: headers,
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

            Some(result)
        },
        None => {
            None
        }
    }
}

//Decode percent encoded values and then return None if we find any ..
//Don't need to check for ~ due to how we construct our path for resources later
fn valid_uri(uri: &str) -> Option<String> {
    match percent_decode_str(uri).decode_utf8() {
        Ok(decoded_uri) => {
            match decoded_uri.find("..") {
                Some(_x) => None,
                None => Some((&decoded_uri).to_string())
            }
        },
        Err(_x) => None
    }
}

//Validates and parses the request line
fn parse_request_line(header_info: &str) -> Option<RequestLine> {
    let tokens: Vec<&str> = header_info.split(' ').collect();
    let mut result: RequestLine  = RequestLine {
        method: RequestMethod::UNKNOWN,
        target: "".to_string(),
        version: "HTTP/1.1".to_string()
    };

    if tokens.len() == 3 {
        match tokens[0] {
            "GET" => {result.method = RequestMethod::GET;}
            "HEAD" => { result.method = RequestMethod::HEAD;}
            "POST" => { result.method = RequestMethod::POST;}
            "CONNECT" => { result.method = RequestMethod::CONNECT;}
            "OPTIONS" => { result.method = RequestMethod::OPTIONS;}
            "TRACE" => { result.method = RequestMethod::TRACE;}
            _ => { return None; }
        };

        match valid_uri(tokens[1]) {
            Some(decoded_uri) => result.target = decoded_uri,
            None => return None
        };
        
        if tokens[2] != "HTTP/1.1" && tokens[2] != "HTTP/1.0" {
            println!("Wrong version type {}", tokens[2]);
            return None;
        }
        result.version = String::from(tokens[2]);
        Some(result)
    } else {
        None
    }
}

fn parse_header_information(headers: &str) -> Option<RequestHeader> {
    let header_lines: Vec<&str> = headers.split("\r\n").collect();

    match parse_request_line(header_lines[0]) {
        Some(req_line) =>  {
            let mut result: RequestHeader = RequestHeader {
                request_line: req_line,
                header_fields: HashMap::new(),
            };

            for iter in header_lines.iter().skip(1) {
                let field: &str = iter;

                let split_field: Vec<&str> = field.split(':').collect();
                let field_value: String = split_field[1..].join(":");

                result.header_fields.insert(String::from(split_field[0]), field_value.trim().to_string());
            }

            Some(result)
        },
        None => None
    }
}

fn file_does_not_exist() -> Vec<u8>{
    String::from("<!DOCTYPE html><html><head><title>GET response</title></head><body>Couldn't find file</body></html>").as_bytes().to_vec()
}

fn build_get_response(config_map: &HashMap<String,String>, request: HttpRequest)  -> HttpResponse {
    let mut path: PathBuf = PathBuf::from((*config_map).get("server_directory").unwrap());
    let mut response: HttpResponse = HttpResponse {
        header_info: build_test_headers(),
        payload: None,
    };
    
    if request.header_info.request_line.target == "/" {
        path.push("index.html");
    } else {
        //skip the first forward slash
        path.push(&request.header_info.request_line.target[1..]);
    }

    let file_type: FileType = match path.extension() {
        Some(ext) => {
            match ext.to_str().unwrap() {
                "html" => FileType::HTML,
                "htm" => FileType::HTM,
                "css" => FileType::CSS,
                "jpeg" => FileType::JPEG,
                "jpg" => FileType::JPG,
                "png" => FileType::PNG,
                "js" => FileType::JS,
                "ico" => FileType::ICO,
                _ => FileType::UNKNOWN
            }
        },
        None => FileType::UNKNOWN
    };
    
    response.header_info.header_fields.insert(String::from("Content-Type"),
        String::from(match file_type {
            FileType::HTM => "text/html",
            FileType::HTML => "text/html",
            FileType::JPG => "image/jpeg",
            FileType::JPEG => "image/jpeg",
            FileType::PNG => "image/png",
            FileType::JS  => "text/javascript",
            FileType::CSS => "text/css",
            FileType::ICO => "image/x-icon",
            FileType::UNKNOWN => "text/html"
        })
    );

    match File::open(&path) {
        Err(_) => {
            return generate_404();
        },
        Ok(mut file) => {
            if file.metadata().unwrap().is_dir() {
                return generate_404();
            }
        
            let mut buf: Vec<u8> = Vec::new();
        
            match file.read_to_end(&mut buf) {
                Ok(num_read) => {
                    response.header_info.header_fields.insert(String::from("Content-Length"), num_read.to_string());
                    response.payload = Some(buf);
                },
                Err(e) => {
                    println!("Read failed on {}", e.description());
                    return generate_404();
                }
            };
        }
    };
    response
}

fn generate_404() -> HttpResponse {
    let resp = file_does_not_exist();
    let mut headers: HashMap<String, String> = HashMap::new();

    headers.insert(String::from("Content-Length"), resp.len().to_string());

    HttpResponse {
        header_info: ResponseHeader {
            response_line: ResponseLine {
                http_version: String::from("HTTP/1.1"),
                status_code: 404,
                reason_phrase: String::from("Not Found"),
            },
            header_fields: headers
        },
        payload: Some(resp)
    }
}
//Temporary helper function for testing
fn build_test_headers() -> ResponseHeader {
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert(String::from("Server"), String::from("rust_test"));

    ResponseHeader {
        response_line: ResponseLine {
            http_version: String::from("HTTP/1.1"),
            status_code: 200,
            reason_phrase: String::from("OK"),
        },
        header_fields: headers,
    }
}