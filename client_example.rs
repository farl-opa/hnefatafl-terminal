use std::collections::HashMap;
use std::io::{self, Write, Read};
use std::net::TcpStream;
use serde::{Serialize, Deserialize};
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    from: (usize, usize),
    to: (usize, usize),
}

#[derive(Serialize, Deserialize, Clone)]
struct ServerMessage {
    message: Option<String>,
    role: Option<String>,
    board_state: Option<BoardState>,
    current_turn: Option<String>,
    winner: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct BoardState {
    board: HashMap<String, String>,
}

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    println!("Connected to the server");

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..n]);
    println!("Server response: {}", response);

    let mut server_message: ServerMessage = ServerMessage {
        message: None,
        role: None,
        board_state: None,
        current_turn: None,
        winner: None,
    };

    for part in response.split("}{") {
        let json_str = if part.starts_with('{') && part.ends_with('}') {
            part.to_string()
        } else if part.starts_with('{') {
            format!("{}{}", part, "}")
        } else {
            format!("{}{}", "{", part)
        };

        match serde_json::from_str(&json_str) {
            Ok(msg) => server_message = msg,
            Err(e) => {
                eprintln!("Failed to parse server message: {}", e);
                return Ok(());
            }
        }
    }

    let mut player_role = String::new();

    if let Some(message) = server_message.message {
        if message == "Game has started" {
            if let Some(role) = server_message.role {
                player_role = role.clone();
                if role == "Attacker" {
                    if let Some(board_state) = server_message.board_state.clone() {
                        send_move(&mut stream, &board_state.board, &role)?;
                    }
                } else {
                    println!("Waiting for the opponent's move...");
                }
            }
        }
    }

    loop {
        let n = stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..n]);

        for part in response.split("}{") {
            let json_str = if part.starts_with('{') && part.ends_with('}') {
                part.to_string()
            } else if part.starts_with('{') {
                format!("{}{}", part, "}")
            } else {
                format!("{}{}", "{", part)
            };

            match serde_json::from_str(&json_str) {
                Ok(msg) => server_message = msg,
                Err(e) => {
                    eprintln!("Failed to parse server message: {}", e);
                    continue;
                }
            }
        }

        if let Some(winner) = server_message.winner.clone() {
            println!("Game over! The winner is: {}", winner);
        }

        if let Some(board_state) = server_message.board_state.clone() {
            println!("Board state: {:?}", board_state.board);
        }

        if let Some(current_turn) = server_message.current_turn.clone() {
            if current_turn == player_role && server_message.winner.is_none() {
                if let Some(ref board_state) = server_message.board_state {
                    send_move(&mut stream, &board_state.board, &player_role)?;
                } else {
                    println!("No board state available for the current turn.");
                }
            } else {
                println!("Waiting for the opponent's move...");
            }
        }
    }
}

fn send_move(stream: &mut TcpStream, board: &HashMap<String, String>, role: &str) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let mut pieces: Vec<(usize, usize)> = Vec::new();

    // Collect all pieces of the current player
    for (pos, piece) in board {
        let coords: Vec<usize> = pos[1..pos.len()-1].split(", ").map(|s| s.parse().unwrap()).collect();
        if (role == "Attacker" && piece == "Attacker") || (role == "Defender" && (piece == "Defender" || piece == "King")) {
            pieces.push((coords[0], coords[1]));
        }
    }

    // Find a random valid move
    loop {
        // Select a random piece
        let piece = pieces[rng.gen_range(0..pieces.len())];
        let directions = vec![(1, 0), (-1, 0), (0, 1), (0, -1)];
        let direction = directions[rng.gen_range(0..directions.len())];

        // Start from the piece's position
        let mut to = (piece.0 as isize, piece.1 as isize);

        // Collect all valid cells in the chosen direction
        let mut valid_positions = vec![];
        for _ in 1..6 {
            to = (to.0 + direction.0, to.1 + direction.1);

            // Check if the move is out of bounds
            if to.0 < 0 || to.0 >= 6 || to.1 < 0 || to.1 >= 6 {
                break;
            }

            // Check if the destination is a corner or the middle of the board
            if (to.0 == 0 && to.1 == 0)
                || (to.0 == 0 && to.1 == 6)
                || (to.0 == 6 && to.1 == 0)
                || (to.0 == 6 && to.1 == 6)
                || (to.0 == 3 && to.1 == 3)
            {
                break;
            }

            // Check if the cell is empty
            if board
                .get(&format!("({}, {})", to.0, to.1))
                .unwrap_or(&"Empty".to_string())
                != "Empty"
            {
                break;
            }

            // Add this position to the list of valid positions
            valid_positions.push((to.0 as usize, to.1 as usize));
        }

        // If there are no valid positions, continue to the next piece/direction
        if valid_positions.is_empty() {
            continue;
        }

        // Choose a random valid position to move to
        let random_destination = valid_positions[rng.gen_range(0..valid_positions.len())];

        // Send the move to the server
        let game_move = Move {
            from: piece,
            to: random_destination,
        };

        let serialized_move = serde_json::to_string(&game_move).unwrap();
        stream.write_all(serialized_move.as_bytes())?;
        println!("Move sent to the server: {:?}", game_move);
        return Ok(());
    }


}