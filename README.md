# Hnefatafl Terminal

Hnefatafl Terminal is a terminal-based implementation of the ancient Viking board game Hnefatafl. This project allows you to play the game against another player over a network.

## Features

- Play Hnefatafl against another player over a network.
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
    Copy the client_example.rs code to a new project and 
    ```sh
    cargo run
    ```
3. For the game to start, connect two clients from two different terminals.

## Project Structure

- `src/main.rs`: Contains the main server logic.
- `src/client_example.rs`: Contains the client logic.
- `src/brandubh.rs`: Contains the game logic and data structures.
- `Cargo.toml`: Project dependencies and metadata.


## Acknowledgements

- [Rust Programming Language](https://www.rust-lang.org/)
- [Serde](https://serde.rs/)
- [Tokio](https://tokio.rs/)
- [Warp](https://github.com/seanmonstar/warp)
