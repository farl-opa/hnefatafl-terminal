use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};
mod brandubh;
use brandubh::{GameState, Cell, CellType};
use std::fs::File;
use std::fs::OpenOptions;

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    from: (usize, usize),
    to: (usize, usize),
}

#[derive(Serialize, Deserialize, Debug)]
struct BoardState {
    board: HashMap<String, CellType>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GameStateResponse {
    board_state: BoardState,
    current_turn: CellType,
    winner: Option<CellType>,
}

fn process_move(
    game: &mut GameState,
    game_move: Move,
    role: CellType,
    clients: &Arc<Mutex<HashMap<usize, TcpStream>>>,
    stats: &Arc<Mutex<GameStats>>
) -> Result<(), String> {
    if game.current_turn.cell_type != role {
        return Err("It's not your turn".to_string());
    }

    game.process_click(game_move.from.0, game_move.from.1)?;
    if let Err(err) = game.process_click(game_move.to.0, game_move.to.1) {
        game.winner = Some(Cell {
            cell_type: match role {
                CellType::Attacker => CellType::Defender,
                CellType::Defender => CellType::Attacker,
                _ => CellType::Empty,
            },
            is_corner: false,
            is_throne: false,
            is_selected: false,
            is_possible_move: false,
        });
        println!("Invalid move from {}: {}",role, err);
        println!("Game over! Winner: {:?}", game.winner);
        return Err(format!("Invalid move: {}", err));
    }

    // Increment move counter based on the current role
    match role {
        CellType::Attacker => game.attacker_moves += 1,
        CellType::Defender => game.defender_moves += 1,
        CellType::King => game.defender_moves += 1,
        _ => {}
    }

    let mut board_state = HashMap::new();
    for (row_idx, row) in game.board.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let key = format!("({}, {})", row_idx, col_idx);
            board_state.insert(key, cell.cell_type);
        }
    }

    let response = GameStateResponse {
        board_state: BoardState { board: board_state },
        current_turn: game.current_turn.cell_type,
        winner: game.winner.map(|cell| cell.cell_type),
    };

    let response_json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
    let board_state_json = serde_json::to_string(&response.board_state).expect("Unable to serialize board state");
    
    let file_name = {
        let stats_lock = stats.lock().unwrap();
        let file_name = format!("board_state_game_{}.txt", stats_lock.total_games + 1);
        file_name
    };

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_name)
        .map_err(|e| format!("Failed to open board_state.txt: {}", e))?;

    file.write_all(board_state_json.as_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    let clients_lock = clients.lock().unwrap();
    for (id, mut client_stream) in clients_lock.iter() {
        if let Err(e) = client_stream.write_all(response_json.as_bytes()) {
            eprintln!("Failed to write to client {}: {}", id, e);
        }
    }

    if let Some(winner) = game.winner {
        println!("Game over! Winner: {:?}", winner.cell_type);
        println!("Attacker moves: {}", game.attacker_moves);
        println!("Defender moves: {}", game.defender_moves);
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    game_state: Arc<Mutex<GameState>>,
    clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
    client_id: usize,
    role: CellType,
    stats: &Arc<Mutex<GameStats>>,
) {
    let mut buffer = [0; 1024];
    let mut games_played = 0;

    loop {
        let size = match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(size) => size,
            Err(e) => {
                eprintln!("Failed to read from client {}: {}", client_id, e);
                break;
            }
        };

        let received_str = String::from_utf8_lossy(&buffer[..size]);
        match serde_json::from_str::<Move>(&received_str) {
            Ok(game_move) => {
                let mut game = game_state.lock().unwrap();
                if let Err(err) = process_move(&mut game, game_move, role, &clients, stats) {
                    let error_message = format!("{{\"error\":\"{}\"}}", err);
                    if let Err(e) = stream.write_all(error_message.as_bytes()) {
                        eprintln!("Failed to write error to client {}: {}", client_id, e);
                    }
                }

                if let Some(winner) = game.winner {
                    // Update statistics
                    let mut guard_stats = stats.lock().unwrap();
                    guard_stats.total_games += 1;
                    guard_stats.total_attacker_moves += game.attacker_moves;
                    guard_stats.total_defender_moves += game.defender_moves;
                    match winner.cell_type {
                        CellType::Attacker => guard_stats.attacker_wins += 1,
                        CellType::Defender => guard_stats.defender_wins += 1,
                        _ => {}
                    }
                    game.winner = None;

                    games_played += 1;

                    if games_played < 50 {
                        *game = GameState::new(1); // Reset the game state
                        drop(game);
                        drop(guard_stats);
                        initialize_game(&game_state, &clients, &stats);
                    } else {
                        println!("All games finished.");
                        println!(
                            "Average attacker moves: {:.2}",
                            guard_stats.total_attacker_moves as f64 / guard_stats.total_games as f64
                        );
                        println!(
                            "Average defender moves: {:.2}",
                            guard_stats.total_defender_moves as f64 / guard_stats.total_games as f64
                        );
                        println!("Attacker wins: {}", guard_stats.attacker_wins);
                        println!("Defender wins: {}", guard_stats.defender_wins);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to deserialize move from client {}: {}", client_id, e);
            }
        }
    }

    clients.lock().unwrap().remove(&client_id);
    println!("Client {} disconnected", client_id);
}


fn initialize_game(
    game_state: &Arc<Mutex<GameState>>,
    clients: &Arc<Mutex<HashMap<usize, TcpStream>>>,
    stats: &Arc<Mutex<GameStats>>,
) {
    let mut game = game_state.lock().unwrap();

    game.current_turn = Cell {
        cell_type: CellType::Attacker,
        is_corner: false,
        is_throne: false,
        is_selected: false,
        is_possible_move: false,
    };
    game.attacker_moves = 0;
    game.defender_moves = 0;


    // Send the messages to the clients
    let clients_lock = clients.lock().unwrap();
    for (id, mut client_stream) in clients_lock.iter() {
        let start_message = if *id == 1 {
            "{\"message\":\"Game has started\", \"role\":\"Attacker\"}"
        } else {
            "{\"message\":\"Game has started\", \"role\":\"Defender\"}"
        };
        if let Err(e) = client_stream.write_all(start_message.as_bytes()) {
            eprintln!("Failed to write start message to client {}: {}", id, e);
        }
    }
    let mut board_state = HashMap::new();
    for (row_idx, row) in game.board.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let key = format!("({}, {})", row_idx, col_idx);
            board_state.insert(key, cell.cell_type);
        }
    }

    let response = GameStateResponse {
        board_state: BoardState { board: board_state },
        current_turn: game.current_turn.cell_type,
        winner: None,
    };
    
    let file_name = {
        let stats_lock = stats.lock().unwrap();
        let file_name = format!("board_state_game_{}.txt", stats_lock.total_games + 1);
        println!("Starting game {}", stats_lock.total_games + 1);
        file_name
    };

    let mut file = File::create(&file_name).expect("Unable to create file");
    let board_state_json = serde_json::to_string(&response.board_state).expect("Unable to serialize board state");
    file.write_all(board_state_json.as_bytes()).expect("Unable to write data");
    file.write_all(b"\n").expect("Unable to write newline");
    drop(file);

    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| {
        "{\"error\":\"Failed to serialize game state\"}".to_string()
    });

    for (id, mut client_stream) in clients_lock.iter() {
        if let Err(e) = client_stream.write_all(response_json.as_bytes()) {
            eprintln!("Failed to write board state to client {}: {}", id, e);
        }
    }    
}


struct GameStats {
    total_games: u32,
    total_attacker_moves: u32,
    total_defender_moves: u32,
    attacker_wins: u32,
    defender_wins: u32,
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let game_state = Arc::new(Mutex::new(GameState::new(1)));
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let stats = Arc::new(Mutex::new(GameStats {
        total_games: 0,
        total_attacker_moves: 0,
        total_defender_moves: 0,
        attacker_wins: 0,
        defender_wins: 0,
    }));
    let mut client_id = 0;
    let mut roles_assigned = 0;

    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream?;
        let game_state = Arc::clone(&game_state);
        let clients = Arc::clone(&clients);
        let stats = Arc::clone(&stats);

        client_id += 1;
        clients.lock().unwrap().insert(client_id, stream.try_clone()?);

        let role = match roles_assigned {
            0 => { roles_assigned += 1; CellType::Attacker },
            1 => { roles_assigned += 1; CellType::Defender },
            _ => CellType::Empty,
        };

        println!("Client {} connected with role {:?}", client_id, role);

        if roles_assigned == 2 {
            println!("Both clients connected. Starting the game.");
            initialize_game(&game_state, &clients, &stats);
        }

        thread::spawn(move || {
            handle_client(stream, game_state, clients, client_id, role, &stats);
        });
    }

    Ok(())
}
