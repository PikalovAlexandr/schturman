# AGENTS.md

This directory (`/home/user/Загрузки`) is a downloads folder, not a dedicated repository. It contains the `opencode.json` config for OmniRoute model routing.

## Project: Программа Штурман (ЛОЦМАН:PLM Analog)

### Tech Stack
- **Language**: Rust (multi-platform: Linux, Windows, macOS)
- **Server Architecture**: Microservices with AI agent integration from inception

### Recommended Server Stack

**Database Layer** (choose based on requirements):
- **Primary Option**: PostgreSQL + TimescaleDB
  - Open source, ACID compliant, excellent JSONB support for flexible schemas
  - TimescaleDB extension for time-series data (change tracking, versioning)
  - Native full-text search and spatial data support
  - Mature Rust drivers: `tokio-postgres`, `sqlx`, `diesel`

- **Document Store**: PostgreSQL JSONB + optional Milvus/Qdrant for vector embeddings
  - Store CAD metadata, AI embeddings for semantic search
  - Milvus or Qdrant for vector similarity (AI-powered search)

- **Graph Database** (for BOM/dependency tracking): Neo4j Community or SurrealDB
  - SurrealDB: Rust-native, multi-model (document + graph + time-series)
  - Neo4j: mature ecosystem, Cypher query language
  - Use for part relationships, assemblies, where-used queries

**Backend Framework**:
- **Axum** or **Actix-web**
  - Axum: modern, tokio-native, type-safe, less boilerplate
  - Actix-web: mature, high performance, larger ecosystem
- **tonic** for gRPC microservices (inter-service communication)
- **sea-orm** or **sqlx** for database access (async, compile-time checked queries)

**AI Integration Layer**:
- **langchain-rust** or **llm-chain** for agent orchestration
- **candle** or **burn** for local ML inference (Rust-native)
- **qdrant-client** or **milvus-sdk-rust** for vector storage
- REST/gRPC endpoints for external AI services (OpenAI, Anthropic, local LLMs)

**File Storage**:
- **Object Storage**: MinIO (S3-compatible, self-hosted) or cloud S3/B2
- **Versioning**: git-like storage with content-addressable blobs
- Rust crate: `opendal` (unified storage abstraction)

**Message Queue** (for async tasks, AI agents):
- **NATS** or **RabbitMQ**
  - NATS: lightweight, Rust-native client, built-in clustering
  - RabbitMQ: mature, many patterns, `lapin` Rust client

**Search Engine**:
- **Meilisearch** (Rust-native, typo-tolerant) or **Typesense**
- Full-text search for parts, documents, change orders

**Recommended Stack Summary**:
```
Frontend: Tauri (Rust) or Electron with Rust backend
Backend: Axum + tonic (microservices)
Database: PostgreSQL + SurrealDB (graph) + Qdrant (vectors)
Storage: MinIO (S3)
Queue: NATS
Search: Meilisearch
ORM: sqlx or sea-orm
```

---

## Development Plan (Inspired by CADcraft Agent Distribution)

### Phase 0: Infrastructure & Architecture
**Goal**: Establish foundation, CI/CD, agent framework

**Agent Tasks**:
- **architect-agent**: Design microservice boundaries, API contracts (gRPC/REST)
  - Define service mesh: CAD service, PLM service, workflow service, AI agent service
  - API schema (protobuf for gRPC, OpenAPI for REST)
  - Database schemas and migration strategy

- **devops-agent**: Set up CI/CD, containerization
  - Dockerfile for each service
  - Docker Compose for local dev
  - GitHub Actions / GitLab CI pipelines
  - K8s manifests for production (Helm charts)

- **database-agent**: Initialize schema, migrations, seed data
  - PostgreSQL schema for parts, BOMs, users, revisions
  - SurrealDB graph schema for relationships
  - Qdrant collections for embeddings
  - Migration scripts (diesel-cli or sqlx-migrate)

**Deliverables**:
- `docs/architecture.md` - system design, service boundaries
- `docs/api-contracts/` - protobuf definitions, OpenAPI specs
- `infra/docker-compose.yml` - local dev environment
- `migrations/` - database schemas
- `.github/workflows/` - CI/CD pipelines

---

### Phase 1: Core PLM Backend Services
**Goal**: Implement fundamental PLM capabilities

**Service: Part Management (`services/part-service/`)**
- **agent: backend-core-agent**
  - CRUD for parts (number, name, description, type, status)
  - Part attributes (custom fields, metadata)
  - Part lifecycle states (draft, released, obsolete)
  - Multi-tenancy support
  - API: `CreatePart`, `GetPart`, `UpdatePart`, `ListParts`, `SearchParts`

**Service: Document Management (`services/document-service/`)**
- **agent: document-agent**
  - Upload/download CAD files, PDFs, images
  - Versioning (immutable storage, content-addressable)
  - Checkout/checkin workflow
  - Format conversion hooks (AI-assisted)
  - Thumbnail generation
  - API: `UploadDocument`, `DownloadDocument`, `GetVersions`, `CheckOut`, `CheckIn`

**Service: BOM Management (`services/bom-service/`)**
- **agent: bom-agent**
  - Tree structure: assemblies, sub-assemblies, parts
  - Where-used queries (SurrealDB graph queries)
  - BOM comparison (diff between revisions)
  - Flatten BOM (recursive explosion)
  - API: `CreateBOM`, `AddBOMItem`, `GetBOM`, `GetWhereUsed`, `CompareBOM`

**Service: Change Management (`services/change-service/`)**
- **agent: workflow-agent**
  - Engineering Change Orders (ECO), Change Requests (CR)
  - Approval workflows (state machine)
  - Affected items tracking
  - Notification system
  - API: `CreateECO`, `SubmitForApproval`, `ApproveECO`, `GetECOStatus`

**Cross-cutting: Authentication/Authorization**
- **agent: auth-agent**
  - JWT-based auth (or OAuth2)
  - Role-Based Access Control (RBAC)
  - Middleware for all services
  - Libraries: `jsonwebtoken`, `axum-extra` (auth layers)

**Deliverables**:
- `services/part-service/src/` - Rust service with Axum routes
- `services/document-service/src/` - file handling, S3 integration
- `services/bom-service/src/` - graph queries
- `services/change-service/src/` - workflow engine
- `services/auth-service/src/` - JWT issuer, RBAC

---

### Phase 2: AI Agent Integration Layer
**Goal**: Enable AI-powered features for all PLM operations

**Service: AI Orchestrator (`services/ai-orchestrator/`)**
- **agent: ai-integration-agent**
  - Agent registry (list available AI capabilities)
  - Task queue for AI jobs (NATS)
  - Prompt management and versioning
  - Result caching
  - API: `ExecuteAITask`, `GetTaskStatus`, `ListAgents`

**AI Capabilities to Implement**:
1. **Semantic Search Agent**
   - Embed part descriptions, BOM notes, documents
   - Qdrant integration
   - Natural language queries: "find all fasteners M6" → retrieve similar parts

2. **CAD Analysis Agent**
   - Extract metadata from CAD files (STEP, IGES parsers)
   - Generate thumbnails, extract dimensions
   - Suggest part classifications

3. **BOM Validation Agent**
   - Check for duplicate parts
   - Suggest standard parts from catalog
   - Validate material compatibility

4. **Change Impact Agent**
   - Analyze affected items for ECO
   - Generate change summary (LLM)
   - Risk assessment

5. **Documentation Agent**
   - Auto-generate datasheets from part data
   - Extract text from PDFs (OCR + LLM)
   - Translate documents

6. **Workflow Assistant Agent**
   - Suggest next approvers based on history
   - Auto-fill change request templates
   - Reminder notifications (smart scheduling)

**Implementation**:
- **agent: semantic-search-agent**
  - Embedding pipeline: part data → vector
  - Qdrant indexing
  - Query endpoint with similarity threshold

- **agent: cad-parser-agent**
  - Integrate `opencascade-rs` or `truck` (Rust CAD kernel)
  - Parse STEP/IGES, extract geometry
  - Store extracted data in PostgreSQL

- **agent: llm-agent**
  - Wrap OpenAI/Anthropic/local LLM APIs
  - Prompt templates for common tasks
  - Response parsing and validation

**Deliverables**:
- `services/ai-orchestrator/src/` - task dispatcher, agent registry
- `ai-agents/semantic-search/src/` - embedding + Qdrant
- `ai-agents/cad-parser/src/` - CAD file analysis
- `ai-agents/llm-tasks/src/` - LLM integration
- `docs/ai-agents.md` - agent capabilities, API reference

---

### Phase 3: Frontend Application
**Goal**: Cross-platform desktop app with rich UI

**Technology**: Tauri (Rust backend + web frontend) or Iced (pure Rust GUI)

**Option A: Tauri + SvelteKit/React**
- **agent: frontend-agent**
  - Tauri shell (Rust)
  - Web UI (SvelteKit for reactivity, or React)
  - gRPC-web or REST API client
  - Offline support (local SQLite cache)

**Option B: Iced (pure Rust)**
- **agent: rust-ui-agent**
  - Native Rust GUI (iced)
  - Direct gRPC client
  - Better performance, more complex development

**Core UI Modules**:
1. **Part Browser**
   - Tree/list view with search
   - AI-powered semantic search bar
   - Quick filters (status, type, owner)

2. **BOM Editor**
   - Tree view with drag-drop
   - Visual diff for BOM comparison
   - Where-used panel

3. **Document Viewer**
   - PDF/image viewer
   - CAD thumbnail preview
   - Version timeline

4. **Change Management Dashboard**
   - Kanban board for ECOs
   - Approval workflow visualization
   - AI-suggested actions

5. **AI Assistant Panel**
   - Chat interface for queries
   - Task status monitoring
   - Prompt history

**Deliverables**:
- `frontend/src/` - Tauri/Iced application
- `frontend/src-tauri/` - Rust backend for Tauri
- `frontend/ui/` - web components or Rust widgets

---

### Phase 4: Advanced Features
**Goal**: Production-ready capabilities

**Features**:
- **Collaboration**: Real-time editing (CRDT or operational transforms)
- **Reporting**: BOM exports (Excel, CSV, PDF), custom reports
- **Integration**: CAD tool plugins (SolidWorks, FreeCAD APIs)
- **Mobile**: Read-only mobile app (Tauri mobile or Flutter)
- **Analytics**: Usage metrics, AI performance tracking

**Agent Tasks**:
- **integration-agent**: CAD tool connectors, import/export bridges
- **reporting-agent**: Template engine, PDF generation
- **mobile-agent**: Simplified UI for mobile platforms
- **analytics-agent**: Telemetry, dashboards (Grafana integration)

---

## Agent Distribution Strategy (CADcraft-inspired)

### Agent Roles
1. **Architect Agent**: High-level design decisions, API contracts
2. **Backend Agent(s)**: One per microservice (part, BOM, document, change, AI)
3. **Database Agent**: Schema design, migrations, query optimization
4. **AI Agent(s)**: One per AI capability (search, parser, LLM tasks)
5. **Frontend Agent**: UI implementation
6. **DevOps Agent**: Infrastructure, deployment, monitoring
7. **Testing Agent**: Integration tests, load tests, AI validation
8. **Documentation Agent**: API docs, user guides, architecture diagrams

### Coordination
- **Central Orchestrator**: Project manager agent tracks milestones
- **Daily Standup**: Agents report progress, blockers (automated summary)
- **Code Review**: Cross-agent review before merge
- **Integration Tests**: Each agent writes tests for their service boundaries

---

## File Structure
```
shtorman-plm/
├── docs/
│   ├── architecture.md         # System design
│   ├── api-contracts/          # gRPC protos, OpenAPI
│   ├── ai-agents.md            # AI capabilities reference
│   └── development-plan.md     # This plan
├── services/
│   ├── part-service/
│   ├── document-service/
│   ├── bom-service/
│   ├── change-service/
│   ├── auth-service/
│   └── ai-orchestrator/
├── ai-agents/
│   ├── semantic-search/
│   ├── cad-parser/
│   ├── llm-tasks/
│   └── shared/                 # Common AI utilities
├── frontend/
│   ├── src/                    # UI code
│   └── src-tauri/              # Tauri backend (if used)
├── infra/
│   ├── docker-compose.yml
│   ├── k8s/                    # Kubernetes manifests
│   └── terraform/              # Cloud infrastructure (optional)
├── migrations/                 # Database migrations
├── proto/                      # Shared protobuf definitions
├── shared/                     # Shared Rust crates
│   ├── models/                 # Common data models
│   ├── auth/                   # Auth utilities
│   └── clients/                # Service clients
├── tests/
│   ├── integration/            # End-to-end tests
│   └── load/                   # Performance tests
├── Cargo.toml                  # Workspace root
└── README.md
```

---

## Development Workflow

### Step 1: Initialize Workspace
```bash
cargo new --bin services/part-service
cargo new --lib shared/models
cargo new --bin ai-agents/semantic-search
# ... repeat for all services
```

### Step 2: Define Workspace in Root `Cargo.toml`
```toml
[workspace]
members = [
    "services/*",
    "ai-agents/*",
    "shared/*",
    "frontend/src-tauri",
]
resolver = "2"

[workspace.dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
serde = { version = "1", features = ["derive"] }
# ... shared dependencies
```

### Step 3: Agent Assignment
- Each agent creates a branch: `feature/part-service`, `feature/ai-orchestrator`
- Implements service with tests
- Opens PR with API documentation
- Integration tests added by testing-agent

### Step 4: CI/CD Pipeline
- **On PR**: Lint (clippy), format (rustfmt), unit tests, integration tests
- **On merge**: Build Docker images, deploy to staging
- **On tag**: Deploy to production

---

## Key Rust Crates

**Web Frameworks**:
- `axum` - HTTP server
- `tonic` - gRPC server/client
- `tower` - middleware

**Database**:
- `sqlx` - async SQL with compile-time checks
- `sea-orm` - ORM
- `surrealdb` - graph database client

**AI/ML**:
- `candle` - ML framework (Rust)
- `qdrant-client` - vector database
- `llm-chain` - LLM orchestration
- `tokenizers` - Hugging Face tokenizers

**Storage**:
- `opendal` - unified storage (S3, file, etc.)
- `aws-sdk-s3` - AWS S3 client

**Messaging**:
- `async-nats` - NATS client
- `lapin` - RabbitMQ client

**Frontend**:
- `tauri` - cross-platform desktop
- `iced` - pure Rust GUI
- `egui` - immediate mode GUI

**Utilities**:
- `tracing` - structured logging
- `serde_json` - JSON serialization
- `config` - configuration management
- `chrono` - date/time

---

## Next Steps

1. **Create GitHub repository** with structure above
2. **Initialize Rust workspace** with all services as members
3. **Set up PostgreSQL + SurrealDB + Qdrant** in Docker Compose
4. **Implement auth-service** first (all services depend on it)
5. **Parallel development**: 
   - Backend agents build services (part, document, BOM, change)
   - AI agents build capabilities (semantic search, CAD parser)
   - Frontend agent builds UI shell
6. **Weekly integration**: Merge all services, run end-to-end tests
7. **Iterate**: Add AI features incrementally, gather feedback

---

## Performance Targets
- Part CRUD: <50ms p95
- BOM queries (1000 parts): <200ms
- Semantic search: <100ms
- Document upload (100MB): <5s
- Concurrent users: 100+ (horizontal scaling)

---

## Security Considerations
- JWT with short expiration, refresh tokens
- Row-level security in PostgreSQL
- File encryption at rest (MinIO with KMS)
- Audit logging for all changes
- Rate limiting on API endpoints
- Input validation with `validator` crate

---

## Monitoring & Observability
- **Metrics**: Prometheus + Grafana
- **Tracing**: Jaeger or Tempo (OpenTelemetry)
- **Logging**: Loki or ELK stack
- **Alerting**: AlertManager
- Rust integration: `tracing-opentelemetry`, `metrics` crate

---

## AI Agent API Examples

### Semantic Search Agent
```rust
// POST /ai/semantic-search
{
  "query": "M6 stainless steel fasteners",
  "filters": { "status": "released" },
  "limit": 10
}

// Response
{
  "results": [
    { "part_id": "P123", "name": "M6x20 Bolt SS", "score": 0.95 },
    { "part_id": "P456", "name": "M6 Nut SS A2", "score": 0.89 }
  ]
}
```

### CAD Analysis Agent
```rust
// POST /ai/analyze-cad
{
  "document_id": "DOC789",
  "tasks": ["extract_dimensions", "generate_thumbnail"]
}

// Response (async task)
{
  "task_id": "TASK123",
  "status": "processing"
}

// GET /ai/task/TASK123
{
  "status": "completed",
  "result": {
    "dimensions": { "length": 100, "width": 50, "height": 30 },
    "thumbnail_url": "s3://bucket/thumbnails/DOC789.png"
  }
}
```

### LLM Change Summary Agent
```rust
// POST /ai/summarize-change
{
  "eco_id": "ECO001",
  "context": { "affected_parts": ["P1", "P2"], "reason": "cost reduction" }
}

// Response
{
  "summary": "ECO001 modifies 2 parts to reduce manufacturing costs by switching to alternative supplier. Estimated savings: $500/year. No functional impact."
}
```

---

## Open Questions for User

1. **Target scale**: How many users? Parts? Documents? (affects DB choice)
2. **CAD formats**: Which formats are priority? (STEP, IGES, native SolidWorks, etc.)
3. **Deployment**: On-premise or cloud? (affects infra setup)
4. **AI models**: Use cloud APIs (OpenAI) or self-hosted LLMs? (affects cost/privacy)
5. **Integration priority**: Which external systems to integrate first? (ERP, MRP, etc.)
6. **Licensing**: Open-source or commercial? (affects dependency choices)

---

## Estimated Timeline (Agent Team)

- **Phase 0** (Infrastructure): 2 weeks
- **Phase 1** (Core Backend): 6 weeks (parallel development)
- **Phase 2** (AI Integration): 4 weeks
- **Phase 3** (Frontend): 6 weeks (overlap with Phase 2)
- **Phase 4** (Advanced Features): 4 weeks
- **Testing & Hardening**: 2 weeks

**Total**: ~24 weeks (6 months) for MVP with AI capabilities

---

This plan provides a structured, agent-distributed approach to building a modern PLM system in Rust with AI integration from the ground up. Each agent has clear responsibilities, and the microservice architecture allows parallel development and independent scaling.
