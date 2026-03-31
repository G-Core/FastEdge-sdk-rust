# AI Agent Instructions for FastEdge Rust SDK

## CRITICAL: Read Smart, Not Everything

**DO NOT read all context files upfront.** This repository uses a **discovery-based context system** to minimize token usage while maximizing effectiveness.

---

## Getting Started: Discovery Pattern

### Step 1: Read the Index (REQUIRED — ~140 lines)

**First action when starting work:** Read `context/CONTEXT_INDEX.md`

This lightweight file gives you:
- Project quick start (what this repo does in 10 lines)
- Documentation map organized by topic with sizes
- Decision tree for what to read based on your task
- Search patterns for finding information

### Step 2: Read Based on Your Task (JUST-IN-TIME)

Use the decision tree in CONTEXT_INDEX.md to determine what to read. **Only read what's relevant to your current task.**

**Examples:**

**Task: "Add a new WIT interface"**
- Read: `context/architecture/RUNTIME_ARCHITECTURE.md` (WIT section + change workflow)
- Read: existing `.wit` files in `wit/`
- Grep: `context/CHANGELOG.md` for similar past changes

**Task: "Fix the proc macro"**
- Read: `context/architecture/SDK_ARCHITECTURE.md` (attribute macro pattern)
- Read: `derive/src/lib.rs` (entry point)

**Task: "Add ProxyWasm wrapper for new host function"**
- Read: `context/architecture/RUNTIME_ARCHITECTURE.md` (ProxyWasm FFI section)
- Read: `src/proxywasm/key_value.rs` as template
- Grep: `src/proxywasm/mod.rs` for existing FFI declarations

**Task: "Add a new example"**
- Browse: `examples/http/wasi/` for similar existing example (**use `#[wstd::http_server]`, not `#[fastedge::http]`**)
- Read: `context/development/BUILD_AND_CI.md` (example build pattern)

### Step 3: Search, Don't Read Everything

**Use grep and search tools** instead of reading large files linearly:

- **CHANGELOG.md**: Will grow over time — always grep, never read end-to-end
- **Architecture docs** (~130-170 lines): Read specific sections by heading
- **Source code**: Use module names to navigate (`src/proxywasm/`, `derive/`, etc.)

---

## Decision Tree Reference

**Quick lookup for common tasks:**

| Task Type | What to Read |
|-----------|-------------|
| **Writing a new WASI-HTTP app** | SDK_ARCHITECTURE (wstd section) + `examples/http/wasi/` |
| **Working with basic sync handler** | SDK_ARCHITECTURE (fastedge::http section) + `examples/http/basic/` |
| **Adding a WIT interface** | RUNTIME_ARCHITECTURE (WIT + change workflow) |
| **Fixing proc macro** | SDK_ARCHITECTURE (macro section) + `derive/src/lib.rs` |
| **Adding ProxyWasm feature** | RUNTIME_ARCHITECTURE (FFI) + existing wrapper as template |
| **Adding an example** | Browse `examples/http/wasi/` (**use wstd, not fastedge::http**) |
| **Modifying HTTP client** | SDK_ARCHITECTURE (HTTP client + type conversion) |
| **Working with KV/secrets** | SDK_ARCHITECTURE (module structure) + `src/proxywasm/` |
| **Understanding the system** | PROJECT_OVERVIEW (~149 lines) |
| **Changing build/CI** | BUILD_AND_CI |
| **Modifying type conversions** | SDK_ARCHITECTURE (type conversion + body type) |
| **Updating dependencies** | PROJECT_OVERVIEW (deps table) + BUILD_AND_CI (workspace) |
| **Working with WASI-NN/ML** | RUNTIME_ARCHITECTURE (submodules) |
| **Debugging host errors** | ERROR_CODES (host codes 3100-3120) + HOST_SDK_CONTRACT (constraints) |
| **Using request properties** | PROPERTIES_REFERENCE + `examples/cdn/properties/` |
| **HTTP callout pause/resume** | REQUEST_LIFECYCLE (callout section) + HOST_SDK_CONTRACT |
| **Adding host function wrapper** | HOST_SDK_CONTRACT (FFI + memory) + RUNTIME_ARCHITECTURE (change workflow) |

---

## Anti-Patterns (What NOT to Do)

**Don't:** Read all 5 context docs upfront (~625 lines wasted if you only need one)
**Don't:** Read `src/lib.rs` (667 lines) end-to-end for a simple ProxyWasm change
**Don't:** Read DOCUMENTATION.md or AGENTS.md — they are superseded by this system
**Don't:** Read entire architecture docs when you need one specific section
**Don't:** Modify `wit/` files directly — they come from the `G-Core/FastEdge-wit` submodule

**Do:** Read `context/CONTEXT_INDEX.md` first — always
**Do:** Use grep to search CHANGELOG and large source files
**Do:** Read `examples/` for real-world usage patterns
**Do:** Read only sections relevant to your current task
**Do:** Follow the decision tree for targeted reading

---

## Critical Working Practices

### Task Checklists (ALWAYS USE)

When starting any non-trivial task (multi-step, multiple files, features, etc.):

1. Use `TaskCreate` to break work into discrete steps
2. Mark tasks `in_progress` when starting, `completed` when done
3. This helps track progress and prevents missed steps

### Parallel Agents

For independent work, spawn parallel agents:
- Research different subsystems simultaneously
- Build multiple examples at once
- Read multiple source files concurrently

### Documentation Maintenance

When you make significant changes, update the relevant context docs:

1. **After adding a feature:** Add a CHANGELOG.md entry (agent decision log)
2. **After changing architecture:** Update the relevant architecture doc
3. **After changing build config:** Update BUILD_AND_CI.md
4. **After adding a new module:** Update SDK_ARCHITECTURE.md and PROJECT_OVERVIEW.md

**CHANGELOG entry format:**
```markdown
## [YYYY-MM-DD] — Brief Description

### Overview
One sentence summary.

### Decisions
- Why this approach was chosen
- What alternatives were considered

### Changes
- Bullet list of what changed
```

---

## Context Organization

```
FastEdge-sdk-rust/
├── CLAUDE.md                              <- YOU ARE HERE
├── context/
│   ├── CONTEXT_INDEX.md                   <- Read first (discovery hub)
│   ├── PROJECT_OVERVIEW.md               <- New to codebase? Start here
│   ├── CHANGELOG.md                       <- Agent decision log (search with grep)
│   ├── architecture/
│   │   ├── SDK_ARCHITECTURE.md            <- Core arch, types, errors, modules
│   │   ├── RUNTIME_ARCHITECTURE.md        <- WIT, interfaces, ProxyWasm FFI
│   │   ├── HOST_SDK_CONTRACT.md           <- ABI contract, FFI functions, constraints
│   │   └── REQUEST_LIFECYCLE.md           <- Request phases, callout pause/resume
│   ├── reference/
│   │   ├── PROPERTIES_REFERENCE.md        <- Request properties (geo, IP, host, etc.)
│   │   └── ERROR_CODES.md                 <- Host status codes, SDK errors
│   └── development/
│       └── BUILD_AND_CI.md                <- Build system, CI, release, examples
├── src/                                   <- Rust source (core SDK)
│   ├── lib.rs                             <- Entry point, type conversions
│   ├── http_client.rs                     <- Outbound HTTP
│   ├── helper.rs                          <- Internal utilities
│   └── proxywasm/                         <- ProxyWasm FFI wrappers
├── derive/                                <- Proc macro crate
├── wit/                                   <- WIT definitions (submodule)
├── wasi-nn/                               <- ML interface (submodule)
├── examples/                              <- 30+ example apps
│   ├── http/basic/                        <- Sync handler examples
│   ├── http/wasi/                         <- Async handler examples
│   └── cdn/                               <- CDN-specific examples
├── README.md                              <- User-facing (not agent context)
└── AGENTS.md                              <- Pointer to this file
```

---

## Search Tips

**Find public API surface:**
```bash
grep -r "pub fn\|pub struct\|pub enum" src/
```

**Find feature-gated code:**
```bash
grep -r "#\[cfg(feature" src/
```

**Find FFI declarations:**
```bash
grep -r "extern \"C\"" src/proxywasm/
```

**Find handler examples:**
```bash
grep -r "fastedge::http" examples/
```

**Find WIT binding usage:**
```bash
grep -r "wit_bindgen" src/
```

---

## Quick Reference

**Tech Stack:** Rust (edition 2021), wit-bindgen 0.46, wasm32-wasip1
**Crate:** `fastedge` v0.3.5 on crates.io
**Docs:** https://docs.rs/fastedge
**License:** Apache-2.0

**Common Commands:**

| Command | Purpose |
|---------|---------|
| `cargo build --release` | Build (WASM, default target) |
| `cargo check` | Type-check only |
| `cargo clippy --all-targets --all-features` | Lint |
| `cargo fmt` | Format |
| `cargo test` | Run Rust-native tests |
| `cargo build --release --package <name>` | Build specific example |
| `cargo doc` | Generate docs |

---

## Summary

1. Read `context/CONTEXT_INDEX.md` first
2. Use the decision tree to find relevant docs
3. Read only what you need for your current task
4. Use grep for CHANGELOG and large files
5. Update context docs after significant changes
6. Use TaskCreate for multi-step work

---

**Last Updated**: March 2026
