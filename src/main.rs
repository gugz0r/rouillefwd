use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 5 || args[1] != "-p" || args[3] != "-d" {
        eprintln!("Usage: rouillefwd -p [listening_port] -d [destination_host:destination_port]");
        return Ok(());
    }

    let listening_port = &args[2];
    let destination = &args[4];
    let (destination_host, destination_port) = destination.split_once(':')
        .expect("Invalid destination format. Use [host]:[port].");

    let listen_address = format!("0.0.0.0:{}", listening_port);
    let listener = TcpListener::bind(&listen_address)?;
    println!("Listening on port {} and forwarding to {}:{}", listening_port, destination_host, destination_port);

    for client_stream in listener.incoming().flatten() {
        let destination_address = format!("{}:{}", destination_host, destination_port);
        
        thread::spawn(move || {
            if let Err(e) = handle_connection(client_stream, &destination_address) {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_connection(mut client_stream: TcpStream, destination_address: &str) -> io::Result<()> {
    let mut server_stream = TcpStream::connect(destination_address)?;
    println!("Connection established. Forwarding traffic to {}", destination_address);

    let mut client_stream_clone = client_stream.try_clone()?;
    let mut server_stream_clone = server_stream.try_clone()?;

    let client_to_server = thread::spawn(move || copy_data(&mut client_stream_clone, &mut server_stream));
    let server_to_client = thread::spawn(move || copy_data(&mut server_stream_clone, &mut client_stream));

    client_to_server.join().expect("client_to_server thread panicked")?;
    server_to_client.join().expect("server_to_client thread panicked")?;

    println!("Connection closed.");
    Ok(())
}

fn copy_data(from: &mut TcpStream, to: &mut TcpStream) -> io::Result<()> {
    io::copy(from, to)?;
    Ok(())
}