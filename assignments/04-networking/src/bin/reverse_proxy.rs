use std::env;
use std::io::Read;
use std::net::{Ipv4Addr, TcpListener, TcpStream, ToSocketAddrs},

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <proxy-from> <proxy-to>", args[0]);
        std::process::exit(2);
    }

    let proxy_addr = &args[1];
    let listner = TcpListener::bind(proxy_addr.to_string()).unwrap();
    let origin_addr = &args[2];
    loop {
        for stream in listner.incoming(){
            match stream {
                Ok(stream) =>,
                Err(E) =>todo!(),
            }
        }
    }
    
}
fn handle_stream(stream:TcpStream,server_address:SocketAddr){
    let mut buffer:[u8;1024] = [0;1024]; 
    //First we read the message from the stream to our buffer
    let size = stream.read(buf).unwrap();
    //To determine how we forward we want to check if the stream is a response
    //from our server or a client buffer that will hold read message
    if stream.local_addr().unwrap() == server_address{
        //here we want to take the message from the server and pass it along to the client,
        
    }
    else{
        //here we want to take the message from our client and pass it along
    }

}
