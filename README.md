<p align="center">
  <img src="https://www.matrix.org/images/matrix-logo.svg" alt="Matrix" width="150"/>
</p>

<h1 align="center">MXIndex</h1>

<p align="center">
  Federated Matrix homeserver discovery
</p>

<p align="center">
  <a href="https://www.gnu.org/licenses/gpl-3.0.html">
    <img src="https://img.shields.io/badge/License-GPL%203.0-blue.svg" alt="License: GPL-3.0"/>
  </a>
  <a href="https://matrix.org/#community">
    <img src="https://img.shields.io/badge/Matrix-Community-blue.svg" alt="Matrix Community"/>
  </a>
</p>

---

## What is MXIndex?

MXIndex helps discover Matrix homeservers. Like Matrix itself, it's decentralized—any server can be indexed without registration, just by querying its public API.

## Why?

Matrix is about decentralized communication, but finding servers in the federation isn't easy. MXIndex makes server discovery transparent by exposing what servers already publish through standard Matrix APIs.

## Features

- **Zero registration** - Servers are discovered automatically via Matrix APIs
- **Federation-aware** - Detects server implementation, versions, and delegation
- **Open data** - Only collects what's already public
- **REST API** - Simple JSON endpoints
- **Interactive docs** - Swagger UI included

## Quick Start

```bash
# Run the server
cargo run

# Add a server
curl -X POST http://localhost:8000/servers \
  -H "Content-Type: application/json" \
  -d '{"domain": "matrix.org"}'

# List servers
curl http://localhost:8000/servers
```

Server runs at `http://localhost:8000`. API docs at `/swagger`.

## What We Collect

| Field | From |
|-------|------|
| name, description, logo, theme | `/.well-known/matrix/client` |
| registration open | Server capabilities |
| public rooms | Public rooms API |
| room versions | Server capabilities |
| federation version | `/_matrix/federation/v1/version` |
| delegated server | `/.well-known/matrix/server` |

Only public server metadata—no private data.

## Tech

- Rust + Rocket
- Diesel (SQLite)
- matrix-sdk

## License

GPL-3.0 - see [LICENSE](LICENSE) or <https://www.gnu.org/licenses/>

---

<p align="center">
  Built for the Matrix ecosystem
</p>
