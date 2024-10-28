use std::{fmt::Display, str::FromStr};


use crate::error::AspirinEatsError;

/// Simple wrapper for an HTTP Request
#[derive(Debug)]
pub struct HttpRequest {
    /// The HTTP method used in the request (GET, POST, etc)
    pub method: Option<String>,

    /// The path requested by the client
    pub path: Option<String>,

    /// The body of the request
    pub body: Option<String>,
}

impl FromStr for HttpRequest {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut output = HttpRequest{
            method:Some(String::new()),
            path:Some(String::new()),
            body:Some(String::new()),
        };
    
        //isolate the request line by finding the first newline, and 
        match s.split_once("\n"){
            Some((header,rest))=>{
                //Strip unecessary protocol 
                println!("{}",header);
                let mut path = header.strip_suffix("HTTP/1.1\r").unwrap();
                output.method = if header.contains("GET"){
                    path = path.strip_prefix("GET").unwrap();
                    Some(String::from("GET"))
                }
                else if header.contains("POST"){
                    path = path.strip_prefix("POST").unwrap();
                    Some(String::from("POST"))
                }
                else if header.contains("DELETE"){
                    path = path.strip_prefix("DELETE").unwrap();
                    Some(String::from("DELETE"))
                }
                else{
                    //not sure how to add error handling
                    //Err(AspirinEatsError::InvalidRequest)
                    todo!()
                };
                //What remains is our path
                output.path = Some(String::from(path.trim()));
                //split the rest of the message to find the body
                match rest.split_once("\r\n\r\n"){
                    Some((_,body)) => output.body = Some(String::from(body)),
                    None => todo!(),
                }
            },
            None => todo!(),
        }
        Ok(output)
    }
}

pub struct HttpResponse {
    status_code: u16,
    status_text: String,
    body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16, status_text: &str, body: &str) -> Self {
        HttpResponse {
            status_code,
            status_text: status_text.to_string(),
            body: body.to_string(),
        }
    }
}

impl Display for HttpResponse {
    /// Convert an HttpResponse struct to a valid HTTP Response
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"HTTP/1.1 {} {}\r\n\r\n{}",self.status_code.to_string(),self.status_text,self.body)
    }
}

impl From<AspirinEatsError> for HttpResponse {
    /// Given an error type, convert it to an appropriate HTTP Response
    fn from(value: AspirinEatsError) -> Self {
        let mut output = HttpResponse{
            status_code:0,
            status_text:String::new(),
            body:String::new(),
        };
        match value{
            AspirinEatsError::InvalidRequest =>{
                output.status_code = 400;
                output.status_text = String::from("Bad Request");
                output.body = String::from("Invalid Request");
            }
            AspirinEatsError::NotFound =>{
                output.status_code = 404;
                output.status_text = String::from("Not Found");
                output.body = String::from("Resource not found");            }
            AspirinEatsError::MethodNotAllowed =>{
                output.status_code = 405;
                output.status_text = String::from("Method Not Allowed");
                output.body = String::from("Method not allowed");            }
            AspirinEatsError::Io(_) =>{
                output.status_code = 500;
                output.status_text = String::from("Internal Server Error");
                output.body = String::from("Internal Server Error");            }
            _ =>{
                output.status_code = 505;
                output.status_text = String::from("Unknown Error");
                output.body = String::from("Unexpected Error");
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_from_str() {
        let request = "GET /orders HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(request).unwrap();
        assert_eq!(http_request.method, Some("GET".to_string()));
        assert_eq!(http_request.path, Some("/orders".to_string()));
        assert_eq!(http_request.body, Some("this is the body.".to_string()));
    }

    #[test]
    fn test_http_response_to_string() {
        let response = HttpResponse::new(200, "OK", "Welcome to Aspirin Eats!");
        assert_eq!(
            response.to_string(),
            "HTTP/1.1 200 OK\r\n\r\nWelcome to Aspirin Eats!"
        );
    }

    #[test]
    fn test_http_response_from_aspirin_eats_error() {
        let error = AspirinEatsError::InvalidRequest;
        let response: HttpResponse = error.into();
        assert_eq!(response.status_code, 400);
        assert_eq!(response.status_text, "Bad Request");
        assert_eq!(response.body, "Invalid Request");

        let error = AspirinEatsError::NotFound;
        let response: HttpResponse = error.into();
        assert_eq!(response.status_code, 404);
        assert_eq!(response.status_text, "Not Found");
        assert_eq!(response.body, "Resource not found");

        let error = AspirinEatsError::MethodNotAllowed;
        let response: HttpResponse = error.into();
        assert_eq!(response.status_code, 405);
        assert_eq!(response.status_text, "Method Not Allowed");
        assert_eq!(response.body, "Method not allowed");

        let error = AspirinEatsError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        let response: HttpResponse = error.into();
        assert_eq!(response.status_code, 500);
        assert_eq!(response.status_text, "Internal Server Error");
        assert_eq!(response.body, "Internal Server Error");
    }
}
