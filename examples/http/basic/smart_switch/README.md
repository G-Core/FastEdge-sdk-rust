[← Back to examples](../../../README.md)

# Smart Switch

Toggles a SmartThings smart outlet on/off by wrapping multiple API calls into a single edge application.

## Configuration

- Environment variable: `PASSWORD` — password for simple authentication
- Environment variable: `DEVICE` — SmartThings device ID
- Environment variable: `TOKEN` — SmartThings API token

## How it works

1. Validates the `Authorization` header against the `PASSWORD` env var
2. Queries the SmartThings API for current device status
3. Sends a command to toggle the switch (on→off or off→on)
4. Handles API redirects automatically
