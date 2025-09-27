# P2P Chat Application

A simple peer-to-peer chat application built with Rust featuring TLS encryption and decentralized networking.

## 🚀 Features

- **P2P Architecture** - No central server required
- **TLS Encryption** - Always-on encryption with self-signed certificates
- **Auto Discovery** - Automatic peer discovery via multicast
- **Clean Terminal UI** - Simple and intuitive chat interface

## 🚀 Quick Start

```bash
# Start bootstrap node (first peer)
cargo run --bin p2p-chat -- -u Alice -p 8080

# Connect to bootstrap (in another terminal)
cargo run --bin p2p-chat -- -u Bob -b 127.0.0.1:8080
```

## 📖 Usage

### Command Line Options
- `-u, --username <NAME>` - Set your username (required)
- `-p, --port <PORT>` - Set listening port (optional, random if not set)
- `-b, --bootstrap <IP:PORT>` - Connect to bootstrap peer
- `-h, --help` - Show help information

### Chat Commands
- Type any message and press Enter to send
- `/peers` - Show connected peers
- `/help` - Show available commands
- `/quit` - Exit the application

### Example
```bash
# Terminal 1 - Start Alice as bootstrap
cargo run --bin p2p-chat -- -u Alice -p 8080

# Terminal 2 - Bob connects to Alice
cargo run --bin p2p-chat -- -u Bob -b 127.0.0.1:8080

# Terminal 3 - Charlie connects and discovers both
cargo run --bin p2p-chat -- -u Charlie -b 127.0.0.1:8080
```

## 🔧 Building

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release
```

## 📁 Project Structure

```
simple-chat-app/
├── p2p-chat/          # Main P2P chat application
├── shared/            # Core P2P networking and TLS
└── README.md          # This file
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE.md file for details.
