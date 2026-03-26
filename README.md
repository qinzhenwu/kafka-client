# Kafka Client

A modern, cross-platform desktop client for Apache Kafka, built with Tauri v2 and Vue 3.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)

## Features

- **Multi-Cluster Management** - Connect and manage multiple Kafka clusters simultaneously
- **Topic Operations** - Create, delete, and configure topics with ease
- **Message Browser** - Consume messages with filtering, real-time streaming, and message production
- **Consumer Groups** - View consumer group details and reset offsets
- **Cross-Platform** - Native support for macOS, Windows, and Linux
- **Modern UI** - Clean, intuitive interface with dark/light themes
- **i18n Support** - English and Chinese language support

## Installation

### Download

Download the latest release from [GitHub Releases](https://github.com/qinzhenwu/kafka-client/releases):

- **macOS (Apple Silicon)**: Download `-aarch64.dmg` file
- **macOS (Intel)**: Download `-x64.dmg` file
- **Windows**: Download `.msi` or `.exe` file
- **Linux**: Download `.AppImage` or `.deb` file

### Build from Source

```bash
# Clone the repository
git clone https://github.com/qinzhenwu/kafka-client.git
cd kafka-client

# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build production release
npm run tauri build
```

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [pnpm](https://pnpm.io/) or npm

### Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Vue 3 + TypeScript |
| UI Framework | Naive UI |
| State Management | Pinia |
| Backend | Rust + Tauri v2 |
| Kafka Client | rdkafka (librdkafka) |

### Project Structure

```
kafka-client/
├── src/                    # Vue frontend
│   ├── components/         # Feature components
│   ├── layouts/            # Layout components
│   ├── stores/             # Pinia stores
│   ├── styles/             # Global styles
│   └── i18n/               # Internationalization
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers
│       ├── kafka/          # Kafka operations
│       └── models/         # Data models
└── docker/                 # Docker test environment
```

### Development Commands

```bash
# Start development server with hot reload
npm run tauri dev

# Type check frontend
npm run build

# Check Rust backend
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test
```

### Docker Test Environment

A local Kafka environment is provided for testing:

```bash
cd docker
docker-compose up -d      # Start Kafka + Zookeeper + Schema Registry
docker-compose down       # Stop environment
```

Services:
- Kafka: `localhost:9092`
- Schema Registry: `localhost:8081`

## Configuration

### Supported Security Protocols

- PLAINTEXT
- SSL
- SASL_PLAINTEXT
- SASL_SSL

### Supported SASL Mechanisms

- PLAIN
- SCRAM-SHA-256
- SCRAM-SHA-512
- GSSAPI (Kerberos)

## Roadmap

- [ ] Schema Registry integration
- [ ] Kafka Connect management
- [ ] Message serialization/deserialization
- [ ] Cluster metrics and monitoring
- [ ] Saved queries and filters

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Build smaller, faster, and more secure desktop apps
- [rdkafka](https://github.com/fede1024/rust-rdkafka) - Rust wrapper for librdkafka
- [Naive UI](https://www.naiveui.com/) - Vue 3 UI library