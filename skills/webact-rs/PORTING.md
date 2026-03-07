# Rust Port Parity Tracker

Status: command-surface parity is implemented in `webact-rs`.

## Implemented commands

- `launch`, `connect`, `run`
- `navigate`, `back`, `forward`, `reload`
- `dom`, `axtree`, `axtree -i`, `axtree -i --diff`, `observe`, `find`
- `screenshot`, `pdf`
- `click`, `doubleclick`, `rightclick`, `hover`, `focus`, `clear`
- `type`, `keyboard`, `paste`, `select`, `upload`, `drag`
- `dialog`, `waitfor`, `waitfornav`, `press`, `scroll`, `eval`
- `cookies`, `console`, `network`, `block`, `viewport`
- `frames`, `frame`, `download`
- `tabs`, `tab`, `newtab`, `close`
- `activate`, `minimize`
- `humanclick`, `humantype`
- `lock`, `unlock`

## Known differences from JS implementation

- State files are intentionally isolated (`webact-rs-*`) so JS and Rust can run side-by-side without collisions.
- Request blocking uses `Network.setBlockedURLs` URL-pattern blocking. This matches many use cases but differs from JS's `Fetch.requestPaused` interception behavior.
- Dialog handling is implemented via one-shot page-level `alert/confirm/prompt` overrides stored in session state; this differs from JS's CDP `Page.javascriptDialogOpening` event handler approach.
- WSL host probing/path conversion behavior is not ported yet.

## Validation

- `cargo check` passes.
- `cargo run -- --help` lists all supported commands.

