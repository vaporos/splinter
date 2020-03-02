
use splinter::mesh::Mesh;
use splinter::transport::{
    raw::RawTransport,
    Incoming,
    Transport,
};
use rustyline;
use std::thread;

fn main() {
    let mesh = Mesh::new(512, 512);
    let mut transport = RawTransport::default();

    let mut rl = rustyline::Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                match parse(line) {
                    Ok(Command::Exit) => {
                        println!("Exiting...");
                        break;
                    }
                    Ok(Command::Listen(endpoint)) => {
                        let mut listener = transport.listen(&endpoint).expect("failed to listen");
                        let mesh_clone = mesh.clone();
                        let _  = thread::spawn(move || {
                            for connection_result in listener.incoming() {
                            println!("Recieved connection");
                            let connection = match connection_result {
                                Ok(c) => c,
                                Err(err) => return println!("{:?}", err),
                            };

                            mesh_clone.add(connection).unwrap();
                        }
                    });
                        println!("Listening on {}", endpoint);
                    }
                    Err(_) => {
                        println!("Parse error");
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("User interrupt, exiting...");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("EOF, exiting...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

enum Command {
    Exit,
    Listen(String)
}

struct ParseError;

fn parse(line: String) -> Result<Command, ParseError> {
    let mut iter = line.split_whitespace();
    match iter.next() {
        Some("exit") => {
            Ok(Command::Exit)
        }
        Some("listen") => {
            match iter.next() {
                Some(endpoint) => {
                    Ok(Command::Listen(endpoint.to_owned()))
                }
                None => Err(ParseError{})
            }
        }
        _ => Err(ParseError{})
    }
}
