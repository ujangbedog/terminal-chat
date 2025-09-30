# DPQ Chat

A secure peer-to-peer chat application built with Rust that implements post-quantum cryptography to protect against both classical and quantum computer attacks. The system uses CRYSTALS-Dilithium for digital signatures and CRYSTALS-Kyber for key exchange, providing quantum-resistant security for all communications.

Features a terminal-based interface with real-time messaging, decentralized architecture (no central servers), and hybrid cryptographic protocols that combine classical and post-quantum algorithms for maximum security.

## 🛠️ Technologies & Architecture

### Core Technologies
- **Rust** - Memory-safe systems programming language
- **Tokio** - High-performance async runtime for network applications
- **CRYSTALS-Dilithium** - NIST-standardized post-quantum digital signature algorithm
- **CRYSTALS-Kyber** - NIST-standardized post-quantum key encapsulation mechanism
- **AES-256-GCM** - Authenticated encryption for message confidentiality and integrity

### Why Post-Quantum Cryptography?

#### The Quantum Threat
Current cryptographic systems (RSA, ECDH, ECDSA) rely on mathematical problems that quantum computers can solve efficiently using Shor's algorithm. Post-quantum cryptography provides:
- **Quantum Resistance**: Security against both classical and quantum computer attacks
- **Future-Proof Security**: Protection against "harvest now, decrypt later" attacks
- **NIST Standardization**: Using algorithms that have undergone rigorous security analysis
- **Hybrid Approach**: Combining classical and post-quantum algorithms for defense in depth


#### CRYSTALS-Kyber (Key Exchange)
- **NIST Standard**: Primary KEM standard selected by NIST in 2022
- **Performance**: Excellent performance for real-time applications
- **Security**: Equivalent to AES-256 against quantum attacks
- **Efficiency**: Reasonable key and ciphertext sizes for network transmission

#### CRYSTALS-Dilithium (Digital Signatures)
- **NIST Standard**: Primary signature standard selected by NIST in 2022
- **Quantum-Resistant**: Based on lattice problems, resistant to quantum attacks
- **Fast Verification**: Efficient signature verification for real-time messaging
- **Deterministic**: Consistent signatures for the same message and key

## 🔄 Complete Cryptographic Flow

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
   - Stores encrypted private key securely in `~/.dpq-chat/identities/`

3. **Identity Storage**:
   - Saves identity metadata (username, creation date, expiration, fingerprint)
   - Stores encrypted private key separately from metadata
   - Creates backup-friendly format for identity portability

### 2. Authentication & Key Loading Process
When starting a chat session:

1. **Identity Discovery**:
   - Scans `~/.dpq-chat/identities/` for available identities
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

2. **Hybrid Post-Quantum Handshake**:
   - **Step 1**: Kyber Key Exchange
     - Peer A initiates Kyber key encapsulation
     - Generates Kyber public key and sends to Peer B
     - Peer B responds with Kyber ciphertext
     - Both peers derive shared secret from Kyber exchange
   
   - **Step 2**: Dilithium Authentication
     - Each peer signs handshake data with their Dilithium private key
     - Handshake data includes: peer info + Kyber exchange + timestamp
     - Signatures are verified using peer's Dilithium public key
     - Ensures authentic identity and prevents man-in-the-middle attacks
   
   - **Step 3**: Session Key Derivation
     - Shared secret from Kyber → SHA-256 → 32-byte AES session key
     - Each peer-to-peer connection has unique session key
     - Session keys expire after 1 hour for forward secrecy

### 4. Message Flow & Security
During active chat sessions:

1. **Message Encryption Process**:
   ```
   Plain Message → JSON Serialize → AES-256-GCM(session_key) → Encrypted Message
   ```
   - Each message encrypted with peer-specific session key (derived from handshake)
   - Unique nonce generated for each message (prevents replay attacks)
   - Message sequence numbers ensure ordering and detect duplicates
   - Authenticated encryption provides both confidentiality and integrity

2. **Message Authentication**:
   - Messages are authenticated through AES-GCM (not individually signed)
   - Session key authenticity guaranteed by Dilithium-signed handshake
   - Any tampering with encrypted messages will fail decryption
   - Peer identity verified once during handshake, not per message

3. **Perfect Forward Secrecy**:
   - Session keys derived fresh from each Kyber handshake
   - Keys automatically expire after 1 hour
   - Old session keys securely erased from memory
   - Compromise of current keys doesn't affect past communications

## 🔧 Technical Implementation

### Cryptographic Flow Summary
```
1. Identity Generation:
   Dilithium Keypair → AES-256-GCM(password) → Encrypted Storage

2. Handshake Process:
   Kyber Key Exchange + Dilithium Signatures → Shared Secret → Session Key

3. Message Encryption:
   Plain Message → AES-256-GCM(session_key) → Encrypted Message
```

### Key Components

#### HandshakeManager (`shared/src/crypto/handshake.rs`)
- Manages peer-to-peer handshake process
- Integrates Kyber key exchange with Dilithium authentication
- Creates session keys from shared secrets
- Verifies peer signatures for authentication

#### DilithiumKeypair (`shared/src/crypto/dilithium_ops.rs`)
- Handles Dilithium signing and verification operations
- Loads keypairs from encrypted identity files
- Signs handshake data for peer authentication

#### SessionKey (`shared/src/crypto/session.rs`)
- Derives AES-256 keys from Kyber shared secrets
- Manages key expiration (1 hour lifetime)
- Provides encrypt/decrypt methods for messages

#### MessageCrypto (`shared/src/crypto/message_crypto.rs`)
- Encrypts/decrypts messages using session keys
- Handles message serialization and sequence numbers
- Prevents replay attacks through nonce management

### Usage Example
```rust
// Load user identity and decrypt private key
let identity = load_identity("username")?;
let password = "user_password";

// Create HandshakeManager with Dilithium support
let mut handshake_manager = create_handshake_manager_from_identity(&identity, password)?;

// Initiate handshake with peer
let handshake_data = handshake_manager.initiate_handshake("peer_fingerprint")?;

// Process peer's response and get session key
let (session_key, response) = handshake_manager.process_handshake(peer_handshake_data)?;

// Encrypt messages with session key
let plain_message = MessageCrypto::create_text_message("username", "Hello!");
let encrypted = MessageCrypto::encrypt_message(&session_key, &plain_message, sequence_num)?;
```

## 🚀 Getting Started & Usage Guide

### First Time Setup

#### Step 1: Generate Your Cryptographic Identity
Before you can use DPQ Chat, you need to create a cryptographic identity. This identity consists of a CRYSTALS-Dilithium key pair for authentication and handshake signing.

```bash
cargo run -- generate-key
```

**What happens during identity generation:**
1. System prompts for username (becomes your chat display name)
2. You create a secure password (encrypts your private key)
3. System generates CRYSTALS-Dilithium key pair (public + private keys)
4. Private key encrypted with password using AES-256-GCM + Argon2id
5. Identity saved to `~/.dpq-chat/identities/[username].json`
6. Unique fingerprint generated from public key (your identity hash)

**Security Features:**
- Password never stored - only used to derive encryption key via Argon2id
- Private key never leaves device in unencrypted form
- Fingerprint serves as your unique peer identifier
- Identity files are portable and can be backed up securely

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
1. System scans for available identities in `~/.dpq-chat/identities/`
2. If multiple identities exist, you select one from the list
3. Enter password for your selected identity
4. System decrypts Dilithium private key using Argon2id + AES-256-GCM
5. HandshakeManager initialized with your Dilithium keypair
6. You're authenticated and ready for secure P2P connections

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
# 5. System establishes secure connection:
#    - Kyber key exchange generates shared secret
#    - Both peers sign handshake data with Dilithium private keys
#    - Signatures verified using each other's public keys
#    - Session key derived from Kyber shared secret
#    - Encrypted communication channel established
# 6. Both users see connection confirmation with peer fingerprints
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
- All messages encrypted with AES-256-GCM using session keys
- Session keys derived from Kyber post-quantum key exchange
- Peer authentication via Dilithium signatures during handshake
- Perfect forward secrecy through session key expiration (1 hour)
- Automatic detection of tampered messages via authenticated encryption
- Sequence numbers prevent replay attacks

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
cd dpq-chat

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
./target/release/dpq-chat
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

