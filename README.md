# Terminal Chat Application

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

### Interactive Menu (Recommended)
```bash
# Start the application with interactive menu
cargo run --bin terminal-chat

# Or use CLI arguments for direct P2P mode
cargo run --bin terminal-chat -- p2p -u Alice -p 8080
cargo run --bin terminal-chat -- p2p -u Bob -b 127.0.0.1:8080
```

### Advanced Usage (Direct P2P Core)
```bash
# Start bootstrap node (first peer) - MUST specify port
cargo run --bin p2p-core -- -u Alice -p 8080

# For external connections, use 0.0.0.0
cargo run --bin p2p-core -- -u Alice --host 0.0.0.0 -p 8080

# Connect to bootstrap (in another terminal) - uses random port automatically
cargo run --bin p2p-core -- -u Bob -b 127.0.0.1:8080
cargo run --bin p2p-core -- -u Bob -b 192.168.1.106:8080  # for external IP

# Multiple clients can connect to the same bootstrap
cargo run --bin p2p-core -- -u Charlie -b 192.168.1.106:8080
```

## Usage

### Interactive Menu Mode (Recommended)
```bash
cargo run --bin terminal-chat
```
This will show an interactive menu where you can:
- Create P2P Chat sessions
- Configure settings
- Access help and documentation

### CLI Mode
```bash
# Direct P2P chat via CLI
cargo run --bin terminal-chat -- p2p [OPTIONS]
```

#### CLI Options for P2P Mode:
- `-u, --username <NAME>` - Set your username (required)
- `-p, --port <PORT>` - Set listening port (overrides .env DEFAULT_PORT)
- `--host <HOST>` - Set listening host (overrides .env DEFAULT_HOST)
- `-b, --bootstrap <IP:PORT>` - Connect to bootstrap peer
- `--no-tls` - Disable TLS encryption
- `-h, --help` - Show help information

### Advanced Direct P2P Core
For advanced users who want direct access to P2P core:
```bash
cargo run --bin p2p-core -- [OPTIONS]
```
Same options as CLI mode but bypasses the launcher system.

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

### Examples

#### Using Interactive Menu (Recommended)
```bash
# Terminal 1 - Start interactive menu
cargo run --bin terminal-chat
# Select "Create P2P Chat" and follow prompts

# Terminal 2 - Start another instance
cargo run --bin terminal-chat
# Select "Create P2P Chat" and connect to first peer
```

#### Using CLI Mode
```bash
# Terminal 1 - Start Alice as bootstrap
cargo run --bin terminal-chat -- p2p -u Alice -p 8080

# Terminal 2 - Bob connects to Alice
cargo run --bin terminal-chat -- p2p -u Bob -b 127.0.0.1:8080

# Terminal 3 - Charlie connects and discovers both
cargo run --bin terminal-chat -- p2p -u Charlie -b 127.0.0.1:8080
```

#### Advanced Direct P2P Core
```bash
# Terminal 1 - Start Alice as bootstrap (advanced)
cargo run --bin p2p-core -- -u Alice -p 8080

# Terminal 2 - Bob connects to Alice (advanced)
cargo run --bin p2p-core -- -u Bob -b 127.0.0.1:8080

# Terminal 3 - Charlie connects and discovers both (advanced)
cargo run --bin p2p-core -- -u Charlie -b 127.0.0.1:8080
```

## Building

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release
```

## Project Structure

```
terminal-chat/
├── launcher/          # Entry point with interactive menu
├── cli/              # CLI interface and menu system
├── p2p-core/         # Core P2P chat functionality
├── shared/           # Core P2P networking and TLS
└── README.md         # This file
