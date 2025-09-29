//! Help text for P2P core

use shared::config::*;

/// Print help information
pub fn print_help() {
    println!("\n📖 P2P Chat Client Help");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Usage: p2p-core [OPTIONS]");
    println!("\nOptions:");
    println!("  -u, --username <NAME>     Set username (required)");
    println!("  -p, --port <PORT>         Set listening port (default: auto-select from {}-{})", FIXED_PORT, FALLBACK_PORT_END);
    println!("      --host <HOST>         Set listening host (default: {})", DEFAULT_HOST_LOCALHOST);
    println!("  -b, --bootstrap <IP:PORT> Add bootstrap peer (can be used multiple times)");
    println!("  -h, --help                Show this help");
    println!("\nConfiguration:");
    println!("  🔌 Fixed Port: {} (with fallback range {}-{})", FIXED_PORT, FALLBACK_PORT_START, FALLBACK_PORT_END);
    println!("  🔒 TLS: Always enabled for security");
    println!("  🌐 Default Host: {} (localhost)", DEFAULT_HOST_LOCALHOST);
    println!("\nExamples:");
    println!("  p2p-core -u Alice                              # Create new chat room");
    println!("  p2p-core -u Bob --host 0.0.0.0               # Allow external connections");
    println!("  p2p-core -u Charlie -b 192.168.1.100:40000   # Connect to existing peer");
    println!("  p2p-core -u David -p 40005                   # Use specific port");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
