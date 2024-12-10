use aspirin_eats::error::AspirinEatsError;
use std::env;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        eprintln!("Usage: {} <proxy-from> <proxy-to>", args[0]);
        std::process::exit(2);
    }

    let proxy_addr = &args[1];
    let listner = TcpListener::bind(proxy_addr.to_string()).unwrap();
    let origin_addr = &args[2];
    let mut lastadder = String::new();
    loop {
        for stream in listner.incoming() {
            match stream {
                Ok(mut stream) => {
                    //get in a message from a stream, if this is the origin we want to forward to our last client,
                    // if it is not the origin we want to send it to our origin
                    let buf = message_reader(&mut stream).expect("Error Reading");
                    if stream.local_addr().unwrap()
                        == SocketAddr::from_str("127.0.0.1:2000").unwrap()
                    {
                        //the stream is from our oriign serve we need to send our client
                        let mut client =
                            TcpStream::connect(SocketAddr::from_str(&lastadder).unwrap())
                                .expect("Failed to Connect to Client");
                        message_writer(&mut client, buf).expect("Error Writing to Client");
                    } else {
                        //save the address of our client
                        lastadder = stream.local_addr().unwrap().to_string();
                        //stream is from our client, we need to send to our origin server
                        let mut origin =
                            TcpStream::connect(origin_addr).expect("Failed to connect to origin");
                        message_writer(&mut origin, buf).expect("Error Writing to Origin");
                    }
                }
                Err(e) => eprint!("Error{}", e),
            }
        }
    }
}
//Returns u8 with message
fn message_reader<R: Read>(stream: &mut R) -> Result<(usize, [u8; 1024]), AspirinEatsError> {
    let mut buf: [u8; 1024] = [0; 1024];
    //only parse the section of the read buffer that is new
    match stream.read(&mut buf) {
        Ok(end) => Ok((end, buf)),
        Err(e) => Err(AspirinEatsError::Io(e)),
    }
}
//writes the given message to a
fn message_writer<W: Write>(
    stream: &mut W,
    buffer: (usize, [u8; 1024]),
) -> Result<(), AspirinEatsError> {
    let buf = &buffer.1[..buffer.0];
    match stream.write(buf) {
        Ok(_) => Ok(()),
        Err(e) => Err(AspirinEatsError::Io(e)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;
    use std::io::Error;
    use std::io::ErrorKind;
    #[test]
    fn testmessage_reader() {
        let data = "Hello World".as_bytes();
        let mut cursor: Cursor<&[u8]> = Cursor::new(data);
        let result = message_reader(&mut cursor);
        let out = result.unwrap();
        assert_eq!(&out.1[..out.0], "Hello World".as_bytes());
        //mock an error
        let mut cursor = ErrorCursor {
            _inner: Cursor::new(Vec::new()),
        };
        let result = message_reader(&mut cursor);
        assert_eq!(true, result.is_err())
    }
    #[test]
    fn testmessage_writer() {
        let mut writer = Cursor::new(Vec::new());
        let buf: [u8; 1024] = [0; 1024]; // Fill with zero bytes
        let buffer = (1024, buf);
        // Call the messageWriter function
        let result = message_writer(&mut writer, buffer);
        // Assert that the result is Ok
        assert!(result.is_ok());
        //create a failure
        let mut cursor = ErrorCursor {
            _inner: Cursor::new(Vec::new()),
        };
        let result = message_writer(&mut cursor, buffer);
        assert_eq!(true, result.is_err())
    }
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
    impl Read for ErrorCursor {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(Error::new(
                ErrorKind::Other,
                "Simulated write error for testing",
            ))
        }
    }
}
