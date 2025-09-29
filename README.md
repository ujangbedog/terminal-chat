# Terminal Chat Application

A simple peer-to-peer chat application built with Rust featuring TLS encryption and decentralized networking.

## 🚀 Features

- **P2P Architecture** - No central server required
- **TLS Encryption** - Always-on encryption with self-signed certificates
- **Auto Discovery** - Automatic peer discovery via multicast
- **Clean Terminal UI** - Simple and intuitive chat interface

## ⚙️ Configuration

Configuration is now **hardcoded for security and simplicity**:

- **Fixed Port**: `40000` (with fallback range `40001-40010`)
- **TLS**: Always enabled for security
- **Host Options**: 
  - `127.0.0.1` (localhost only)
  - `192.168.x.x` (local network - auto-detected)
  - `0.0.0.0` (all interfaces)
- **Log Level**: `error` (minimal logging for clean UI)

## 🚀 Quick Start

### Interactive Menu (Recommended)
```bash
# Start the application with interactive menu
cargo run --bin terminal-chat

# The menu will guide you through:
# 1. Enter username
# 2. Select host type (localhost/local network/wildcard)
# 3. Choose to create new chat or connect to existing peer
```

### Direct CLI Mode
```bash
# Create new chat room (uses fixed port 40000 or fallback)
cargo run --bin terminal-chat -- p2p -u Alice --host 127.0.0.1

# Connect to existing peer
cargo run --bin terminal-chat -- p2p -u Bob --host 127.0.0.1 -b 127.0.0.1:40000

# Allow external connections
cargo run --bin terminal-chat -- p2p -u Alice --host 0.0.0.0
```

### Advanced Usage (Direct P2P Core)
```bash
# Create new chat room (auto-selects port from 40000-40010)
cargo run --bin p2p-core -- -u Alice

# Connect to existing peer
cargo run --bin p2p-core -- -u Bob -b 192.168.1.100:40000

# Use specific port
cargo run --bin p2p-core -- -u Alice -p 40005

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
- `-p, --port <PORT>` - Set listening port (default: auto-select from 40000-40010)
- `--host <HOST>` - Set listening host (default: 127.0.0.1)
- `-b, --bootstrap <IP:PORT>` - Connect to bootstrap peer
- `--no-tls` - ⚠️ Ignored (TLS always enabled for security)
- `-h, --help` - Show help information

### Advanced Direct P2P Core
For advanced users who want direct access to P2P core:
```bash
cargo run --bin p2p-core -- [OPTIONS]
```
Same options as CLI mode but bypasses the launcher system.

### Configuration Details
All configuration is now **hardcoded** for security and simplicity:

- **Fixed Port**: `40000` (primary), fallback range `40001-40010`
- **TLS Encryption**: Always enabled (cannot be disabled)
- **Log Level**: `error` (minimal logging for clean UI)
- **Multicast Address**: `224.0.0.1:9999` (peer discovery)
- **Connection Timeout**: `30 seconds`
- **Heartbeat Interval**: `60 seconds`
- **Max Connections**: `50 peers`

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
