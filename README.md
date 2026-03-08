# webact - token-efficient browser control for AI agents

A highly token efficient browser control tool that lets you control any Chromium-based browser via the Chrome DevTools Protocol. Ships as a Rust binary with zero runtime dependencies. Works as an MCP server with Claude Code, Claude Desktop, Cursor, Codex, Windsurf, Cline, ChatGPT Desktop, and any MCP-compatible client. Also works as a CLI skill with Claude Code, Cursor, Codex, Windsurf, Cline, Copilot, OpenCode, Goose, and any tool supporting the [Agent Skills](https://agentskills.io) spec.

No Playwright, no browser automation frameworks. Raw CDP over WebSocket.

## Install

### MCP Server (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/kilospark/webact/main/install.sh | sh
```

Downloads the `webact-mcp` binary and auto-configures any detected MCP clients (Claude Desktop, Claude Code, ChatGPT Desktop, Cursor, Windsurf, Cline, Codex).

### Agent Skill

```bash
npx skills add kilospark/webact
```

Works with Claude Code, Cursor, Codex, Windsurf, Cline, Copilot, OpenCode, Goose, and [40+ agents](https://github.com/vercel-labs/skills). Powered by Vercel's [skills](https://github.com/vercel-labs/skills) CLI.

### Manual MCP config

```json
{
  "mcpServers": {
    "webact": {
      "command": "webact-mcp"
    }
  }
}
```

For Claude Code:

```bash
claude mcp add webact webact-mcp
```

## Usage

Just tell your agent what you want:

```
check the top stories on Hacker News
navigate to github.com and show my notifications
search google for "best restaurants near me"
```

Or describe any goal - the agent will figure out the steps.

## How it works

The agent follows a **perceive-act loop**:

1. **Plan** - break the goal into steps
2. **Act** - navigate, click, type via CDP commands
3. **Perceive** - read the page to see what happened
4. **Decide** - adapt, continue, or report results
5. **Repeat** - until the goal is done

## Reading the page

webact provides multiple ways to read page content, each optimized for different needs:

| Need | Tool | Output |
|------|------|--------|
| Page content (articles, docs) | `read` | Clean text, no UI chrome |
| Full page + interaction targets | `text` | Text + numbered refs |
| Interactive elements only | `axtree -i` | Flat list of clickable/typeable elements |
| HTML structure/selectors | `dom` | Compact HTML |
| Visual layout | `screenshot` | PNG image |

**`read`** strips navigation, sidebars, ads, and returns just the main content as clean text with headings, lists, and paragraphs. Best for articles, docs, search results, and information retrieval.

**`text`** shows the full page in reading order, interleaving static text with interactive elements (numbered refs). Like a screen reader view. Generates a ref map so you can immediately use `click 12` or `type 3 hello`.

## Sessions

Each agent invocation gets its own **session** with isolated tab tracking. On `launch`, a unique session ID is generated and a fresh Chrome tab is created for that session.

- Multiple agents can work side by side in the same Chrome instance
- Each session only sees and controls its own tabs

## CLI

The `webact-rs` CLI wraps CDP:

```bash
webact-rs launch                  # Start browser, create session
webact-rs navigate <url>          # Go to a URL
webact-rs read [selector]         # Reader-mode text extraction (strips nav/sidebar/ads)
webact-rs text [selector]         # Full page in reading order with interactive refs
webact-rs dom [selector]          # Get compact DOM HTML
webact-rs dom --tokens=N          # Truncate DOM to ~N tokens
webact-rs axtree                  # Get accessibility tree (auto-capped at ~4k tokens)
webact-rs axtree -i               # Interactive elements with ref numbers
webact-rs axtree -i --diff        # Show only changes since last snapshot
webact-rs observe                 # Interactive elements as ready-to-use commands
webact-rs find <query>            # Find element by description
webact-rs screenshot              # Capture screenshot
webact-rs pdf [path]              # Save page as PDF
webact-rs click <sel|x,y|--text>  # Click by selector, coordinates, or text match
webact-rs doubleclick <sel>       # Double-click
webact-rs rightclick <sel>        # Right-click (context menu)
webact-rs hover <sel>             # Hover (tooltips/menus)
webact-rs focus <selector>        # Focus an element without clicking
webact-rs clear <selector>        # Clear an input field
webact-rs type <selector> <text>  # Type into an input (focuses first)
webact-rs keyboard <text>         # Type at current caret position (no selector)
webact-rs paste <text>            # Paste via clipboard event (for rich editors)
webact-rs select <sel> <value>    # Select option(s) from a dropdown
webact-rs upload <sel> <file>     # Upload file(s) to a file input
webact-rs humanclick <sel>        # Click with human-like mouse movement
webact-rs humantype <sel> <text>  # Type with variable delays
webact-rs drag <from> <to>        # Drag from one selector to another
webact-rs dialog accept|dismiss   # Handle alert/confirm/prompt dialogs
webact-rs waitfor <sel> [ms]      # Wait for element to appear (default 5s)
webact-rs waitfornav [ms]         # Wait for navigation to complete (default 10s)
webact-rs press <key>             # Press a key or combo (Enter, Ctrl+A, Meta+C)
webact-rs scroll <target> [px]    # Scroll: up, down, top, bottom, or selector
webact-rs eval <js>               # Run JavaScript in page context
webact-rs cookies                 # List cookies for current page
webact-rs cookies set <n> <v>     # Set a cookie
webact-rs cookies delete <name>   # Delete a cookie
webact-rs cookies clear           # Clear all cookies
webact-rs console                 # Show recent console output
webact-rs console errors          # Show only JS errors
webact-rs block <pattern>         # Block requests: images, css, fonts, media, scripts, or URL
webact-rs block --ads             # Block ads, analytics, and tracking (40+ patterns)
webact-rs block off               # Disable request blocking
webact-rs viewport <preset|w h>   # Set viewport (mobile, tablet, desktop, iphone, ipad)
webact-rs frames                  # List all frames/iframes
webact-rs frame <id|sel>          # Switch to a frame
webact-rs frame main              # Return to main frame
webact-rs tabs                    # List this session's tabs
webact-rs tab <id>                # Switch to a session-owned tab
webact-rs newtab [url]            # Open a new tab in this session
webact-rs close                   # Close current tab
webact-rs back / forward / reload # Navigation history
webact-rs activate                # Bring browser window to front (macOS)
webact-rs minimize                # Minimize browser window (macOS)
```

**Ref-based targeting:** After `axtree -i`, `observe`, or `text`, use the ref numbers directly as selectors - `click 1`, `type 3 hello`. Cached per URL.

## Token Stats

Each command is designed to minimize token usage while giving the agent enough context to decide its next step.

| Command | webact output | Playwright equivalent | Savings |
|---------|--------------|----------------------|---------|
| **brief** (auto) | ~200 chars | No equivalent - `page.content()` returns ~50k-500k chars | **~99%** |
| **read** | ~1k-4k chars (clean text) | No equivalent - manual extraction needed | - |
| **text** | ~1k-4k chars (text + refs) | `page.accessibility.snapshot()` ~10k-50k chars | **~90%** |
| **dom** | ~1k-4k chars (compact HTML) | `page.content()` ~50k-500k chars (full raw HTML) | **~95%** |
| **axtree -i** | ~500-1.5k chars (flat list) | `page.accessibility.snapshot()` ~10k-50k chars | **~95%** |

**Recommended flow for minimal token usage:**
1. State-changing commands auto-print the **brief** (~200 chars) - often enough to decide next step
2. Need to read page content? Use **read** - strips UI chrome, returns clean text
3. Need to see everything + interact? Use **text** - full page with refs
4. Need just interactive elements? Use **axtree -i** (~500 tokens)
5. Need HTML structure? Use **dom** with a selector to scope
6. Reserve **screenshot** for visual-heavy pages where text extraction is insufficient

## vs. Playwright-based tools

Several tools give AI agents browser control on top of Playwright: [agent-browser](https://github.com/vercel-labs/agent-browser) (Vercel), [Playwright MCP](https://github.com/microsoft/playwright-mcp) (Microsoft), [Stagehand](https://github.com/browserbase/stagehand) (Browserbase), and [Browser Use](https://github.com/browser-use/browser-use).

|  | **webact** | **Playwright-based tools** |
|--|-----------|--------------------------|
| **What it is** | Rust binary - MCP server + CLI | CLI / MCP server / SDK wrapping Playwright |
| **Architecture** | Direct CDP WebSocket to your Chrome | CLI/SDK &rarr; IPC &rarr; Playwright &rarr; bundled Chromium |
| **Install size** | Single binary, zero deps | ~200 MB+ (node_modules + Chromium download) |
| **Uses your browser** | Yes - your Chrome, your cookies, your logins | No - launches bundled Chromium with clean state |
| **User agent** | Your real Chrome user agent | Modified Playwright/Chromium UA - detectable |
| **Headed mode** | Always - you see what the agent sees | Headless by default |

### Token comparison (same pages, measured output)

| Scenario | **webact** | **Playwright-based*** | Savings |
|----------|-----------|------------------|---------|
| **Navigate + see page** | `navigate` = 186 chars | `open` + `snapshot -i` = 7,974 chars | **98%** |
| **Navigate + see page** | `navigate` = 756 chars | `open` + `snapshot -i` = 8,486 chars | **91%** |
| **Full page read** | `read` = ~3,000 chars | No equivalent (manual extraction) | - |
| **Full page + refs** | `text` = ~4,000 chars | `snapshot` = 104,890 chars | **96%** |
| **Interactive elements** | `axtree -i` = 5,997 chars | `snapshot -i` = 7,901 chars | **24%** |

## Build from source

```bash
git clone https://github.com/kilospark/webact.git
cd webact
cargo build --release
# Binaries: target/release/webact-rs (CLI), target/release/webact-mcp (MCP server)
```

## Requirements

- Any Chromium-based browser: Google Chrome, Microsoft Edge, Brave, Arc, Vivaldi, Opera, or Chromium
- No runtime dependencies (single Rust binary)

Auto-detected on macOS, Linux, Windows, and WSL. Set `CHROME_PATH` to override.

## License

MIT
