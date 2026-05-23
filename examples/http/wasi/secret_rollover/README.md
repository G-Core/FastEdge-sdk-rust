[← Back to examples](../../../README.md)

# Secret Rollover (WASI)

Demonstrates slot-based secret retrieval for secret rotation scenarios using `secret::get_effective_at()`.

Compares the current secret value with the value effective at a given slot, returning both as JSON. This lets you validate that rotation is working correctly before removing the old slot.

## Usage

| Request header | Default | Description |
|---|---|---|
| `x-secret-name` | `TOKEN_SECRET` | Name of the secret to query |
| `x-slot` | current Unix timestamp | Slot value passed to `get_effective_at` |

## How slots work

Slots use a **greatest-match rule**: the slot with the highest value that is `<= effective_at` is returned.

```
Secret slots: { 0: "old-password", 1741790697: "new-password" }

get_effective_at("TOKEN_SECRET", 0)          → "old-password"
get_effective_at("TOKEN_SECRET", 100)        → "old-password"
get_effective_at("TOKEN_SECRET", 1741790697) → "new-password"
get_effective_at("TOKEN_SECRET", 9999999999) → "new-password"
```

When used with token `iat` (issued-at) timestamps, `get_effective_at(name, claims.iat)` returns the password that was active when the token was issued — enabling zero-downtime rotation without invalidating existing tokens.

## What it returns

```json
{
  "secret_name": "TOKEN_SECRET",
  "slot": 0,
  "current": "new-password",
  "effective_at_slot": "old-password",
  "is_same": false
}
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/secret_rollover.wasm
```

## APIs used

- `fastedge::secret::get(name)` — current (latest-slot) secret value; `Ok(Option<String>)`
- `fastedge::secret::get_effective_at(name, slot)` — secret value at a given slot; `Ok(Option<String>)`
