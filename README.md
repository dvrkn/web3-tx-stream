# Base Transaction Sniffer TUI

A terminal user interface for monitoring Base blockchain transactions in real-time.

## Features

- 📡 Real-time WebSocket connection to Base RPC
- 📊 Live transaction monitoring with scrollable list
- 🎨 Color-coded function signatures
- 🔍 Detailed transaction view popup
- 🔄 Automatic reconnection on disconnect
- 💾 Memory-efficient circular buffer (max 1000 transactions)
- ⌨️ Full keyboard navigation
- 🔀 Toggle between newest-first and oldest-first sorting
- 🗑️ Clear transaction list on demand

## Installation

```bash
# Build the project
cargo build --release

# The binary will be at:
./target/release/base-sniffer-tui
```

## Usage

### Running with Base Mainnet

```bash
# Using default RPC (wss://base-rpc.publicnode.com)
./target/release/base-sniffer-tui

# Or with custom RPC URL
BASE_RPC_URL=wss://your-base-rpc-url.com cargo run --release
```

### Testing with Debug Mode

To test the UI with sample data (debug build only):

```bash
# Build debug version
cargo build

# Run with sample data
DEBUG_MODE=1 cargo run
```

## Controls

### Navigation
| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Scroll up/down one transaction |
| `PgUp`/`PgDn` | Page up/down |
| `Home` or `g` | Jump to first transaction |
| `End` or `G` | Jump to last transaction |
| `Enter` | Show transaction details popup |

### Actions
| Key | Action |
|-----|--------|
| `t` | Toggle sort (new transactions on top/bottom) |
| `c` | Clear all transactions |
| `r` | Reconnect to RPC |
| `q` or `Esc` | Quit (or close details popup) |
| `Ctrl+C` | Force quit |

## Configuration

Environment variables:

- `BASE_RPC_URL` - WebSocket RPC endpoint (default: `wss://base-rpc.publicnode.com`)
- `MAX_TRANSACTIONS` - Maximum transactions to keep in memory (default: 1000)
- `RECONNECT_ATTEMPTS` - Number of reconnection attempts (default: 10)
- `RECONNECT_DELAY_MS` - Delay between reconnection attempts in ms (default: 5000)

## UI Layout

```
┌─────────────────────────────────────────────────────────┐
│ Base Transaction Sniffer | Connected: ✓ | TX: 1,234    │
├─────────────────────────────────────────────────────────┤
│ Time     Hash         From        To          Value     │
│ 12:34:56 0x1a2b...   0x3c4d...   0x5e6f...   0.001 ETH │
│ 12:34:57 0x7g8h...   0x9i0j...   0xabcd...   0.000 ETH │
│ ...                                                     │
├─────────────────────────────────────────────────────────┤
│ ↑/↓: Scroll | q: Quit | RPC: base-rpc.publicnode.com  │
└─────────────────────────────────────────────────────────┘
```

## Troubleshooting

### No transactions appearing
- Check your internet connection
- Verify the RPC URL is correct and accessible
- Base mainnet might have low activity periods
- Try a different RPC endpoint

### UI Issues
- Make sure your terminal is large enough (minimum 80x24)
- Try resizing your terminal window
- Some terminals may not support all Unicode characters

## Development

### Project Structure

```
src/
├── main.rs           # Entry point and event loop
├── app/              # Application state and event handling
│   ├── state.rs      # State management
│   └── handler.rs    # Event handlers
├── rpc/              # WebSocket RPC client
│   └── client.rs     # Base RPC implementation
├── model/            # Data models
│   ├── transaction.rs # Transaction struct
│   └── decoder.rs    # Function signature decoder
└── ui/               # Terminal UI components
    ├── header.rs     # Header widget
    ├── list.rs       # Transaction list widget
    └── footer.rs     # Footer widget
```

### Building from Source

Requirements:
- Rust 1.70+ (stable)
- Cargo

```bash
git clone <repository>
cd base-sniffer-tui
cargo build --release
```

## License

MIT