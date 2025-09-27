# P2P Chat Application

A simple peer-to-peer chat application built with Rust featuring TLS encryption and decentralized networking.

## 🚀 Features

- **P2P Architecture** - No central server required
- **TLS Encryption** - Always-on encryption with self-signed certificates
- **Auto Discovery** - Automatic peer discovery via multicast
- **Clean Terminal UI** - Simple and intuitive chat interface

## ⚙️ Configuration

First, copy the environment configuration file:
```bash
cp .env.example .env
```

Edit `.env` to configure default settings:
```bash
# Default host (use 0.0.0.0 for external connections)
DEFAULT_HOST=127.0.0.1
DEFAULT_PORT=8080
TLS_ENABLED=true
LOG_LEVEL=error
```

## 🚀 Quick Start

```bash
# Start bootstrap node (first peer) - MUST specify port
cargo run --bin p2p-chat -- -u Alice -p 8080

# For external connections, use 0.0.0.0
cargo run --bin p2p-chat -- -u Alice --host 0.0.0.0 -p 8080

# Connect to bootstrap (in another terminal) - uses random port automatically
cargo run --bin p2p-chat -- -u Bob -b 127.0.0.1:8080
cargo run --bin p2p-chat -- -u Bob -b 192.168.1.106:8080  # for external IP

# Multiple clients can connect to the same bootstrap
cargo run --bin p2p-chat -- -u Charlie -b 192.168.1.106:8080
```

## 📖 Usage

### Command Line Options
- `-u, --username <NAME>` - Set your username (required)
- `-p, --port <PORT>` - Set listening port (overrides .env DEFAULT_PORT)
- `--host <HOST>` - Set listening host (overrides .env DEFAULT_HOST)
- `-b, --bootstrap <IP:PORT>` - Connect to bootstrap peer
- `-h, --help` - Show help information

### Environment Variables (.env file)
- `DEFAULT_HOST` - Default host to bind to (127.0.0.1 or 0.0.0.0)
- `DEFAULT_PORT` - Default port to listen on
- `TLS_ENABLED` - Enable/disable TLS encryption (true/false)
- `LOG_LEVEL` - Logging level (error, warn, info, debug, trace)
- `MULTICAST_ADDR` - Multicast address for peer discovery
- `CONNECTION_TIMEOUT` - Connection timeout in seconds
- `HEARTBEAT_INTERVAL` - Heartbeat interval in seconds
- `MAX_CONNECTIONS` - Maximum number of peer connections

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

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
