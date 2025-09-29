# Terminal Chat

A modern, secure peer-to-peer chat application built with Rust that implements cutting-edge cryptographic technologies to ensure communication remains private and secure, even against future quantum computing threats. This application combines the robustness of Rust's memory safety with advanced cryptographic protocols to create a decentralized chat system that requires no central servers, providing users with complete control over their communications.

The application features a beautiful terminal-based user interface that provides real-time messaging capabilities while maintaining the highest standards of security through post-quantum cryptography, hybrid TLS encryption, and secure key management. Every aspect of the system has been designed with security-first principles, ensuring that user communications remain confidential and authenticated.

## 🛠️ Technologies & Architecture

### Core Technologies
- **Rust** - Memory-safe systems programming language that eliminates entire classes of vulnerabilities
- **Tokio** - High-performance async runtime optimized for network applications
- **TLS 1.3** - Latest transport layer security with hybrid classical/post-quantum certificates
- **CRYSTALS-Dilithium** - NIST-standardized post-quantum digital signature algorithm
- **CRYSTALS-Kyber** - NIST-standardized post-quantum key encapsulation mechanism

### Why These Technologies?

#### Why TLS?
Transport Layer Security (TLS) provides essential protection for data in transit. We use TLS 1.3, the latest version, which offers:
- **Perfect Forward Secrecy**: Each session uses unique keys, so compromising one session doesn't affect others
- **Reduced Handshake Latency**: Faster connection establishment compared to older TLS versions
- **Stronger Cipher Suites**: Removal of vulnerable legacy algorithms
- **Protection Against Downgrade Attacks**: Prevents attackers from forcing use of weaker protocols

#### Why Post-Quantum Cryptography (PQC)?
Current cryptographic systems rely on mathematical problems that quantum computers can solve efficiently using Shor's algorithm. PQC provides:
- **Quantum Resistance**: Security against both classical and quantum computer attacks
- **Future-Proof Security**: Protection against "harvest now, decrypt later" attacks
- **NIST Standardization**: Using algorithms that have undergone rigorous security analysis
- **Hybrid Approach**: Combining classical and post-quantum algorithms for defense in depth

#### Why CRYSTALS-Kyber for Key Exchange?
Kyber is our chosen key encapsulation mechanism (KEM) because:
- **NIST Standard**: Selected as the primary KEM standard by NIST in 2022
- **Performance**: Excellent performance characteristics for real-time applications
- **Security Level**: Provides security equivalent to AES-256 against quantum attacks
- **Small Key Sizes**: Reasonable key and ciphertext sizes for network transmission

#### Why CRYSTALS-Dilithium for Digital Signatures?
Dilithium serves as our digital signature algorithm because:
- **NIST Standard**: Selected as the primary signature standard by NIST in 2022
- **Strong Security**: Based on the hardness of lattice problems, resistant to quantum attacks
- **Efficient Verification**: Fast signature verification suitable for real-time messaging
- **Deterministic Signatures**: Provides consistent signatures for the same message and key

## 🔄 System Flow & Architecture

### 1. Identity Generation Process
When a user first runs the application, the system performs the following steps:

1. **Key Pair Generation**:
   - Generates a CRYSTALS-Dilithium key pair (public/private keys)
   - Creates a unique fingerprint from the public key for identity verification
   - Generates cryptographic random values for enhanced security

2. **Password-Based Encryption**:
   - Prompts user for a secure password
   - Uses Argon2id (password hashing function) to derive encryption key from password
   - Encrypts the private key using AES-256-GCM with the derived key
   - Stores encrypted private key securely in `~/.terminal-chat/identities/`

3. **Identity Storage**:
   - Saves identity metadata (username, creation date, expiration, fingerprint)
   - Stores encrypted private key separately from metadata
   - Creates backup-friendly format for identity portability

### 2. Authentication & Key Loading Process
When starting a chat session:

1. **Identity Discovery**:
   - Scans `~/.terminal-chat/identities/` for available identities
   - Presents list of identities to user for selection
   - Validates identity integrity and expiration status

2. **Password Verification**:
   - Prompts for password to decrypt selected identity
   - Derives decryption key using stored Argon2id parameters
   - Attempts to decrypt private key with derived key
   - Validates decrypted key format and integrity

3. **Session Preparation**:
   - Loads decrypted private key into secure memory
   - Prepares public key for peer verification
   - Initializes cryptographic contexts for the session

### 3. P2P Connection & Handshake Process
When establishing peer connections:

1. **Network Discovery**:
   - Uses multicast UDP for local peer discovery
   - Announces presence with encrypted identity information
   - Listens for peer announcements on the network

2. **TLS Handshake with Hybrid Cryptography**:
   - Establishes TLS 1.3 connection with classical ECDH key exchange
   - Performs additional Kyber key encapsulation for post-quantum security
   - Combines classical and post-quantum shared secrets
   - Derives session keys from hybrid shared secret

3. **Peer Authentication**:
   - Exchanges Dilithium public keys over secure TLS channel
   - Each peer signs a challenge with their Dilithium private key
   - Verifies peer signatures to ensure authentic identity
   - Establishes authenticated, encrypted communication channel

### 4. Message Flow & Security
During active chat sessions:

1. **Message Encryption**:
   - Each message is encrypted with session-specific AES-256-GCM keys
   - Uses unique nonces for each message to prevent replay attacks
   - Includes message sequence numbers for ordering and integrity

2. **Digital Signatures**:
   - Each message is signed with sender's Dilithium private key
   - Recipients verify signatures to ensure message authenticity
   - Prevents message tampering and impersonation attacks

3. **Perfect Forward Secrecy**:
   - Session keys are rotated periodically
   - Old keys are securely erased from memory
   - Compromise of current keys doesn't affect past communications

## 🚀 Getting Started & Usage Guide

### First Time Setup

#### Step 1: Generate Your Cryptographic Identity
Before you can use Terminal Chat, you need to create a cryptographic identity. This identity consists of a CRYSTALS-Dilithium key pair that will be used for authentication and message signing.

```bash
cargo run -- generate-key
```

**What happens during identity generation:**
1. The system will prompt you for a username (this becomes your chat display name)
2. You'll be asked to create a secure password (this encrypts your private key)
3. The system generates a CRYSTALS-Dilithium key pair
4. Your private key is encrypted with your password using AES-256-GCM
5. Your identity is saved to `~/.terminal-chat/identities/[username].json`

**Important Security Notes:**
- Your password is never stored - it's only used to derive the encryption key
- If you forget your password, your identity cannot be recovered
- Your private key never leaves your device in unencrypted form
- The fingerprint shown is derived from your public key and serves as your unique identifier

#### Step 2: Verify Your Identity
After generation, you can list your identities to verify creation:

```bash
cargo run -- list
```

This will show:
- Your username and status (ACTIVE/EXPIRED)
- Your unique fingerprint (first 8 characters shown for brevity)
- Creation date and expiration date (if set)
- The cryptographic algorithm used (CRYSTALS-Dilithium)

### Starting a Chat Session

#### Method 1: Interactive Menu (Recommended for Beginners)

Start the application without any arguments to access the interactive menu:

```bash
cargo run
```

**Authentication Process:**
1. The system scans for available identities
2. If multiple identities exist, you'll be prompted to select one
3. Enter the password for your selected identity
4. The system decrypts and loads your private key
5. You're authenticated and ready to chat

**Menu Navigation:**
- Use arrow keys (↑/↓) to navigate menu options
- Press Enter to select an option
- Press Esc or Ctrl+C to exit

**Menu Options:**
1. **🔗 Create P2P Chat**: Start a new chat room that others can join
2. **🏠 Join Chat Room**: Connect to an existing chat room
3. **⚙️ Settings**: View configuration and manage identities
4. **🚪 Exit**: Close the application

#### Method 2: Direct CLI Mode (Advanced Users)

For advanced users who prefer command-line interfaces:

```bash
# Create a new chat room
cargo run -- p2p -u [username] --host [host_address]

# Join an existing chat room
cargo run -- p2p -u [username] --host [host_address] -b [peer_address:port]
```

**CLI Parameters:**
- `-u, --username`: Your identity username (must match an existing identity)
- `--host`: Network interface to bind to (127.0.0.1, 192.168.x.x, or 0.0.0.0)
- `-p, --port`: Specific port to use (optional, auto-selects from 40000-40010)
- `-b, --bootstrap`: Address of peer to connect to (IP:PORT format)

### Detailed Usage Scenarios

#### Scenario 1: Two Friends on Same Network

**Alice starts a new chat room:**
```bash
# Alice runs the application
cargo run

# Authentication process:
# 1. System finds Alice's identity
# 2. Alice enters her password
# 3. Private key is decrypted and loaded

# Menu selection:
# 1. Alice selects "🔗 Create P2P Chat"
# 2. System prompts for network interface:
#    - "🏠 Localhost (127.0.0.1)" - only local machine
#    - "🌐 Local Network (192.168.x.x)" - same WiFi/LAN
#    - "🌍 All Interfaces (0.0.0.0)" - internet accessible
# 3. Alice selects "🌐 Local Network"
# 4. System automatically finds available port (e.g., 40000)
# 5. Chat room starts, showing:
#    - Alice's username and fingerprint
#    - Listening address (e.g., 192.168.1.100:40000)
#    - "⏳ Waiting for peers..." status
```

**Bob joins Alice's chat:**
```bash
# Bob runs the application
cargo run

# Authentication process (same as Alice)
# Bob enters his password to decrypt his identity

# Menu selection:
# 1. Bob selects "🔗 Create P2P Chat"
# 2. Bob selects "🔗 Connect to existing peer"
# 3. System prompts for peer address
# 4. Bob enters: 192.168.1.100:40000
# 5. System establishes connection:
#    - TLS 1.3 handshake with hybrid Kyber key exchange
#    - Dilithium signature verification
#    - Secure channel established
# 6. Both users see connection confirmation
```

#### Scenario 2: Multiple Users Joining

**Charlie joins the existing chat:**
```bash
cargo run

# Charlie follows the same process as Bob
# Enters Alice's address: 192.168.1.100:40000
# System automatically discovers Bob through Alice
# All three users are now connected in a mesh network
```

**Network Topology:**
- Each user maintains direct connections to all other users
- Messages are sent directly between peers (no routing through Alice)
- If Alice leaves, Bob and Charlie remain connected
- New users can join through any existing peer

#### Scenario 3: Internet Chat (Advanced)

**Alice creates internet-accessible chat:**
```bash
cargo run

# Alice selects "🌍 All Interfaces (0.0.0.0)"
# This binds to all network interfaces
# Alice shares her public IP and port with Bob
# Example: 203.0.113.10:40000
```

**Bob connects from different network:**
```bash
cargo run

# Bob enters Alice's public address: 203.0.113.10:40000
# Connection works across internet (requires port forwarding)
```

### In-Chat Commands and Features

Once connected to a chat, you have access to various commands:

#### Basic Communication
```
# Send a message
Just type your message and press Enter

# Example:
Hello everyone! How are you doing today?
```

#### Chat Commands
```
# Show help
/help

# Display detailed peer information
/stats
# Shows: Peer ID, Username, IP Address, Port, Connection status

# Clear chat history
/clear
# Removes all messages from your local display

# Exit the chat
/quit
# or
/exit
# Cleanly disconnects from all peers and returns to menu

# Force exit
Ctrl+C
# Emergency exit with terminal cleanup
```

#### Advanced Features

**Real-time Status Updates:**
- Connection status shown in header
- Peer join/leave notifications
- Message delivery confirmations
- Network connectivity indicators

**Message Features:**
- Real-time message delivery
- Message history with timestamps
- Different colors for different users
- Automatic message ordering
- Delivery confirmations

**Security Features:**
- All messages are encrypted with AES-256-GCM
- Each message is digitally signed with Dilithium
- Perfect forward secrecy through key rotation
- Automatic detection of tampered messages

### Identity Management

#### Listing Identities
```bash
cargo run -- list
```
Shows all identities with detailed information including creation dates, expiration status, and fingerprints.

#### Generating Additional Identities
```bash
cargo run -- generate-key
```
You can create multiple identities for different purposes (work, personal, etc.).

#### Configuration Management
```bash
cargo run -- config --show
```
Displays current system configuration including:
- Network settings (ports, timeouts)
- Cryptographic parameters
- File locations
- Security settings

### Troubleshooting Common Issues

#### Connection Problems
1. **Cannot connect to peer:**
   - Verify the peer address is correct
   - Check firewall settings
   - Ensure both users are on compatible networks

2. **Authentication failures:**
   - Verify password is correct
   - Check identity file integrity
   - Ensure identity hasn't expired

3. **Network discovery issues:**
   - Check multicast is enabled on network
   - Verify firewall allows UDP traffic
   - Try direct IP connection instead

#### Performance Optimization
- Use localhost (127.0.0.1) for same-machine testing
- Use local network for best performance on LAN
- Consider network latency for internet connections
- Monitor system resources during large group chats

### Security Best Practices

1. **Password Security:**
   - Use strong, unique passwords for each identity
   - Never share your password
   - Consider using a password manager

2. **Identity Protection:**
   - Backup your identity files securely
   - Don't share private key files
   - Regularly check for expired identities

3. **Network Security:**
   - Use local networks when possible
   - Be cautious with internet-accessible chats
   - Verify peer fingerprints for important communications

4. **Operational Security:**
   - Regularly update the application
   - Monitor for suspicious connection attempts
   - Use /stats to verify peer identities

## 🔧 Building & Development

### Prerequisites
- Rust 1.70 or later
- Cargo (comes with Rust)
- Git (for cloning the repository)

### Building from Source

#### Development Build (Fast Compilation)
```bash
# Clone the repository
git clone <repository-url>
cd terminal-chat

# Build in development mode
cargo build

# Run directly from source
cargo run
```

#### Release Build (Optimized Performance)
```bash
# Build with full optimizations
cargo build --release

# The optimized binary will be in target/release/
./target/release/terminal-chat
```

#### Additional Build Commands
```bash
# Check code without building (fast)
cargo check

# Run tests
cargo test

# Clean build artifacts
cargo clean

# Build documentation
cargo doc --open
```

### Performance Optimizations
The release build includes several optimizations:
- **LTO (Link Time Optimization)**: Enables cross-crate optimizations
- **Codegen Units**: Single unit for better optimization
- **Panic Strategy**: Abort on panic for smaller binaries
- **Symbol Stripping**: Removes debug symbols for smaller size
- **Optimized Dependencies**: Minimal feature sets for faster compilation

