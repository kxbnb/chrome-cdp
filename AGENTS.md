# WebAct

Browser automation via the Chrome DevTools Protocol. Rust implementation.

## Binary

The `webact` CLI and `webact-mcp` MCP server are built from `src/`.

## Setup

Install via `install.sh` or build from source with `cargo build --release`.

## Sandbox Note

The CDP tool launches Chrome on an automatically discovered free port. The port is printed in the launch output and saved in the session state. If your agent sandbox blocks local network access, you'll need to allow connections to `127.0.0.1` on the assigned port.
