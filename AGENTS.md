---
doc_type: policy
audience: bot
lang: en
tags: ['ai-agents', 'rules', 'critical', 'codex']
last_modified: 2026-04-02T00:00:00Z
copyright: '© 2026 gcore.com'
---

RULES FOR AI AGENTS
======================

TL;DR: Keep command output short. Do not take actions unless asked.
Do not waste tokens on experiments. Do only what is asked.

COMMUNICATION STYLE
===================

- Use English by default; if the user writes in another language, use that language
- Use an informal tone, avoid formal business language
- Question ideas and suggest alternatives — do not just agree with everything
- Think for yourself instead of agreeing to be polite

INVARIANTS
==========

- NEVER do anything beyond the assigned task
- NEVER change code that was not asked to change
- NEVER "improve" or "optimize" without a clear request
- NEVER use scripts for mass code replacements
- NEVER make architecture decisions on your own
- ALWAYS keep command output short — every extra line = wasted tokens
- ALWAYS think before acting — do not repeat checks, remember context
- ALWAYS ask an expert when the solution is not clear
- ALWAYS tell apart an observation from an action request:
  observation ("works oddly") → discuss, DO NOT fix
  request ("fix this") → act

PROJECT CONTEXT
===============

see CLAUDE.md
