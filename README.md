# Sha3x Miner

> High-Performance Multi-Algorithm CPU Miner with Web Dashboard

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

A high-performance CPU miner supporting SHA3x (Tari) and experimental SHA256d (Bitcoin) mining with advanced features including DNS resolution, real-time web dashboard, and experimental Stratum V2 support.

## 🚀 Features

### Production Ready
- **SHA3x Mining**: Full support for Tari blockchain mining
- **DNS Resolution**: Connect using pool domains (e.g., `pool.sha3x.supportxtm.com:6118`)
- **Web Dashboard**: Real-time mining statistics at `http://localhost:8080`
- **Pool Compatibility**: Works with Lucky Pool, community pools, and strict protocol pools
- **High Performance**: Achieves 15-18 MH/s on dual Xeon systems (72 threads)
- **Thread Scaling**: Optimized for high-core-count systems
- **Benchmark Mode**: Performance testing without pool connection

### Experimental (WIP)
- **SHA256d Support**: Bitcoin mining capabilities (under development)
- **Stratum V2 (SV2)**: Next-generation Bitcoin mining protocol testing
- **Multi-Algorithm**: Framework for supporting multiple mining algorithms

## 📊 Performance Highlights

| System Configuration | SHA3x Hashrate | Efficiency |
|----------------------|----------------|------------|
| Dual Xeon 2699v3 (72T) | 15-18 MH/s | ~250 KH/s per thread |
| High-end Desktop (32T) | 8-12 MH/s | ~300-400 KH/s per thread |
| Workstation (64T) | 15-25 MH/s | ~250-400 KH/s per thread |

## 🛠 Installation

### Prerequisites

- Rust 1.70 or later
- CPU with AVX2 support (recommended)
- 4GB+ RAM for optimal performance

### Building from Source

```bash
# Clone the repository
git clone https://github.com/OIEIEIO/sha3x-miner.git
cd sha3x-miner

# Build in release mode (optimized)
cargo build --release

# The binary will be available at ./target/release/sha3x-miner
```

## 🎯 Quick Start

### SHA3x Mining (Tari)

```bash
# Basic mining with web dashboard
cargo run --release -- \
  --algo sha3x \
  --wallet 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW \
  --pool pool.sha3x.supportxtm.com:6118 \
  --worker x99-cpu \
  --threads 18 \
  --web

# Alternative pools
cargo run --release -- \
  --algo sha3x \
  --wallet YOUR_TARI_WALLET \
  --pool 127.0.0.1:7777\
  --worker my-worker \
  --threads 32

# High-performance mining (dual Xeon example)
cargo run --release -- \
  --algo sha3x \
  --wallet YOUR_TARI_WALLET \
  --pool pool.sha3x.supportxtm.com:6118 \
  --worker xeon-dual \
  --threads 72 \
  --web
```

### Experimental Bitcoin Mining (SHA256d - WIP)

```bash
# Bitcoin pool testing (experimental)
cargo run --release -- \
  --algo sha256 \
  --wallet YOUR_BITCOIN_ADDRESS \
  --pool stratum+tcp://pool.bitcoin.com:3333 \
  --worker bitcoin-test \
  --threads 16

# SV2 (Stratum V2) connection testing
cargo run --release -- \
  --test-sv2 \
  --pool 127.0.0.1:34254
```

### Benchmarking

```bash
# SHA3x performance test
cargo run --release -- \
  --algo sha3x \
  --benchmark \
  --threads 32 \
  --benchmark-duration 60

# SHA256d performance test (experimental)
cargo run --release -- \
  --algo sha256 \
  --benchmark \
  --threads 32 \
  --benchmark-duration 60
```

## 🌐 Web Dashboard

When using the `--web` flag, access your real-time mining dashboard at:
- **URL**: `http://localhost:8080`
- **WebSocket**: `ws://localhost:8080/ws`
- **Features**: Live hashrate charts, share tracking, thread monitoring

## 📋 Command Line Options

### Required for Mining
- `--algo <ALGORITHM>` - Mining algorithm (`sha3x` or `sha256`)
- `--wallet <ADDRESS>` - Wallet address (Tari or Bitcoin)
- `--pool <HOST:PORT>` - Pool address (supports DNS resolution)

### Optional Parameters
- `--worker <NAME>` - Worker identifier for pool
- `--threads <NUM>` - Number of CPU threads (0 = auto-detect)
- `--web` - Enable real-time web dashboard
- `--benchmark` - Run performance benchmark
- `--benchmark-duration <SEC>` - Benchmark duration in seconds
- `--benchmark-difficulty <N>` - Target difficulty for benchmarking

### Experimental Options
- `--test-sv2` - Test Stratum V2 connection (Bitcoin)

### Help and Information
- `-h, --help` - Show detailed help
- `-V, --version` - Show version information

## 🏗 Project Structure

```
sha3x-miner/
├── src/
│   ├── benchmark/           # 🧪 Performance testing framework
│   ├── core/                # ⚡ Core mining algorithms
│   │   ├── sha3x.rs         # SHA3x (Tari) implementation
│   │   ├── sha256.rs        # SHA256d (Bitcoin) implementation [WIP]
│   │   └── types.rs         # Algorithm definitions and CLI args
│   ├── miner/               # ⛏️ Mining implementation
│   │   └── cpu/             # CPU mining with thread management
│   ├── pool/                # 🌐 Pool communication
│   │   ├── client.rs        # TCP client with DNS resolution
│   │   └── protocol.rs      # Stratum protocol (V1 + experimental V2)
│   ├── web_server.rs        # 📊 Real-time web dashboard
│   ├── sv2_protocol.rs      # 🔬 Stratum V2 implementation [WIP]
│   └── dashboard.html       # 🖥️ Web dashboard frontend
```

## ⚡ New Features

### DNS Resolution
Connect to pools using domain names instead of IP addresses:
```bash
# Before (IP only)
--pool 148.163.124.162:6118

# Now (DNS supported)
--pool pool.sha3x.supportxtm.com:6118
```

### Enhanced Pool Compatibility
Fixed compatibility with strict pools that require algorithm as array format:
- ✅ Lucky Pool (flexible format)
- ✅ Community pools (standard format)
- ✅ Strict pools (array format required)

### Real-time Web Dashboard
Monitor your mining operation with live statistics:
- Live hashrate charts
- Per-thread performance monitoring
- Share acceptance/rejection tracking
- Pool connection status
- WebSocket-based real-time updates

## 🔬 Experimental Features

### Bitcoin SHA256d Mining (Work in Progress)
- Basic SHA256d algorithm implementation
- Stratum V1 protocol support
- Pool connectivity testing
- **Status**: Under development, may have issues

### Stratum V2 (SV2) Support (Experimental)
- TCP connection testing to Job Declaration Server (JDS)
- Basic noise protocol framework
- **Status**: Very early development, connection testing only

```bash
# Test SV2 connection
cargo run --release -- --test-sv2 --pool 127.0.0.1:34254
```

## 🧪 Testing and Validation

### Run Performance Tests

```bash
# Test SHA3x performance
cargo test test_sha3x_correctness -- --nocapture

# Test thread scaling
cargo test test_thread_scaling -- --nocapture

# Test multi-algorithm support
cargo test bench_ -- --nocapture
```

## 🔧 Configuration Examples

### Production Mining (SHA3x)

```bash
# Tari mining with web dashboard
cargo run --release -- \
  --algo sha3x \
  --wallet 125ohcEDcG8sL4DcdtqZ6YLbSgVYFJWtGeCfHmRocTcyGNYRqMYidnfs1JQPijqQvqV5SLygC5ynxZH3zED5Rr9fPAW \
  --pool pool.sha3x.supportxtm.com:6118 \
  --worker production-rig \
  --threads 64 \
  --web
```

### Experimental Bitcoin Mining

```bash
# Bitcoin pool testing (experimental)
cargo run --release -- \
  --algo sha256 \
  --wallet 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2 \
  --pool stratum+tcp://slushpool.com:4444 \
  --worker test-worker \
  --threads 32
```

## 📊 Monitoring and Statistics

### Console Output
```
🚀 Starting SHA3x Miner
📍 Pool: pool.sha3x.supportxtm.com:6118
💳 Wallet: 125oh...9fPAW
👷 Worker: x99-cpu
🧵 Threads: 18
📊 Real-time dashboard will be available at: http://localhost:8080
✅ Connected to SHA3x pool
📋 New job sent: abcd1234 (height: 123456, difficulty: 1,000,000)
📊 Progress: 17.84 MH/s | Shares: 42
```

### Web Dashboard Features
- **Live Charts**: Real-time hashrate visualization
- **Thread Monitoring**: Individual thread performance
- **Share Tracking**: Accepted/rejected share statistics
- **Connection Status**: Pool connectivity and latency
- **Job Information**: Current mining job details

## 🛠 Troubleshooting

### Common Issues

**Pool Connection:**
- Use DNS names when possible: `pool.sha3x.supportxtm.com:6118`
- Test with different pools if connection fails
- Check firewall settings for outbound connections

**Algorithm Support:**
- SHA3x: Production ready ✅
- SHA256d: Experimental, may have issues ⚠️
- SV2: Connection testing only 🔬

**Web Dashboard:**
- Access at `http://localhost:8080` when using `--web` flag
- Ensure port 8080 is available
- Dashboard auto-starts with mining

## 🤝 Contributing

Contributions welcome! Current development priorities:

### High Priority
- Complete SHA256d Bitcoin mining implementation
- Improve SV2 protocol support
- Performance optimizations
- More comprehensive testing

### Areas for Contribution
- GPU acceleration (CUDA/OpenCL)
- Additional pool protocols
- Enhanced web dashboard features
- Documentation improvements

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tari Project](https://github.com/tari-project) for SHA3x algorithm
- Bitcoin community for Stratum V2 specifications
- Rust cryptographic libraries
- Mining community for testing and feedback

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/OIEIEIO/sha3x-miner/issues)
- **Discussions**: [GitHub Discussions](https://github.com/OIEIEIO/sha3x-miner/discussions)
- **Communities**: Tari Discord, Bitcoin mining forums

---

**Happy Mining!** ⛏️💎

*Note: Experimental features are under active development. For production mining, use SHA3x algorithm with established pools.*