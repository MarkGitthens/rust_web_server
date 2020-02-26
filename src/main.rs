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

/*TODO: 1) Add basic threading support
        2) Add support for static HTML responses.
        3) Determine how much of the HTTP std we need to implement for a bare minimum static server
        4) path sanitation for resource files to mitigate risk of leaking non-server data
*/
use std::path::{PathBuf, Path};
use std::io::prelude::*;
use std::net::TcpListener;
use std::collections::HashMap;
use std::fs::File;
use std::error::Error;

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
    //TODO: Not sure if there is a faster/better way to do this
    fn serialize(&mut self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let response_line: String = format!("{} {} {}\r\n",
            self.header_info.response_line.http_version,
            self.header_info.response_line.status_code,
            self.header_info.response_line.reason_phrase);
        result.extend_from_slice(response_line.as_bytes());

        for i in self.header_info.header_fields.iter() {
            result.extend_from_slice(&mut format!("{}: {}\r\n", i.0, i.1).as_bytes());
        }

        result.extend_from_slice(&mut String::from("\r\n").as_bytes());
        match &self.payload {
            Some(x) => result.extend_from_slice(&x[..]),
            None => ()
        }

        return result;
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
                        let mut response: HttpResponse = build_get_response(&config_data, msg);
                        stream.write(&response.serialize()[..]).unwrap();  
                    },
                    //TODO: Send 400 malformed request error message
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
fn parse_http_message(request: &mut [u8]) -> Option<HttpRequest> {
    let header_info: RequestHeader;

    let http_message: String = format!("{}", String::from_utf8_lossy(&request[..]));
    let split_message: Vec<&str> = http_message.split("\r\n\r\n").collect::<Vec<&str>>();
    
    match parse_header_information(split_message[0]) {
        Some(x) => header_info = x,
        None => {
            println!("Couldn't parse header information");
            return None;
        }
    };

    let mut result = HttpRequest {
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

//Validates and parses the request line
fn parse_request_line(header_info: &str) -> Option<RequestLine> {
    let tokens: Vec<&str> = header_info.split(" ").collect();
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

        //Need to sanitize and verify this is a correct uri 
       /* for i in tokens[1].chars() {

        }
        */
        result.target = String::from(tokens[1]);
        
        if tokens[2].to_lowercase() != "http/1.1" {
            println!("Wrong version type");
            return None;
        }
        result.version = String::from(tokens[2]);
    
    } else {
        return None;
    }
    return Some(result);
}

//TODO: Verify request line
fn parse_header_information(headers: &str) -> Option<RequestHeader> {
    let header_lines: Vec<&str> = headers.split("\r\n").collect();

    match parse_request_line(header_lines[0]) {
        Some(request_line) =>  {
            //TODO: Add sanity checking and return proper error code if invalid data
            let mut result: RequestHeader = RequestHeader {
                request_line: request_line,
                header_fields: HashMap::new(),
            };

            for x in 1..header_lines.len() {
                let field: &str = header_lines[x];

                let split_field: Vec<&str> = field.split(":").collect();
                let field_value: String = split_field[1..].join(":");

                result.header_fields.insert(String::from(split_field[0]), field_value.trim().to_string());
            }

            return Some(result);
        },
        None =>  {
            return None;
        }
    };    
}

//TODO: We should have this grab info from a user defined error file
fn file_does_not_exist() -> Vec<u8>{
    return String::from("<!DOCTYPE html><html><head><title>GET response</title></head><body>Couldn't find file</body></html>").as_bytes().to_vec();
}

//TODO: A bit cleaner still needs more work
//TODO: Probably vulnerable to Path Traversal exploits
fn build_get_response(config_map: &HashMap<String,String>, request: HttpRequest)  -> HttpResponse {
    let mut path: PathBuf = PathBuf::from((*config_map).get("server_directory").unwrap());
    let mut response: HttpResponse = HttpResponse {
        header_info: build_test_headers(),
        payload: None,
    };
    
    //skip the first forward slash
    //TODO: Default to {server_directory}/index.html if target is /
    path.push(&request.header_info.request_line.target[1..]);

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

    //TODO: Need to clean this up
    //TODO: Need to construct different response depending on requested content type
    match File::open(&path) {
        Err(_) => {
            let resp = file_does_not_exist();
            response.header_info.response_line.reason_phrase = String::from("Not Found");
            response.header_info.response_line.status_code = 404;
            response.header_info.header_fields.insert(String::from("Content-Length"), resp.len().to_string());
            response.payload = Some(resp);
        },
        Ok(mut file) => {
            if file.metadata().unwrap().is_dir() {
                let resp = file_does_not_exist();
                response.header_info.response_line.reason_phrase = String::from("Not Found");
                response.header_info.response_line.status_code = 404;
                response.header_info.header_fields.insert(String::from("Content-Length"), resp.len().to_string());
                response.payload = Some(resp);
            }
        
            let mut buf: Vec<u8> = Vec::new();
        
            match file.read_to_end(&mut buf) {
                Ok(num_read) => {
                    response.header_info.header_fields.insert(String::from("Content-Length"), num_read.to_string());
                    response.payload = Some(buf);
                },
                //TODO: Return http error code
                Err(e) => {println!("Read failed {}", e.description())}
            };
        }
    };
    return response;
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