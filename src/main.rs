use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command-line arguments
    if args.len() != 5 || args[1] != "-p" || args[3] != "-d" {
        eprintln!("Usage: rouillefwd -p [listening_port] -d [destination_host:destination_port]");
        return;
    }

    let listening_port = &args[2];
    let destination = &args[4];
    let destination_parts: Vec<&str> = destination.split(':').collect();

    if destination_parts.len() != 2 {
        eprintln!("Invalid destination format. Use [host]:[port].");
        return;
    }

    let destination_host = destination_parts[0];
    let destination_port = destination_parts[1];

    let listen_address = format!("0.0.0.0:{}", listening_port);

    // Start the TCP listener
    let listener = TcpListener::bind(&listen_address).expect("Could not bind to port");

    println!("Listening on port {} and forwarding to {}:{}", listening_port, destination_host, destination_port);

    // Accept connections and spawn a new thread to handle each one
    for stream in listener.incoming() {
        match stream {
            Ok(mut client_stream) => {
                let destination_host = destination_host.to_string();
                let destination_port = destination_port.to_string();
                
                thread::spawn(move || {
                    // Connect to the destination server
                    let destination_address = format!("{}:{}", destination_host, destination_port);
                    match TcpStream::connect(&destination_address) {
                        Ok(mut server_stream) => {
                            println!("Connection established. Forwarding traffic to {}", destination_address);

                            // Create threads to handle data transfer in both directions
                            let mut client_stream_clone = client_stream.try_clone().expect("Failed to clone client stream");
                            let mut server_stream_clone = server_stream.try_clone().expect("Failed to clone server stream");

                            let client_to_server = thread::spawn(move || {
                                let _ = copy_data(&mut client_stream_clone, &mut server_stream);
                            });

                            let server_to_client = thread::spawn(move || {
                                let _ = copy_data(&mut server_stream_clone, &mut client_stream);
                            });

                            // Wait for both threads to finish
                            let _ = client_to_server.join();
                            let _ = server_to_client.join();

                            println!("Connection closed.");
                        }
                        Err(e) => {
                            eprintln!("Failed to connect to destination: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

// Helper function to copy data between streams
fn copy_data(from: &mut TcpStream, to: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 4096];
    loop {
        let bytes_read = from.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        to.write_all(&buffer[..bytes_read])?;
    }
    Ok(())
}
