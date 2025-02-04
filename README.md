# Hnefatafl Terminal

Hnefatafl Terminal is a terminal-based implementation of the ancient Viking board game Hnefatafl. This project allows you to play the game against another player over a network.

## Features

- Play Hnefatafl against another player over a network.
- Option to play 7x7 (Brandubh) or 11x11 (Copenhagen).
- Save and load game states.
- Display the current board state and game status.

## Requirements

- Rust (latest stable version)
- Cargo (latest stable version)

## Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/farl-opa/hnefatafl-terminal.git
    cd hnefatafl-terminal
    ```

2. Build the project:
    ```sh
    cargo build
    ```

## Usage

1. Start the server:
    ```sh
    cargo run
    ```

2. Connect a client:
    Copy the client_example_brandubh.rs or client_example_copenhagen.rs code to a new project, add the necessary crates to your cargo.toml file,
    ```sh
    [dependencies]
    rand = "0.9"
    serde = { version = "1", features = ["derive"] }
    serde_json = "1.0"
    ```
    then connect your client to the server 
    ```sh
    cargo run
    ```
3. For the game to start, connect two clients from two different terminals.

## Project Structure

- `src/main.rs`: Contains the main server logic.
- `src/client_example.rs`: Contains the client logic.
- `src/brandubh.rs`, `src/copenhagen.rs` : Contains the game logic and data structures.
- `Cargo.toml`: Project dependencies and metadata.

## Switching between game modes

On lines 8-14 of main.rs you can choose whether to play 7x7 or 11x11:
    ```sh
    mod brandubh;
    use brandubh::{GameState, Cell, CellType};
    mod copenhagen;
    use copenhagen::{GameState, Cell, CellType};
    ```


## Acknowledgements

- [Rust Programming Language](https://www.rust-lang.org/)
- [Serde](https://serde.rs/)
- [Tokio](https://tokio.rs/)
- [Warp](https://github.com/seanmonstar/warp)
