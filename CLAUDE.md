# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Kafka Client** - A cross-platform Kafka desktop client built with Tauri v2 (Rust backend) + Vue 3 (TypeScript frontend).

## Development Commands

```bash
# Install dependencies
npm install

# Start development server (hot reload for both frontend and backend)
npm run tauri dev

# Build production release
npm run tauri build

# Type check frontend only
npm run build

# Check Rust backend
cd src-tauri && cargo check

# Run Rust tests (when present)
cd src-tauri && cargo test
```

## Prerequisites

- Node.js 18+
- Rust 1.70+

## Docker Test Environment

```bash
cd docker && docker-compose up -d    # Start Kafka + Zookeeper + Schema Registry
cd docker && docker-compose down     # Stop environment
```

Kafka is available at `localhost:9092`, Schema Registry at `localhost:8081`.

## Architecture

### Backend (Rust/Tauri)

```
src-tauri/src/
├── lib.rs              # Tauri app setup, registers all commands
├── error.rs            # Error types using thiserror
├── kafka/
│   ├── client.rs       # KafkaClientManager - manages connections via Arc<RwLock<HashMap>>
│   ├── admin.rs        # Topic CRUD operations
│   ├── producer.rs     # Message production
│   ├── consumer.rs     # Message consumption + streaming sessions
│   └── consumer_group.rs # Consumer group operations
├── commands/           # Tauri command handlers (invoke from frontend)
└── models/             # Shared data structures (Serialize/Deserialize)
```

**Key Pattern**: `KafkaClientManager` holds all connections in a shared state:
- `Arc<RwLock<HashMap<String, KafkaClient>>>` where key is client_id
- Each `KafkaClient` contains admin, consumer, and producer instances
- Injected into Tauri via `.manage()` for command access

**Streaming**: Real-time consumption uses `StreamingSessions` (also `Arc<RwLock<HashMap>>`) to track active streams and allow stopping by session_id. The backend emits `kafka-message` events via `app_handle.emit("kafka-message", msg)`; frontend listens via `listen<KafkaMessage>('kafka-message', (event) => ...)`.

**PartitionInfo**: Includes `leader_host` field (format: "host:port") resolved from broker metadata for displaying broker addresses in topic details.

### Frontend (Vue 3 + TypeScript)

```
src/
├── stores/             # Pinia stores - call Tauri commands via invoke()
├── components/         # Feature-based: cluster/, topic/, message/, consumer/
├── views/              # Route-level pages
├── i18n/               # zh-CN.ts, en-US.ts locale files
└── router/             # Vue Router 4 config
```

**State Flow**: Pinia stores persist cluster configs to localStorage and call Rust commands. Frontend types (e.g., `ClusterConfig`) mirror Rust structs for type safety.

### Tauri IPC

Frontend calls backend via `invoke<T>(command_name, args)`:
```typescript
const result = await invoke<ClusterInfo>('get_cluster_info', { clientId })
```

Commands are registered in `lib.rs` with `tauri::generate_handler![]`.

## Key Dependencies

**Rust**: `tauri`, `rdkafka` (librdkafka bindings), `tokio`, `serde_json`, `thiserror`, `chrono`

**TypeScript**: `vue`, `naive-ui`, `pinia`, `vue-router`, `vue-i18n`, `echarts`

## Code Conventions

- **Rust**: Use `thiserror` for error types. All async operations use `tokio`. Tauri commands should be thin wrappers around kafka module logic.
- **Vue**: Composition API with `<script setup>`. Components organized by feature. Types defined alongside stores.
- **i18n**: All user-facing strings must use `t('key')` from `vue-i18n`.
- **Global Dialogs**: For dialogs that persist across tab switches, store state in Pinia store and render in App.vue instead of component-local state.
- **Icon Sizes**: Close buttons use `:size="18"`, inline icons use `:size="14"`, sidebar icons use `:size="22"`.

## Security Configuration

Supported security protocols: `PLAINTEXT`, `SSL`, `SASL_PLAINTEXT`, `SASL_SSL`

Supported SASL mechanisms: `PLAIN`, `SCRAM_SHA_256`, `SCRAM_SHA_512`, `GSSAPI`

Frontend types (`ClusterConfig`) and Rust structs share identical field names for seamless serialization via `serde_json`.

## UI Components

- **Icons**: Use `lucide-vue-next` via `src/config/icons.ts` mapping. Import from `IconButton.vue` with icon name string.
- **Theming**: Use CSS variables from `src/styles/theme.css` (`--bg-primary`, `--accent`, `--border`, etc.). Never hardcode colors.