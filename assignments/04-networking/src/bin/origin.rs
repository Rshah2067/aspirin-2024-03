use aspirin_eats::db::AspirinEatsDb;
use aspirin_eats::error::AspirinEatsError;
use aspirin_eats::food::{Order, OrderRequest};
use aspirin_eats::http::HttpRequest;
use aspirin_eats::http::HttpResponse;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::str::FromStr;
/// Change this path to match where you want to store the database file
const DB_PATH: &str =
    "/home/rohan/Documents/Code/RUST/aspirin-2024-03/assignments/04-networking/aspirin_eats.db";
#[derive(Debug, PartialEq)]
enum RequestTypes {
    Get(Option<i64>),
    Add(OrderRequest),
    Delete(Option<i64>),
    Root,
}
fn main() {
    let db = AspirinEatsDb::from_path(DB_PATH).expect("Failed to open database");
    //Don't think I need error handling here as this is running on hardware Asprineats manages so we should know if this address is free or not
    let listner = TcpListener::bind("127.0.0.1:2000").unwrap();
    //Loop the server is constantly running, we should be accepting streams from the reverse proxy and then executing requests.
    loop {
        for stream in listner.incoming() {
            match stream {
                Ok(mut tcpstream) => {
                    //This is really ugly nesting but I can't figure out a better way to do this
                    match messagetostring(&mut tcpstream) {
                        Ok(msg) => {
                            match stringto_httprequest(msg) {
                                Ok(request) => {
                                    //after the correct manipulation has been made return the correct return message
                                    let response = querytoresponse(request, &db);
                                    //now send the response back to the client
                                    match transmit_response(&mut tcpstream, response.to_string()) {
                                        None => (),
                                        Some(e) => eprint!("Error Occured!{}", e),
                                    }
                                }
                                Err(e) => {
                                    transmit_response(
                                        &mut tcpstream,
                                        HttpResponse::from(e).to_string(),
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            transmit_response(&mut tcpstream, HttpResponse::from(e).to_string());
                        }
                    }
                }
                Err(e) => eprint!("Error Occured! {}", e),
            }
        }
    }
}
fn messagetostring<R: Read>(stream: &mut R) -> Result<String, AspirinEatsError> {
    let mut buf: [u8; 1024] = [0; 1024];
    //only parse the section of the read buffer that is new
    match stream.read(&mut buf) {
        Ok(end) => {
            //Error Handling the type conversion
            match String::from_utf8(buf[..end].to_vec()) {
                Ok(string) => Ok(string),
                Err(_) => Err(AspirinEatsError::InvalidRequest),
            }
        }
        Err(e) => Err(AspirinEatsError::Io(e)),
    }
}
fn stringto_httprequest(message: String) -> Result<HttpRequest, AspirinEatsError> {
    match HttpRequest::from_str(&message) {
        Ok(request) => Ok(request),
        Err(_) => Err(AspirinEatsError::InvalidRequest),
    }
}
//this function has a decent amount of nesting but I breaking it up into further pieces would be tedious
fn requesttoquery(request: HttpRequest) -> Result<RequestTypes, AspirinEatsError> {
    //first match the request type and then add the path
    match request.method.as_deref() {
        Some("GET") => {
            match request.path.as_deref() {
                Some("/") => Ok(RequestTypes::Root),
                Some("/orders") => Ok(RequestTypes::Get(None)),
                //If it is not one of these we will strip the prefix and parse the id
                Some(other) => match other.strip_prefix("/orders/") {
                    Some(number) => Ok(RequestTypes::Get(Some(number.parse()?))),
                    None => Err(AspirinEatsError::InvalidRequest),
                },
                None => Err(AspirinEatsError::InvalidRequest),
            }
        }
        Some("POST") => {
            //first check to see if something is in the body
            match request.body.as_deref() {
                //All of these are Methord not allowed errors
                Some("/") => Err(AspirinEatsError::MethodNotAllowed),
                Some("/orders") => Err(AspirinEatsError::MethodNotAllowed),
                //Now we check valid paths for this method
                Some(string) => match OrderRequest::from_str(string) {
                    Ok(order) => Ok(RequestTypes::Add(order)),
                    Err(_) => Err(AspirinEatsError::InvalidRequest),
                },
                None => Err(AspirinEatsError::InvalidRequest),
            }
        }
        Some("DELETE") => {
            match request.path.as_deref() {
                //We cannot have a call to root
                Some("/") => Err(AspirinEatsError::MethodNotAllowed),
                Some("/orders") => Ok(RequestTypes::Delete(None)),
                //If it is not one of these we will strip the prefix and parse the id
                Some(other) => match other.strip_prefix("/orders/") {
                    Some(number) => Ok(RequestTypes::Delete(Some(number.parse()?))),
                    None => Err(AspirinEatsError::InvalidRequest),
                },
                None => Err(AspirinEatsError::InvalidRequest),
            }
        }
        Some(_) => Err(AspirinEatsError::InvalidRequest),
        None => Err(AspirinEatsError::InvalidRequest),
    }
}
//I know this should be broken into two functions but if I do it that way it makes sending an error back a pain
fn querytoresponse(request: HttpRequest, db: &AspirinEatsDb) -> HttpResponse {
    match requesttoquery(request) {
        Ok(RequestTypes::Get(id)) => {
            match id {
                Some(id) => {
                    //check to see if the order id was valid
                    match db.get_order(id) {
                        Ok(Some(order)) => HttpResponse::new(200, "Ok", &order.to_string()),
                        Ok(None) => HttpResponse::new(400, "Bad Request", "No Order with Given ID"),
                        Err(e) => HttpResponse::from(AspirinEatsError::Database(e)),
                    }
                }
                None => match db.get_all_orders() {
                    Ok(orders) => HttpResponse::new(200, "Ok", &orderstostring(orders)),
                    Err(e) => HttpResponse::from(AspirinEatsError::Database(e)),
                },
            }
        }
        Ok(RequestTypes::Add(order_request)) => {
            let order = Order::from(order_request);
            match db.add_order(order) {
                Ok(id) => HttpResponse::new(201, "Created", &id.to_string()),
                Err(e) => HttpResponse::from(AspirinEatsError::Database(e)),
            }
        }
        Ok(RequestTypes::Delete(id)) => match id {
            Some(id) => match db.remove_order(id) {
                Ok(_) => HttpResponse::new(204, "No content", ""),
                Err(e) => HttpResponse::from(AspirinEatsError::Database(e)),
            },
            None => match db.reset_orders() {
                Ok(_) => HttpResponse::new(204, "No content", ""),
                Err(e) => HttpResponse::from(AspirinEatsError::Database(e)),
            },
        },
        Ok(RequestTypes::Root) => HttpResponse::new(200, "Ok", "Welcome to Aspirin Eats!"),
        Err(e) => HttpResponse::from(e),
    }
}
//helper function to convert a vector of orders to a body string
fn orderstostring(orders: Vec<Order>) -> String {
    let mut output = String::new();
    for order in orders {
        output.push_str(&order.to_string());
    }
    output
}
fn transmit_response<W: Write>(stream: &mut W, response: String) -> Option<AspirinEatsError> {
    let buf = response.as_bytes();
    match stream.write(buf) {
        Ok(_) => None,
        Err(e) => Some(AspirinEatsError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aspirin_eats::food::*;
    use std::io;
    use std::io::Cursor;
    use std::io::Error;
    use std::io::ErrorKind;
    #[test]
    fn test_get_order() {
        let db = AspirinEatsDb::in_memory().unwrap();
        //should create a an error response (order doesn't exist)
        let request = "GET /orders/1 HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(request).unwrap();
        let error_response = HttpResponse::new(400, "Bad Request", "No Order with Given ID");
        assert_eq!(error_response, querytoresponse(http_request, &db));
        //now we will add an order so it should succeed
        let order = get_test_order();
        let _ = db.add_order(get_test_order());
        let http_request = HttpRequest::from_str(request).unwrap();
        let response = HttpResponse::new(200, "Ok", &order.to_string());

        assert_eq!(response, querytoresponse(http_request, &db));
        //now test getting all orders
        let request = "GET /orders HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(request).unwrap();
        assert_eq!(response, querytoresponse(http_request, &db));
    }
    #[test]
    fn test_add_order() {
        let db = AspirinEatsDb::in_memory().unwrap();
        let mut request = String::from("POST /orders HTTP/1.1\r\nHost: localhost:8080\r\n\r\n");
        request.push_str(&get_test_order().to_string());
        let http_request = HttpRequest::from_str(&request).unwrap();
        let response = HttpResponse::new(201, "Created", "1");
        assert_eq!(response, querytoresponse(http_request, &db))
    }
    #[test]
    fn test_delete_order() {
        let db = AspirinEatsDb::in_memory().unwrap();
        //should create a an error response (order doesn't exist)
        let request = "DELETE /orders/1 HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let _order = get_test_order();
        let _ = db.add_order(get_test_order());
        let http_request = HttpRequest::from_str(request).unwrap();
        let response = HttpResponse::new(204, "No content", "");
        assert_eq!(response, querytoresponse(http_request, &db));
    }
    //tests all nominal cases
    #[test]
    fn test_request_to_query() {
        //Test GET all
        let request = "GET /orders HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(
            RequestTypes::Get(None),
            requesttoquery(http_request).unwrap()
        );
        // Get id
        let request = "GET /orders/1 HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(
            RequestTypes::Get(Some(1)),
            requesttoquery(http_request).unwrap()
        );
        //Delete all
        let request: &str =
            "DELETE /orders HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(
            RequestTypes::Delete(None),
            requesttoquery(http_request).unwrap()
        );
        //Delete ID
        let request = "DELETE /orders/1 HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(
            RequestTypes::Delete(Some(1)),
            requesttoquery(http_request).unwrap()
        );
        //Add
        let mut request = String::from("POST /orders/1 HTTP/1.1\r\nHost: localhost:8080\r\n\r\n");
        request.push_str(&get_test_order().to_string());
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(
            RequestTypes::Add(OrderRequest::from_str(&get_test_order().to_string()).unwrap()),
            requesttoquery(http_request).unwrap()
        );
        //root
        let request = "GET / HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(RequestTypes::Root, requesttoquery(http_request).unwrap());
    }
    #[test]
    fn test_request_errors() {
        //cant have a delete to root
        let request: &str = "DELETE / HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(true, requesttoquery(http_request).is_err());
        //can't get with no path
        let request: &str = "GET HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(true, requesttoquery(http_request).is_err());
        //cant have a Post to root
        let request: &str = "POST / HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(true, requesttoquery(http_request).is_err());
        //cant have a Post to orders/
        let request: &str =
            "POST /orders HTTP/1.1\r\nHost: localhost:8080\r\n\r\nthis is the body.";
        let http_request = HttpRequest::from_str(&request).unwrap();
        assert_eq!(true, requesttoquery(http_request).is_err());
    }
    #[test]
    fn test_transmit_response() {
        //Very hard to test error cases as the cursor that I am using will not give an error so chatgpt told me to write a custom implementation that errors
        let response = String::from("Hello World");
        let mut cursor = Cursor::new(vec![]);
        let result = transmit_response(&mut cursor, response.clone());
        assert_eq!(true, result.is_none());
        let mut cursor = ErrorCursor {
            _inner: Cursor::new(Vec::new()),
        };
        let result = transmit_response(&mut cursor, response);
        assert_eq!(false, result.is_none());
    }
    #[test]
    fn test_message_to_string() {
        let data = "Hello World".as_bytes();
        let mut cursor: Cursor<&[u8]> = Cursor::new(data);
        let result = messagetostring(&mut cursor);
        assert_eq!(result.unwrap(), "Hello World".to_string());
        //mock an error
        let data = vec![0, 159, 146, 150];
        let mut cursor: Cursor<&[u8]> = Cursor::new(&data);
        let result = messagetostring(&mut cursor);
        assert_eq!(true, result.is_err())
    }
    //helper functions that I may or may not have borrowed from bd.rs :)
    fn get_test_order() -> Order {
        Order {
            id: Some(1),
            customer: "Amit".to_string(),
            food: vec![MenuItem::Fries, MenuItem::Drink],
            status: OrderStatus::Pending,
            total: 8.0,
        }
    }
    //More junk I used to mock write errors
    struct ErrorCursor {
        _inner: Cursor<Vec<u8>>,
    }

    impl Write for ErrorCursor {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            // Simulate a write error
            Err(Error::new(
                ErrorKind::Other,
                "Simulated write error for testing",
            ))
        }

        fn flush(&mut self) -> io::Result<()> {
            // Simulate a flush error, if desired
            Err(Error::new(
                ErrorKind::Other,
                "Simulated write Flush for testing",
            ))
        }
    }
}
