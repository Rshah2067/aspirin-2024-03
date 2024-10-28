use std::io::Read;
use std::net::{ Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use std::net::SocketAddr;
use aspirin_eats::db::AspirinEatsDb;
use aspirin_eats::http::HttpRequest;
use aspirin_eats::food::Order;
use serde_json::{json, Value};
/// Change this path to match where you want to store the database file
const DB_PATH: &str =
    "/home/rohan/Documents/Code/RUST/aspirin-2024-03/assignments/04-networking/src/db.rs";

fn main() {
    let db = AspirinEatsDb::from_path(DB_PATH).expect("Failed to open database");
    //bind our listner, we only want to accept communcation from our reverse proxy
    //Don't think I need error handling here as this is running on hardware Asprineats manages so we should know if this address is free or not
    let listner = TcpListener::bind("127.0.0.1:2000").unwrap();
    //Loop the server is constantly running, we should be accepting streams from the reverse proxy and then executing requests. 
    loop {
        for stream in listner.incoming(){
            match stream{
                Ok(mut Tcpstream) =>{
                    //first we want to check to see if the stream is from the sender
                    if check_sender(&Tcpstream){
                        // now we want to convert the request message into a 
                    }
                },
                Err(e) =>println!("{}",e)
            }
        }
    }
    let request = "";
    let HTTPrequest = HttpRequest::from_str(&request).unwrap();
    

}
fn check_sender(stream:&TcpStream)->bool{
   stream.local_addr().unwrap() == SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1),5000))
}
mod tests {
   use serde_json;
   use aspirin_eats::db;
   use aspirin_eats::food::*;
   use super::*;
    #[test]
    fn create_order(){
        let test = Order::from_str(&json!(get_test_order()).to_string()).unwrap();
        assert_eq!(test,get_test_order())
    }
    fn get_test_order() -> Order {
        Order {
            id: None,
            customer: "Amit".to_string(),
            food: vec![MenuItem::Fries, MenuItem::Drink],
            status: OrderStatus::Pending,
            total: 8.0,
        }
    }
}