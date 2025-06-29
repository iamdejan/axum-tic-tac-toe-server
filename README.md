# Axum Tic-Tac-Toe Server

This is a server for tic-tac-toe games, made by Axum and powered by WebSockets.

## Local Setup

### Prerequisites

Here's what you need to setup for the first time:
1. Install Rust language compiler and its toolkits (e.g. `cargo`). You can use Rustup (e.g. in Homebrew: `brew install rustup`) for that.
2. Install `mkcert` by following the guide on [its repository](https://github.com/FiloSottile/mkcert).
3. Download or clone this repo.
4. Run `make cert/generate`.
5. Install `wscat` with this command: `npm install --global wscat`. This would be useful to test WebSocket connection without Postman or Insomnia.

### Run the server

Run `cargo run`. Expect longer times for the first run, because it's downloading and building all dependencies.

To test the server if it's running or not, you can hit this endpoint:
```sh
curl --location 'https://localhost:8080/'
```

Expect this response:
```
Hello world
```

## WebSocket

### Connect to Server

Run this command:
```sh
wscat -c wss://localhost:8080/ws --ca "$(mkcert -CAROOT)/rootCA.pem"
```

### Create Room

```json
{"command": "create", "params": {"user_id": "01JYGRSRD8Y20N08HMD2K9A1G1"}}
```

### Join Room

```json
{"command": "join", "params": {"room_id": "0197a1ac-9f1e-77b3-9173-1c8d57b91106", "user_id": "01JYGRSRD8Y20N08HMD2K9A1G1"}}
```

NOTE:
1. The first player to join will always be assigned with the character `x`.
2. When the room is filled 2 players, the game automatically starts.

### Register Your Move

```json
{"command": "move", "params": {"room_id": "0197a1ac-9f1e-77b3-9173-1c8d57b91106", "user_id": "01JYGRSRD8Y20N08HMD2K9A1G1", "row": "0", "column": "2"}}
```

NOTE:
1. The winner is evaluated each move. If there's a winner, then the game automatically finishes.
2. After the game has been finished, registering a move will yield an error.
