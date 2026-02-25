<p align="center">
  <img src="https://www.matrix.org/images/matrix-logo.svg" alt="Matrix" width="150"/>
</p>

<h1 align="center">MXIndex</h1>

<p align="center">
  Federated Matrix homeserver discovery API
</p>

<p align="center">
  <a href="https://www.gnu.org/licenses/gpl-3.0.html">
    <img src="https://img.shields.io/badge/License-GPL%203.0-blue.svg" alt="License: GPL-3.0"/>
  </a>
  <a href="https://github.com/ilyeshdz/mxindex/actions/workflows/ci.yml">
    <img src="https://github.com/ilyeshdz/mxindex/actions/workflows/ci.yml/badge.svg" alt="CI Status"/>
  </a>
  <a href="https://matrix.org/#community">
    <img src="https://img.shields.io/badge/Matrix-Community-blue.svg" alt="Matrix Community"/>
  </a>
</p>

---

## What is MXIndex?

MXIndex is a federated Matrix homeserver discovery service that indexes public server information through standard Matrix APIs. It enables decentralized server discovery without requiring registration—servers are discovered by querying their public endpoints.

## Why MXIndex?

Matrix promotes decentralized communication, but discovering available homeservers in the federation remains challenging. MXIndex addresses this by exposing the information servers already publish through Matrix protocols, making server discovery transparent and accessible.

## Features

- **Zero Registration** - Servers indexed automatically via Matrix public APIs
- **Federation-Aware** - Discovers server implementations, versions, and delegation
- **Open Data** - Collects only publicly available server metadata
- **REST API** - Clean JSON endpoints with OpenAPI/Swagger documentation
- **High Performance** - Connection pooling, SQL-based filtering, Redis caching
- **Health Monitoring** - Built-in health check endpoint for orchestration
- **Structured Logging** - Production-ready logging with tracing

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | API information |
| `/health` | GET | Health check for container orchestration |
| `/servers` | GET | List all indexed servers (paginated) |
| `/servers` | POST | Add a new server to index |
| `/servers/<domain>` | GET | Get server status info |
| `/servers/search` | GET | Search/filter servers with query parameters |

## Quick Start

```bash
# Clone and build
cargo build --release

# Run with environment variables
export DATABASE_URL=postgres://user:pass@localhost/mxindex
export REDIS_URL=redis://localhost:6379
cargo run
```

### Using Docker

```bash
# Start the service
docker-compose up -d

# Add a server to index
curl -X POST http://localhost:8000/servers \
  -H "Content-Type: application/json" \
  -d '{"domain": "matrix.org"}'

# List servers
curl http://localhost:8000/servers

# Search servers
curl "http://localhost:8000/servers/search?registration_open=true&has_rooms=true"

# Check health
curl http://localhost:8000/health
```

The API runs at `http://localhost:8000`. Interactive documentation available at `/swagger`.

## Search Parameters

The `/servers/search` endpoint supports:

| Parameter | Type | Description |
|-----------|------|-------------|
| `search` | string | Search by domain, name, or description |
| `registration_open` | boolean | Filter by registration status |
| `has_rooms` | boolean | Filter by public rooms availability |
| `room_version` | string | Filter by supported room version |
| `sort_by` | string | Sort field (name, domain, created_at, public_rooms_count) |
| `sort_order` | string | Sort order (asc, desc) |
| `limit` | integer | Results per page (max 100) |
| `offset` | integer | Pagination offset |

## Data Collected

MXIndex only collects publicly available server metadata:

| Field | Source |
|-------|--------|
| name, description, logo, theme | `/.well-known/matrix/client` |
| registration_open | Server capabilities API |
| public_rooms_count | Public rooms directory |
| room_versions | Server capabilities |
| federation_version | `/_matrix/federation/v1/version` |
| delegated_server | `/.well-known/matrix/server` |
| version | Server version API |

No private data or user information is collected.

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│   Rocket   │────▶│  PostgreSQL │
│  (HTTP)     │     │   Server   │     │  (Diesel)   │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
                    ┌──────▼──────┐
                    │    Redis    │
                    │   (Cache)   │
                    └─────────────┘
```

## Technology Stack

- **Runtime**: Rust
- **Web Framework**: Rocket 0.5
- **Database**: PostgreSQL with Diesel ORM
- **Cache**: Redis
- **Matrix Client**: matrix-sdk
- **HTTP Client**: reqwest
- **API Docs**: rocket_okapi + Swagger UI
- **Logging**: tracing

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |

## License

GPL-3.0 - see [LICENSE](LICENSE) or <https://www.gnu.org/licenses/>

---

<p align="center">
  Built for the Matrix ecosystem
</p>
