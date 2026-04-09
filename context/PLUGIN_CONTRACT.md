# Plugin Source Contract — Naming Conventions

This document describes the naming and structure conventions for `manifest.json` in this repo. These rules ensure the sync-reference-docs pipeline correctly maps source files to plugin reference docs and intent skills.

## Reference File Structure

Reference files in the plugin repo are organized by app_type:

```
plugins/gcore-fastedge/skills/
  scaffold/reference/
    http/                     # HTTP app blueprints
      base-rust.md            # Base skeleton
      kv-store-rust.md        # Feature blueprint
    cdn/                      # CDN app blueprints
      base-rust.md            # Base skeleton
      body-rust.md            # Feature blueprint
  fastedge-docs/reference/
    http/                     # HTTP app example patterns
      examples-kv-store-rust.md
    cdn/                      # CDN app example patterns
      examples-body-rust.md
    sdk-reference-rust.md     # Cross-cutting (no subfolder)
    host-services-rust.md     # Cross-cutting (no subfolder)
    cdn-apps-rust.md          # Cross-cutting app-type guide (no subfolder)
```

Note: `http/base-rust.md` and `cdn/base-rust.md` have the same filename but live in different subfolders. The pipeline's path-based intent matching resolves them to different intent files.

## File Naming Convention

**`{concept}-{lang}.md`** — concept first, language last. The subfolder provides the app_type context.

| Type | Pattern | Example |
|---|---|---|
| Base skeleton | `{appType}/base-{lang}.md` | `http/base-rust.md`, `cdn/base-rust.md` |
| Feature blueprint | `{appType}/{concept}-{lang}.md` | `cdn/body-rust.md` |
| Docs pattern | `{appType}/examples-{concept}-{lang}.md` | `cdn/examples-body-rust.md` |
| Cross-cutting SDK ref | `sdk-reference-{lang}.md` | `sdk-reference-rust.md` |
| Cross-cutting host services | `host-services-{lang}.md` | `host-services-rust.md` |
| Cross-cutting app-type guide | `{appType}-apps-{lang}.md` | `cdn-apps-rust.md` |

## Manifest target_mapping Rules

1. **reference_file** paths must include the `http/` or `cdn/` subfolder for per-example content (blueprints and patterns). Cross-cutting references (`sdk-reference`, `host-services`, `cdn-apps`) live directly under `fastedge-docs/reference/` with no subfolder
2. **section** should be `null` for all entries (each file is owned by one repo — no splicing)
3. **Dual-intent pattern**: each **feature** example gets two entries with the same `files` array:
   - `{name}-blueprint` → `scaffold/reference/{appType}/{concept}-{lang}.md`
   - `{name}-pattern` → `fastedge-docs/reference/{appType}/examples-{concept}-{lang}.md`

   **Exception**: Base skeleton examples (`http-hello-world`, `cdn-hello-world`) only get a `-blueprint` entry pointing to `scaffold/reference/{appType}/base-{lang}.md`. They have no `-pattern` counterpart because they don't demonstrate a reusable feature pattern.

## Intent File Matching

The pipeline resolves intent files by extracting the path suffix after `reference/` from the `reference_file` path. It looks for that same relative path inside the plugin's intent directory for this repo.

Example:
- `reference_file`: `plugins/.../scaffold/reference/cdn/body-rust.md`
- Path suffix: `cdn/body-rust.md`
- Intent lookup: `agent-intent-skills/fastedge-sdk-rust/cdn/body-rust.md`

This is why `http/base-rust.md` and `cdn/base-rust.md` can coexist — they resolve to `agent-intent-skills/fastedge-sdk-rust/http/base-rust.md` and `agent-intent-skills/fastedge-sdk-rust/cdn/base-rust.md` respectively.

## When Adding New Examples

1. Add source entries (paired `-blueprint` and `-pattern` for feature examples; `-blueprint` only for base skeletons) to `manifest.json`
2. Add target_mapping entries pointing to `{appType}/{concept}-{lang}.md` paths
3. Request intent files be created in `fastedge-plugin` repo (or create via PR):
   - `agent-intent-skills/fastedge-sdk-rust/{appType}/{concept}-{lang}.md` (scaffold)
   - `agent-intent-skills/fastedge-sdk-rust/{appType}/examples-{concept}-{lang}.md` (docs)
   - Each should reference `../_scaffold-blueprint-base.md` or `../_docs-pattern-base.md`
4. Create placeholder reference files at the target paths in the plugin repo
