# Design: search, auto-dismiss, readurls, and fixes

Date: 2026-03-07

## 1. `search` command

Search the web via a real browser. Navigates to a search engine, types the query, submits, waits for results, runs `read` to extract clean results.

- Default engine: Google
- `--engine=bing|duckduckgo|<custom-url>` to override
- Custom engine URLs: query is appended (e.g. `--engine=https://kagi.com/search?q=`)
- MCP tool: `webact_search` with `query` (required), `engine` (optional string), `max_tokens` (optional integer)
- All requests go through Chrome, no direct HTTP

### Search engine URLs
- google: `https://www.google.com/search?q=`
- bing: `https://www.bing.com/search?q=`
- duckduckgo: `https://duckduckgo.com/?q=`
- Custom: use the provided URL directly, append query

### Flow
1. URL-encode query
2. Navigate to `{engine_url}{encoded_query}`
3. Wait for results to load
4. Run `read` extraction on the results page
5. Return extracted text (respects max_tokens)

## 2. Cookie/popup auto-dismiss on `navigate`

After every `navigate`, before returning the auto-brief, run a quick JS scan for common cookie consent elements and dismiss them.

- Default: enabled (auto-dismiss)
- `--no-dismiss` flag to skip
- MCP tool: add `no_dismiss` boolean property to existing `webact_navigate`

### Dismiss strategy (JS)
Check these selectors in order, click first match:
- `#onetrust-accept-btn-handler` (OneTrust)
- `#CookieBoxSaveButton` (CookieBox)
- `[data-testid="cookie-policy-manage-dialog-accept-button"]`
- `.cc-accept`, `.cc-dismiss` (CookieConsent)
- `#accept-cookies`, `#cookie-accept`
- `button[aria-label*="accept" i]`, `button[aria-label*="cookie" i]`
- Fallback: find button containing text "Accept All", "Accept Cookies", "I Agree", "Got it", "OK"

Wait 200ms after click for banner to disappear. If nothing matches, no-op.

## 3. `readurls` command

Read multiple URLs in parallel using tabs.

- Opens each URL in a new tab
- Runs `read` on each tab
- Returns combined output with `--- <url> ---` headers separating each
- Closes all opened tabs when done
- MCP tool: `webact_readurls` with `urls` (required string array), `max_tokens` (optional integer, per-URL cap)

### Flow
1. For each URL, open a new tab via newtab
2. Wait for all to load
3. Run read extraction on each tab
4. Combine results with URL headers
5. Close all opened tabs

## 4. Fix dom selector suggestions in MCP path

The selector suggestion feature (added in previous session) should append available selectors when `dom` returns "ERROR: Element not found". Debug why it didn't fire for `main`/`article` selectors on code.claude.com.

## 5. Update MCP_INSTRUCTIONS.md

- Document `search` command and engine options
- Document `readurls` command
- Document auto-dismiss behavior and `--no-dismiss` flag
- Update "Choosing the Right Reading Tool" table
- Update "Prefer webact" section to mention `search` replaces WebSearch

## 6. Update README.md and www/index.html

- Add `search` and `readurls` to command tables
- Mention auto-dismiss in feature descriptions
- Update tool/command counts
