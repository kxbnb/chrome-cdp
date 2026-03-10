# WebAct Improvements Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve webact's token efficiency, element targeting reliability, and error clarity across screenshot, click, and DOM tools.

**Architecture:** All changes are self-contained within existing modules. Screenshot gets JPEG default + optional params. Click --text gets a preference heuristic for interactive elements. DOM gets better empty-result messaging. MCP layer updated to pass new args and handle JPEG MIME type.

**Tech Stack:** Rust, Chrome DevTools Protocol (CDP), serde_json

---

## File Structure

| File | Changes |
|------|---------|
| `src/commands/core.rs` | `cmd_screenshot()` — accept format/quality/selector/dimensions; `cmd_dom()` — fix empty output |
| `src/commands/mod.rs` | `dispatch()` — pass new screenshot args |
| `src/mcp_main.rs` | `map_tool_args()` — handle screenshot params; `handle_screenshot_output()` — dynamic MIME type; empty output fix |
| `src/lib.rs` | `locate_element_by_text()` — add interactive element preference |
| `tools.json` | Add screenshot params to schema |
| `MCP_INSTRUCTIONS.md` | Document new screenshot options |

---

## Chunk 1: Screenshot — JPEG Default + Params

### Task 1: Add screenshot parameters to tool schema

**Files:**
- Modify: `tools.json:89-93`

- [ ] **Step 1: Update tools.json screenshot definition**

Replace the current screenshot entry with:

```json
{
  "name": "webact_screenshot",
  "description": "Capture a screenshot of the current page or a specific element. Returns the image directly. Defaults to JPEG quality 80 for token efficiency.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "selector": { "type": "string", "description": "CSS selector to capture only that element's bounding box" },
      "format": { "type": "string", "enum": ["jpeg", "png"], "description": "Image format (default: jpeg)" },
      "quality": { "type": "integer", "description": "JPEG quality 1-100 (default: 80). Ignored for PNG." },
      "width": { "type": "integer", "description": "Scale output to this width in pixels (preserves aspect ratio)" }
    },
    "required": []
  }
}
```

- [ ] **Step 2: Commit**

```bash
git add tools.json
git commit -m "feat(screenshot): add format/quality/selector/width params to tool schema"
```

### Task 2: Wire screenshot args through MCP layer

**Files:**
- Modify: `src/mcp_main.rs:592-596` (map_tool_args, no-arg commands list)
- Modify: `src/mcp_main.rs:225-256` (handle_screenshot_output)

- [ ] **Step 1: Add screenshot to map_tool_args**

In `map_tool_args()`, remove `"screenshot"` from the no-arg commands match arm (line 593). Add a new match arm before it:

```rust
"screenshot" => {
    let mut args = Vec::new();
    if let Some(sel) = arguments.get("selector").and_then(Value::as_str) {
        if !sel.is_empty() {
            args.push(format!("--selector={sel}"));
        }
    }
    if let Some(fmt) = arguments.get("format").and_then(Value::as_str) {
        args.push(format!("--format={fmt}"));
    }
    if let Some(q) = arguments.get("quality").and_then(Value::as_i64) {
        args.push(format!("--quality={q}"));
    }
    if let Some(w) = arguments.get("width").and_then(Value::as_i64) {
        args.push(format!("--width={w}"));
    }
    args
}
```

- [ ] **Step 2: Update handle_screenshot_output for dynamic MIME type**

The file path now ends in `.jpeg` or `.png`. Update `handle_screenshot_output()`:

```rust
fn handle_screenshot_output(output: &str) -> Result<Vec<Value>> {
    let path = output
        .lines()
        .find_map(|line| line.trim().strip_prefix("Screenshot saved to "))
        .map(|s| s.trim())
        .unwrap_or_default();

    if path.is_empty() || !std::path::Path::new(path).exists() {
        return Ok(vec![json!({
            "type": "text",
            "text": output.trim_end()
        })]);
    }

    let bytes = fs::read(path)
        .with_context(|| format!("failed reading screenshot file: {path}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let mime = if path.ends_with(".png") { "image/png" } else { "image/jpeg" };

    Ok(vec![
        json!({
            "type": "image",
            "data": b64,
            "mimeType": mime
        }),
        json!({
            "type": "text",
            "text": output.trim_end()
        }),
    ])
}
```

- [ ] **Step 3: Commit**

```bash
git add src/mcp_main.rs
git commit -m "feat(screenshot): wire new params through MCP layer with dynamic MIME type"
```

### Task 3: Implement screenshot params in core

**Files:**
- Modify: `src/commands/core.rs:894-918` (cmd_screenshot)
- Modify: `src/commands/mod.rs:101` (dispatch call)

- [ ] **Step 1: Update dispatch to pass args to cmd_screenshot**

In `src/commands/mod.rs`, change line 101 from:

```rust
"screenshot" => cmd_screenshot(ctx).await,
```

to:

```rust
"screenshot" => cmd_screenshot(ctx, args).await,
```

- [ ] **Step 2: Rewrite cmd_screenshot to accept params**

Replace `cmd_screenshot` in `src/commands/core.rs`:

```rust
pub(super) async fn cmd_screenshot(ctx: &mut AppContext, args: &[String]) -> Result<()> {
    // Parse args
    let mut selector: Option<&str> = None;
    let mut format = "jpeg";
    let mut quality: u32 = 80;
    let mut scale_width: Option<u32> = None;

    for arg in args {
        if let Some(v) = arg.strip_prefix("--selector=") {
            selector = Some(v);
        } else if let Some(v) = arg.strip_prefix("--format=") {
            format = if v == "png" { "png" } else { "jpeg" };
        } else if let Some(v) = arg.strip_prefix("--quality=") {
            quality = v.parse().unwrap_or(80).clamp(1, 100);
        } else if let Some(v) = arg.strip_prefix("--width=") {
            scale_width = v.parse().ok();
        }
    }
    // Lifetime fix: own the format string before the borrow checker complains
    let format = format.to_string();

    let mut cdp = open_cdp(ctx).await?;
    prepare_cdp(ctx, &mut cdp).await?;

    // Build CDP params
    let mut params = json!({ "format": format });

    if format == "jpeg" {
        params["quality"] = json!(quality);
    }

    // If selector given, get element bounding rect for clip region
    if let Some(sel) = selector {
        let context_id = get_frame_context_id(ctx, &mut cdp).await?;
        let script = format!(
            r#"(function() {{
                const el = document.querySelector({sel});
                if (!el) return {{ error: 'Element not found: ' + {sel} }};
                el.scrollIntoView({{ block: 'center', inline: 'center', behavior: 'instant' }});
                const rect = el.getBoundingClientRect();
                return {{ x: rect.x, y: rect.y, width: rect.width, height: rect.height }};
            }})()"#,
            sel = serde_json::to_string(sel)?
        );
        let result = runtime_evaluate_with_context(&mut cdp, &script, true, false, context_id).await?;
        let value = result.pointer("/result/value").cloned().unwrap_or(Value::Null);
        if let Some(err) = value.get("error").and_then(Value::as_str) {
            bail!("{err}");
        }
        let x = value.get("x").and_then(Value::as_f64).unwrap_or(0.0);
        let y = value.get("y").and_then(Value::as_f64).unwrap_or(0.0);
        let w = value.get("width").and_then(Value::as_f64).unwrap_or(0.0);
        let h = value.get("height").and_then(Value::as_f64).unwrap_or(0.0);
        if w > 0.0 && h > 0.0 {
            params["clip"] = json!({
                "x": x, "y": y, "width": w, "height": h, "scale": 1
            });
        }
    }

    // If width given, compute device scale factor to achieve target width
    if let Some(target_w) = scale_width {
        // Get current viewport width
        let vp_result = runtime_evaluate(&mut cdp, "window.innerWidth", true, false).await?;
        let viewport_w = vp_result
            .pointer("/result/value")
            .and_then(Value::as_f64)
            .unwrap_or(1280.0);
        let scale = target_w as f64 / viewport_w;
        if let Some(clip) = params.get_mut("clip") {
            clip["scale"] = json!(scale);
        } else {
            params["clip"] = json!({
                "x": 0, "y": 0,
                "width": viewport_w,
                "height": viewport_w * 2.0, // will be clipped by actual page
                "scale": scale
            });
        }
    }

    let result = cdp.send("Page.captureScreenshot", params).await?;
    let data = result
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing screenshot data"))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .context("Failed to decode screenshot data")?;

    let sid = ctx
        .current_session_id
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let ext = if format == "png" { "png" } else { "jpeg" };
    let out = ctx.tmp_dir().join(format!("webact-screenshot-{sid}.{ext}"));

    fs::write(&out, bytes).with_context(|| format!("failed writing {}", out.display()))?;
    out!(ctx, "Screenshot saved to {}", out.display());
    cdp.close().await;
    Ok(())
}
```

- [ ] **Step 3: Build and verify compilation**

```bash
cargo build 2>&1 | tail -5
```

Expected: successful build (0 errors)

- [ ] **Step 4: Manual test — default JPEG screenshot**

Run webact CLI: `webact screenshot` — should produce a `.jpeg` file.

- [ ] **Step 5: Commit**

```bash
git add src/commands/core.rs src/commands/mod.rs
git commit -m "feat(screenshot): JPEG default, selector clip, quality, width scaling"
```

### Task 4: Update MCP instructions

**Files:**
- Modify: `MCP_INSTRUCTIONS.md`

- [ ] **Step 1: Add screenshot params to instructions**

In the `screenshot` entry of the reading tools table and in the relevant section, add:

```markdown
**`screenshot` options:** Defaults to JPEG quality 80. Use `format: "png"` for lossless. Use `selector` to capture only one element. Use `width` to downscale (e.g., 800 for token efficiency).
```

- [ ] **Step 2: Commit**

```bash
git add MCP_INSTRUCTIONS.md
git commit -m "docs: document screenshot params in MCP instructions"
```

---

## Chunk 2: Click --text Interactive Element Preference

### Task 5: Prefer interactive elements in text matching

**Files:**
- Modify: `src/lib.rs:591-670` (locate_element_by_text)

- [ ] **Step 1: Add interactive element preference to text search JS**

The current logic picks the shortest-text element matching the search string. When multiple elements match, a `<button>`, `<a>`, `<input>`, or `[role=button]` should be preferred over a `<div>` or `<span>`.

In `locate_element_by_text()` in `src/lib.rs`, replace the JS function body's matching logic. After the `for (const el of allElements(document))` loop's existing match logic, add an `isInteractive` check:

Replace lines 620-625 (the match scoring inside the loop) with:

```javascript
          const isInteractive = ['A','BUTTON','INPUT','SELECT','TEXTAREA','SUMMARY'].includes(el.tagName)
            || el.getAttribute('role') === 'button'
            || el.getAttribute('role') === 'link'
            || el.getAttribute('role') === 'menuitem'
            || el.getAttribute('role') === 'tab';
          const len = t.length;
          // Score: exact+interactive > exact > substring+interactive > substring
          // Within same tier, prefer shorter text (more specific element)
          if (exact) {{
            if (!best || !best.exact || (isInteractive && !best.interactive) || (isInteractive === best.interactive && len < bestLen)) {{
              best = {{ el, exact: true, interactive: isInteractive }}; bestLen = len;
            }}
          }} else if (has && !(best && best.exact)) {{
            if (!best || (isInteractive && !best.interactive) || (isInteractive === best.interactive && len < bestLen)) {{
              best = {{ el, exact: false, interactive: isInteractive }}; bestLen = len;
            }}
          }}
```

This ensures that when "Create filter" matches both a `<span>` and a `<button>`, the button wins.

- [ ] **Step 2: Build and verify**

```bash
cargo build 2>&1 | tail -5
```

Expected: successful build

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "fix(click): prefer interactive elements (button/a/input) in --text matching"
```

---

## Chunk 3: DOM Empty Result Fix

### Task 6: Return meaningful message for empty DOM output

**Files:**
- Modify: `src/mcp_main.rs:216-222` (empty output fallback)
- Modify: `src/commands/core.rs:153-214` (cmd_dom, when selector matches but element is empty)

- [ ] **Step 1: Fix the global empty-output fallback in MCP layer**

In `src/mcp_main.rs`, the empty output case at lines 217-219 returns "OK". Change it to include the command name for context:

```rust
    let text = output.trim_end().to_string();
    if text.is_empty() {
        Ok(vec![json!({ "type": "text", "text": format!("{command}: no output") })])
    } else {
        Ok(vec![json!({ "type": "text", "text": text })])
    }
```

Note: the `command` variable is already in scope (line 196-197 area). Verify it's accessible at this point; if not, pass it through.

- [ ] **Step 2: Handle DOM selector match with empty content**

In `cmd_dom()` in `src/commands/core.rs`, after the `ERROR: Element not found` check (after line 213), add a check for when the selector matched but returned empty DOM:

```rust
    if dom_output.is_empty() {
        if let Some(sel) = selector {
            out!(ctx, "Element matched but has no visible DOM content: {sel}");
        }
        cdp.close().await;
        return Ok(());
    }
```

Insert this block right after line 214 (`return Ok(());` of the error branch), before the `if max_tokens > 0` block.

- [ ] **Step 3: Build and verify**

```bash
cargo build 2>&1 | tail -5
```

Expected: successful build

- [ ] **Step 4: Commit**

```bash
git add src/mcp_main.rs src/commands/core.rs
git commit -m "fix(dom): return descriptive message instead of 'OK' for empty results"
```

---

## Chunk 4: Fill Tool (Multi-Field Form Filling)

### Task 7: Add fill tool for batch form input

**Files:**
- Modify: `tools.json` (add fill tool definition)
- Modify: `src/commands/interaction/forms.rs` (add cmd_fill)
- Modify: `src/commands/mod.rs` (add fill dispatch)
- Modify: `src/mcp_main.rs` (add fill to map_tool_args)

- [ ] **Step 1: Add fill tool to tools.json**

Add after the `type` tool entry:

```json
{
  "name": "webact_fill",
  "description": "Fill multiple form fields in one call. Each entry maps a CSS selector (or ref number) to a value. More efficient than multiple type calls.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "fields": {
        "type": "object",
        "description": "Map of CSS selector/ref -> value to type. Example: {\"#email\": \"user@example.com\", \"#password\": \"secret\"}",
        "additionalProperties": { "type": "string" }
      }
    },
    "required": ["fields"]
  }
}
```

- [ ] **Step 2: Add cmd_fill to forms.rs**

In `src/commands/interaction/forms.rs`, add:

```rust
pub(crate) async fn cmd_fill(ctx: &mut AppContext, fields: &[(String, String)]) -> Result<()> {
    if fields.is_empty() {
        bail!("Usage: webact fill requires at least one field");
    }
    let mut cdp = open_cdp(ctx).await?;
    prepare_cdp(ctx, &mut cdp).await?;

    let mut filled = 0;
    for (selector, value) in fields {
        let resolved = resolve_selector(ctx, selector)?;
        // Focus and clear existing value
        let node = cdp.send("DOM.querySelector", json!({
            "nodeId": cdp.send("DOM.getDocument", json!({})).await?["root"]["nodeId"],
            "selector": resolved
        })).await?;
        let node_id = node.get("nodeId").and_then(Value::as_i64).unwrap_or(0);
        if node_id == 0 {
            out!(ctx, "Skipped (not found): {selector}");
            continue;
        }
        cdp.send("DOM.focus", json!({ "nodeId": node_id })).await?;
        // Select all and replace
        dispatch_key(&mut cdp, "a", &["Meta"]).await?;
        sleep(Duration::from_millis(30)).await;
        for ch in value.chars() {
            dispatch_char(&mut cdp, ch).await?;
        }
        filled += 1;
        sleep(Duration::from_millis(50)).await;
    }

    out!(ctx, "Filled {filled}/{} fields", fields.len());
    out!(ctx, "{}", get_page_brief(&mut cdp).await?);
    cdp.close().await;
    Ok(())
}
```

Note: `dispatch_key` and `dispatch_char` are helpers that should already exist in forms.rs (check and use the existing keystroke dispatch pattern used by `cmd_type`). If they use a different approach (e.g., `Input.insertText`), match that pattern instead.

- [ ] **Step 3: Add fill to dispatch in mod.rs**

```rust
"fill" => {
    // args come as pairs: selector1 value1 selector2 value2 ...
    let fields: Vec<(String, String)> = args.chunks(2)
        .filter(|c| c.len() == 2)
        .map(|c| (c[0].clone(), c[1].clone()))
        .collect();
    cmd_fill(ctx, &fields).await
}
```

- [ ] **Step 4: Add fill to map_tool_args in mcp_main.rs**

```rust
"fill" => {
    let mut args = Vec::new();
    if let Some(fields) = arguments.get("fields").and_then(Value::as_object) {
        for (selector, value) in fields {
            args.push(selector.clone());
            args.push(value.as_str().unwrap_or_default().to_string());
        }
    }
    args
}
```

- [ ] **Step 5: Build and verify**

```bash
cargo build 2>&1 | tail -5
```

- [ ] **Step 6: Commit**

```bash
git add tools.json src/commands/interaction/forms.rs src/commands/mod.rs src/mcp_main.rs
git commit -m "feat: add fill tool for batch form input"
```

---

## Chunk 5: Update MCP Instructions for All Changes

### Task 8: Update MCP_INSTRUCTIONS.md

**Files:**
- Modify: `MCP_INSTRUCTIONS.md`

- [ ] **Step 1: Add fill tool and click --text notes to instructions**

Add to the tools table:

```markdown
**`fill`:** Fill multiple form fields in one call. Pass `fields` object mapping selectors to values. More efficient than multiple `type` calls for forms.
```

Update click section:

```markdown
**`click` --text preference:** When multiple elements match the text, interactive elements (button, a, input, [role=button]) are preferred over generic containers (div, span).
```

- [ ] **Step 2: Commit**

```bash
git add MCP_INSTRUCTIONS.md
git commit -m "docs: update MCP instructions for fill tool and click --text preference"
```
