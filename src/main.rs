use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};

// 7x7 version
mod brandubh;
use brandubh::{GameState, Cell, CellType};

// 11x11 version
//mod copenhagen;
//use copenhagen::{GameState, Cell, CellType};


use std::fs::File;
use std::fs::OpenOptions;
use std::time::{Instant, Duration};

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

struct GameStats {
    game_id: u32,
    total_games: u32,
    total_attacker_moves: u32,
    total_defender_moves: u32,
    attacker_wins: u32,
    defender_wins: u32,
    move_durations_attacker: Vec<Duration>,
    move_durations_defender: Vec<Duration>,
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

    let start_time = Instant::now();

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
        println!("Invalid move from {}: {}", role, err);
        println!("Game over! Winner: {:?}", game.winner);
        return Err(format!("Invalid move: {}", err));
    }

    let duration = start_time.elapsed();

    // Store the move duration
    let mut stats_lock = stats.lock().unwrap();
    if role == CellType::Attacker {
        stats_lock.move_durations_attacker.push(duration);
    } else {
        stats_lock.move_durations_defender.push(duration);
    }
    drop(stats_lock);


    match role {
        CellType::Attacker => game.attacker_moves += 1,
        CellType::Defender => game.defender_moves += 1,
        CellType::King => game.defender_moves += 1,
        _ => {}
    }

    let total_moves = game.attacker_moves + game.defender_moves;
    if total_moves >= 100 {
        println!("Game over! It's a draw after 100 moves.");
        game.winner = Some(Cell {
            cell_type: CellType::Empty, // Represents a draw
            is_corner: false,
            is_throne: false,
            is_selected: false,
            is_possible_move: false,
        });
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
        format!("results/board_state_game_{}_{}.txt", stats_lock.game_id, stats_lock.total_games + 1)
    };

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_name)
        .map_err(|e| format!("Failed to open {}: {}", file_name, e))?;

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

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_name)
            .map_err(|e| format!("Failed to open {}: {}", file_name, e))?;
        let win_string = format!("Winner: {:?}", winner.cell_type);
        file.write_all(win_string.as_bytes()).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        file.flush().map_err(|e| e.to_string())?;
        drop(file);
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
                    let mut guard_stats = stats.lock().unwrap();
                    guard_stats.total_games += 1;
                    
                    match winner.cell_type {
                        CellType::Attacker => {
                            guard_stats.attacker_wins += 1;
                            guard_stats.total_attacker_moves += game.attacker_moves;
                        },
                        CellType::Defender => {
                            guard_stats.defender_wins += 1;
                            guard_stats.total_defender_moves += game.defender_moves;
                        },
                        _ => {}
                    }
                    game.winner = None;

                    if guard_stats.total_games >= 100 {
                        println!("All games finished for session {}.", guard_stats.game_id);
                        println!(
                            "Average attacker moves per winning game: {:.2}",
                            guard_stats.total_attacker_moves as f64 / guard_stats.total_games as f64
                        );
                        println!(
                            "Average defender moves per winning game: {:.2}",
                            guard_stats.total_defender_moves as f64 / guard_stats.total_games as f64
                        );
                        println!(
                            "Average move duration for attacker: {:?}",
                            guard_stats.move_durations_attacker.iter().sum::<Duration>() / guard_stats.move_durations_attacker.len() as u32
                        );
                        println!(
                            "Average move duration for defender: {:?}",
                            guard_stats.move_durations_defender.iter().sum::<Duration>() / guard_stats.move_durations_defender.len() as u32
                        );
                        println!("Attacker wins: {}", guard_stats.attacker_wins);
                        println!("Defender wins: {}", guard_stats.defender_wins);

                        let results_file_name = format!("results_session_{}.txt", guard_stats.game_id);
                        let mut results_file = File::create(&results_file_name).expect("Unable to create results file");
                        writeln!(results_file, "Session ID: {}", guard_stats.game_id).expect("Unable to write to results file");
                        writeln!(results_file, "Total games: {}", guard_stats.total_games).expect("Unable to write to results file");
                        writeln!(results_file, "Average attacker moves per winning game: {:.2}", guard_stats.total_attacker_moves as f64 / guard_stats.total_games as f64).expect("Unable to write to results file");
                        writeln!(results_file, "Average defender moves per winning game: {:.2}", guard_stats.total_defender_moves as f64 / guard_stats.total_games as f64).expect("Unable to write to results file");
                        writeln!(results_file, "Average move duration for attacker: {:?}", guard_stats.move_durations_attacker.iter().sum::<Duration>() / guard_stats.move_durations_attacker.len() as u32).expect("Unable to write to results file");
                        writeln!(results_file, "Average move duration for defender: {:?}", guard_stats.move_durations_defender.iter().sum::<Duration>() / guard_stats.move_durations_defender.len() as u32).expect("Unable to write to results file");
                        writeln!(results_file, "Attacker wins: {}", guard_stats.attacker_wins).expect("Unable to write to results file");
                        writeln!(results_file, "Defender wins: {}", guard_stats.defender_wins).expect("Unable to write to results file");
                        drop(results_file);

                        // Shutdown all client streams
                        let mut clients_lock = clients.lock().unwrap();
                        for (id, client_stream) in clients_lock.iter_mut() {
                            if let Err(e) = client_stream.shutdown(Shutdown::Both) {
                                eprintln!("Failed to shutdown client {}: {}", id, e);
                            }
                        }
                        clients_lock.clear();
                        break;
                    } else {
                        *game = GameState::new(1);
                        drop(game);
                        drop(guard_stats);
                        initialize_game(&game_state, &clients, &stats);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to deserialize move from client {}: {}", client_id, e);
            }
        }
    }

    if let Err(e) = stream.shutdown(Shutdown::Both) {
        eprintln!("Failed to shutdown client {}: {}", client_id, e);
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
        format!("results/board_state_game_{}_{}.txt", stats_lock.game_id, stats_lock.total_games + 1)
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

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let mut pending_clients: Vec<TcpStream> = Vec::new();
    let mut game_id_counter = 1;

    println!("Server listening on port 7878");

    for stream in listener.incoming() {
        let stream = stream?;
        pending_clients.push(stream);

        if pending_clients.len() >= 2 {
            let client1 = pending_clients.remove(0);
            let client2 = pending_clients.remove(0);
            let game_id = game_id_counter;
            game_id_counter += 1;

            thread::spawn(move || {
                let game_state = Arc::new(Mutex::new(GameState::new(1)));
                let clients = Arc::new(Mutex::new(HashMap::new()));
                let stats = Arc::new(Mutex::new(GameStats {
                    game_id,
                    total_games: 0,
                    total_attacker_moves: 0,
                    total_defender_moves: 0,
                    attacker_wins: 0,
                    defender_wins: 0,
                    move_durations_attacker: Vec::new(),
                    move_durations_defender: Vec::new(),
                }));

                {
                    let mut clients_lock = clients.lock().unwrap();
                    clients_lock.insert(1, client1.try_clone().unwrap());
                    clients_lock.insert(2, client2.try_clone().unwrap());
                }

                let game_state_clone1 = Arc::clone(&game_state);
                let clients_clone1 = Arc::clone(&clients);
                let stats_clone1 = Arc::clone(&stats);
                let handle1 = thread::spawn(move || {
                    handle_client(
                        client1,
                        game_state_clone1,
                        clients_clone1,
                        1,
                        CellType::Attacker,
                        &stats_clone1,
                    );
                });

                let game_state_clone2 = Arc::clone(&game_state);
                let clients_clone2 = Arc::clone(&clients);
                let stats_clone2 = Arc::clone(&stats);
                let handle2 = thread::spawn(move || {
                    handle_client(
                        client2,
                        game_state_clone2,
                        clients_clone2,
                        2,
                        CellType::Defender,
                        &stats_clone2,
                    );
                });

                initialize_game(&game_state, &clients, &stats);

                handle1.join().unwrap();
                handle2.join().unwrap();
            });
        }
    }

    Ok(())

}