use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
mod brandubh;
use brandubh::GameState;

#[derive(Serialize, Deserialize)]
struct Move {
    from: (usize, usize),
    to: (usize, usize),
}

fn handle_client(
    mut stream: TcpStream,
    game_state: Arc<Mutex<GameState>>,
    clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
    client_id: usize,
) {
    let mut buffer = [0; 512];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Client disconnected
            Ok(bytes_read) => {
                let move_data: Result<Move, _> = serde_json::from_slice(&buffer[..bytes_read]);
                if let Ok(move_data) = move_data {
                    let mut game = game_state.lock().unwrap();

                    let response = match game.process_click(move_data.from.0, move_data.from.1) {
                        Ok(_) => match game.process_click(move_data.to.0, move_data.to.1) {
                            Ok(_) => serde_json::to_string(&*game).unwrap_or_else(|_| {
                                "{\"error\":\"Failed to serialize game state\"}".to_string()
                            }),
                            Err(e) => format!("{{\"error\":\"{}\"}}", e),
                        },
                        Err(e) => format!("{{\"error\":\"{}\"}}", e),
                    };

                    if let Err(e) = stream.write_all(response.as_bytes()) {
                        eprintln!("Failed to write to client {}: {}", client_id, e);
                        break;
                    }
                } else {
                    let error_message = "{\"error\":\"Invalid move data\"}";
                    if let Err(e) = stream.write_all(error_message.as_bytes()) {
                        eprintln!("Failed to write error message to client {}: {}", client_id, e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from client {}: {}", client_id, e);
                break;
            }
        }
    }

    // Cleanup client on disconnect
    let mut clients = clients.lock().unwrap();
    clients.remove(&client_id);
    println!("Client {} disconnected", client_id);
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let game_state = Arc::new(Mutex::new(GameState::new(1)));
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let mut client_id = 0;

    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let game_state = Arc::clone(&game_state);
                let clients = Arc::clone(&clients);

                client_id += 1;
                let client_id_copy = client_id;

                {
                    let mut clients_lock = clients.lock().unwrap();
                    clients_lock.insert(client_id, stream.try_clone()?);
                }

                thread::spawn(move || {
                    handle_client(stream, game_state, clients, client_id_copy);
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }

    Ok(())
}
