# Simple Chat Application

A simple chat application built with Rust using async/await and Tokio. The server acts as a message relay and multiple clients can connect to chat with each other.

## Features

- ✅ Multiple clients can connect simultaneously
- ✅ Real-time message broadcasting
- ✅ Username validation and uniqueness
- ✅ User join/leave notifications
- ✅ Simple line-based interface
- ✅ Server logging for message relay

## Project Structure

```
simple-chat-app/
├── Cargo.toml          # workspace configuration
├── server/             # chat server
│   ├── src/
│   │   ├── main.rs
│   │   ├── client/     # client info management
│   │   ├── state/      # shared state management
│   │   └── handler/    # message handling
├── client/             # chat client
│   ├── src/
│   │   ├── main.rs
│   │   ├── connection/ # server connection
│   │   ├── ui/         # user interface
│   │   └── chat/       # chat functionality
└── shared/             # shared library
    ├── src/
    │   ├── message/    # message types
    │   ├── config/     # configuration
    │   └── utils/      # utility functions
```

## Quick Start

### 1. Build the project
```bash
cargo build
```

### 2. Start the server
```bash
cargo run --bin server
```

### 3. Start clients (in separate terminals)
```bash
cargo run --bin client
```

### 4. Enter username and start chatting!

## Configuration

Default settings:
- **Server**: 127.0.0.1:8080
- **Max message length**: 1024 characters
- **Max username length**: 32 characters

## Commands

- Type messages and press Enter to send
- `/quit` or `/exit` - disconnect gracefully
- `Ctrl+C` - force exit

## Server Logs

The server displays activity logs:
```
[CONNECTION] New client connected from: 127.0.0.1:xxxxx
[JOIN] User 'alice' joined the chat
[RELAY] alice: Hello everyone!
[LEAVE] User 'alice' left the chat
```

## License

MIT License - see [LICENSE.md](LICENSE.md) for details.
