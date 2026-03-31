# AI Agent Guide

This repository uses a structured, discovery-based context system for AI agents. All agent instructions and project context live in **`CLAUDE.md`** and the **`context/`** directory.

## For All AI Agents

1. **Start with `CLAUDE.md`** — the entry point for understanding this repository
2. **Then read `context/CONTEXT_INDEX.md`** — the discovery hub that maps you to relevant documentation based on your task
3. **Read only what you need** — the system is designed for just-in-time discovery, not upfront reading

## About the CLAUDE.md Convention

`CLAUDE.md` is a standardized format for AI agent instructions, originally designed for Claude Code but usable by any AI coding agent. It provides:

- **Discovery pattern** — how to find relevant documentation efficiently
- **Decision tree** — maps task types to specific documents
- **Anti-patterns** — what not to do (saves tokens and time)
- **Quick reference** — tech stack, commands, project structure

## Context Directory Structure

```
context/
├── CONTEXT_INDEX.md               # Read first — documentation map + decision tree
├── PROJECT_OVERVIEW.md            # Lightweight project overview (~149 lines)
├── CHANGELOG.md                   # Agent decision log (grep, don't read linearly)
├── architecture/
│   ├── SDK_ARCHITECTURE.md        # Core architecture, types, modules (~171 lines)
│   └── RUNTIME_ARCHITECTURE.md    # WIT, interfaces, ProxyWasm FFI (~134 lines)
└── development/
    └── BUILD_AND_CI.md            # Build system, CI pipelines (~142 lines)
```

All documents are under 171 lines — designed for single-sitting reads.
