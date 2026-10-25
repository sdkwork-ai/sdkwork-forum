> Migrated from `docs/superpowers/plans/2026-06-12-forum-foundation.md` on 2026-06-24.
> Owner: SDKWork maintainers

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the SDKWork forum foundation skeleton with database contracts, API contracts, SDK family metadata, route manifests, Rust module boundaries, and precise TODOs for follow-up implementation agents.

**Architecture:** The forum capability belongs to the canonical `communication` domain and uses `topic/reply` terminology. Authored contracts live under `apis/` and `specs/`; SDK family authority files and route manifests live under `sdks/`; Rust service, repository, route, host, server, and worker skeletons live under `crates/`. Open API public reads are anonymous and simple; app/backend APIs use SDKWork dual-token security.

**Tech Stack:** OpenAPI 3.1.2, YAML schema registry, Rust workspace crates, Node.js validation/materialization scripts, SDKWork SDK generation metadata.

---

### Task 1: Root SDKWork Workspace

**Files:**
- Create: `AGENTS.md`, `CODEX.md`, `CLAUDE.md`, `GEMINI.md`
- Create: `.sdkwork/README.md`, `.sdkwork/.gitignore`, `.sdkwork/skills/README.md`, `.sdkwork/plugins/README.md`
- Create: `README.md`, `agent.app.config.json`, `specs/component.spec.json`

- [x] **Step 1: Define root SDKWork standards references**
- [x] **Step 2: Declare domain, capability, and naming guardrails**
- [x] **Step 3: Document active root layout and verification commands**

### Task 2: Database Contract

**Files:**
- Create: `specs/forum-database.schema.yaml`
- Create: `docs/forum-database-design.md`

- [ ] **Step 1: Define schema registry metadata**
- [ ] **Step 2: Define all forum tables with fields, constraints, indexes, lifecycle, and TODOs**
- [ ] **Step 3: Document table groups and implementation priorities**

### Task 3: API Contracts

**Files:**
- Create: `apis/app-api/forum/openapi.yaml`
- Create: `apis/backend-api/forum/openapi.yaml`
- Create: `apis/open-api/forum/openapi.yaml`
- Create: `docs/forum-api-design.md`

- [ ] **Step 1: Define app API protected user operations**
- [ ] **Step 2: Define backend API protected operator operations**
- [ ] **Step 3: Define simple public Open API read operations**
- [ ] **Step 4: Verify Open API has no SDKWork dual-token or business context headers**

### Task 4: SDK Families And Route Manifests

**Files:**
- Create: `sdks/sdkwork-forum-app-sdk/**`
- Create: `sdks/sdkwork-forum-backend-sdk/**`
- Create: `sdks/sdkwork-forum-sdk/**`
- Create: `sdks/_route-manifests/**`
- Create: `tools/generators/materialize_forum_openapi.mjs`

- [ ] **Step 1: Create `sdk-manifest.json`, `sdk-manifest.json`, and component specs**
- [ ] **Step 2: Materialize OpenAPI authority files into SDK family workspaces**
- [ ] **Step 3: Define route manifests matching OpenAPI operations**

### Task 5: Rust Crate Skeletons

**Files:**
- Create: `crates/sdkwork-communication-forum-service/**`
- Create: `crates/sdkwork-communication-forum-repository-sqlx/**`
- Create: `crates/sdkwork-routes-forum-app-api/**`
- Create: `crates/sdkwork-routes-forum-backend-api/**`
- Create: `crates/sdkwork-routes-forum-open-api/**`
- Create: `crates/sdkwork-api-forum-standalone-gateway/**`
- Create: `crates/sdkwork-forum-service-host/**`
- Create: `crates/sdkwork-communication-forum-worker/**`

- [ ] **Step 1: Define domain models, commands, results, events, and value objects**
- [ ] **Step 2: Define service methods and repository ports with TODO comments**
- [ ] **Step 3: Define route descriptors, handler placeholders, and mapping modules**
- [ ] **Step 4: Define host/server/worker entrypoints**

### Task 6: Validation

**Files:**
- Create: `tests/static/forum-contract-boundary.test.mjs`
- Create: `tools/validators/validate_forum_contracts.mjs`

- [x] **Step 1: Write boundary test first and verify initial failure**
- [ ] **Step 2: Implement static validator**
- [ ] **Step 3: Run verification commands**
- [ ] **Step 4: Report results and remaining implementation TODOs**

