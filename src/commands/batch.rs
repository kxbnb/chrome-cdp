use super::*;

#[derive(Clone, Copy, Debug, Default)]
struct BatchActionOptions {
    wait: Option<u64>,
    retries: u64,
    retry_delay: u64,
    optional: bool,
}

#[derive(Debug)]
struct BatchActionOutcome {
    ok: bool,
    output: String,
    error: Option<String>,
    attempts: u64,
}

pub(crate) async fn cmd_batch(ctx: &mut AppContext, args: &[String]) -> Result<()> {
    if args.is_empty() {
        bail!("Usage: webact batch '<json>'");
    }

    let input: Value =
        serde_json::from_str(&args[0]).context("Failed to parse batch JSON input")?;

    let actions = input
        .get("actions")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("'actions' array is required"))?;

    let global_delay = input.get("delay").and_then(Value::as_u64).unwrap_or(0);

    let total = actions.len();
    let mut results: Vec<Value> = Vec::new();
    let mut error: Option<String> = None;

    for (i, action) in actions.iter().enumerate() {
        let tool = action
            .get("tool")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("Action {} missing 'tool' field", i))?;
        let options = parse_action_options(action);

        // Build args from the action object. Batch-control keys are consumed here
        // and are not forwarded into the underlying command dispatcher.
        let action_args = build_action_args(tool, action);
        let outcome = execute_action_with_retries(ctx, tool, &action_args, options).await;

        results.push(build_action_result(tool, &outcome, options));

        if !outcome.ok && !options.optional {
            error = outcome.error.clone();
            break;
        }

        apply_inter_action_wait(
            ctx,
            tool,
            &outcome,
            options.wait,
            global_delay,
            i + 1 < total,
        )
        .await;
    }

    let completed = results.len();
    let output = json!({
        "completed": completed,
        "total": total,
        "error": error,
        "results": results
    });

    out!(ctx, "{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn parse_action_options(action: &Value) -> BatchActionOptions {
    BatchActionOptions {
        wait: action.get("wait").and_then(Value::as_u64),
        retries: action.get("retries").and_then(Value::as_u64).unwrap_or(0),
        retry_delay: action
            .get("retry_delay")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        optional: action
            .get("optional")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

async fn execute_action_with_retries(
    ctx: &mut AppContext,
    tool: &str,
    action_args: &[String],
    options: BatchActionOptions,
) -> BatchActionOutcome {
    let max_attempts = options.retries.saturating_add(1);

    for attempt in 1..=max_attempts {
        ctx.output.clear();
        match crate::commands::dispatch(ctx, tool, action_args).await {
            Ok(()) => {
                return BatchActionOutcome {
                    ok: true,
                    output: ctx.drain_output(),
                    error: None,
                    attempts: attempt,
                };
            }
            Err(e) => {
                let err_msg = format!("{e}");
                let output = combine_output_and_error(ctx.drain_output(), &err_msg);
                if attempt < max_attempts {
                    if options.retry_delay > 0 {
                        sleep(Duration::from_millis(options.retry_delay)).await;
                    }
                    continue;
                }
                return BatchActionOutcome {
                    ok: false,
                    output,
                    error: Some(err_msg),
                    attempts: attempt,
                };
            }
        }
    }

    unreachable!("retry loop always returns");
}

fn combine_output_and_error(output: String, err_msg: &str) -> String {
    if output.trim().is_empty() {
        err_msg.to_string()
    } else {
        format!("{}\n{err_msg}", output.trim_end())
    }
}

fn build_action_result(
    tool: &str,
    outcome: &BatchActionOutcome,
    options: BatchActionOptions,
) -> Value {
    let mut result = json!({
        "tool": tool,
        "ok": outcome.ok,
        "output": outcome.output.trim_end(),
        "attempts": outcome.attempts
    });
    if let Some(obj) = result.as_object_mut() {
        if options.optional {
            obj.insert("optional".to_string(), json!(true));
        }
        if let Some(err) = &outcome.error {
            obj.insert("error".to_string(), json!(err));
        }
    }
    result
}

fn is_smart_wait_command(tool: &str) -> bool {
    matches!(
        tool,
        "navigate" | "click" | "fill" | "select" | "type" | "doubleclick" | "humanclick"
    )
}

fn compute_wait_ms(
    tool: &str,
    succeeded: bool,
    per_action_wait: Option<u64>,
    global_delay: u64,
) -> u64 {
    if let Some(wait) = per_action_wait {
        wait
    } else if succeeded && is_smart_wait_command(tool) {
        500 + global_delay
    } else {
        global_delay
    }
}

async fn apply_inter_action_wait(
    ctx: &mut AppContext,
    tool: &str,
    outcome: &BatchActionOutcome,
    per_action_wait: Option<u64>,
    global_delay: u64,
    has_next_action: bool,
) {
    let wait_ms = compute_wait_ms(tool, outcome.ok, per_action_wait, global_delay);

    if wait_ms == 0 || !has_next_action {
        return;
    }

    sleep(Duration::from_millis(wait_ms)).await;
    // Sync with browser rendering after every wait (smart or explicit).
    // Double requestAnimationFrame ensures at least one full paint cycle
    // has completed — critical for canvas-based apps where wall-clock
    // sleep alone doesn't guarantee the screen has updated.
    if let Ok(mut cdp) = open_cdp(ctx).await {
        let _ = runtime_evaluate(
            &mut cdp,
            "new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))",
            true,
            true,
        )
        .await;
        cdp.close().await;
    }
}

fn build_action_args(tool: &str, action: &Value) -> Vec<String> {
    let mut args = Vec::new();

    match tool {
        "navigate" => {
            if let Some(url) = action.get("url").and_then(Value::as_str) {
                args.push(url.to_string());
            } else if let Some(target) = action.get("target").and_then(Value::as_str) {
                args.push(target.to_string());
            }
            if action
                .get("no_dismiss")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("--no-dismiss".to_string());
            }
        }
        "click" | "doubleclick" | "rightclick" | "hover" | "humanclick" => {
            if let Some(target) = action.get("target").and_then(Value::as_str) {
                args.extend(target.split_whitespace().map(String::from));
            }
        }
        "type" | "humantype" => {
            if let Some(sel) = action.get("selector").and_then(Value::as_str) {
                args.push(sel.to_string());
            }
            if let Some(text) = action.get("text").and_then(Value::as_str) {
                args.push(text.to_string());
            }
        }
        "keyboard" | "paste" => {
            if let Some(text) = action.get("text").and_then(Value::as_str) {
                args.push(text.to_string());
            }
        }
        "press" => {
            if let Some(key) = action.get("key").and_then(Value::as_str) {
                args.push(key.to_string());
            }
        }
        "screenshot" => {
            if let Some(r) = action.get("ref").and_then(Value::as_i64) {
                args.push(format!("--ref={r}"));
            } else if let Some(sel) = action.get("selector").and_then(Value::as_str) {
                if !sel.is_empty() {
                    args.push(format!("--selector={sel}"));
                }
            }
            if let Some(s) = action.get("scale").and_then(Value::as_f64) {
                args.push(format!("--scale={s}"));
            }
            if action.get("full").and_then(Value::as_bool).unwrap_or(false) {
                args.push("--full".to_string());
            }
            if let Some(o) = action.get("output").and_then(Value::as_str) {
                if !o.is_empty() {
                    args.push(format!("--output={o}"));
                }
            }
        }
        "scroll" => {
            if let Some(target) = action.get("target").and_then(Value::as_str) {
                args.extend(target.split_whitespace().map(String::from));
            }
            if let Some(px) = action.get("pixels").and_then(Value::as_i64) {
                args.push(px.to_string());
            }
        }
        "waitfor" => {
            if let Some(sel) = action.get("selector").and_then(Value::as_str) {
                args.push(sel.to_string());
            }
            if let Some(t) = action.get("timeout").and_then(Value::as_i64) {
                args.push(t.to_string());
            }
        }
        "waitfornav" => {
            if let Some(t) = action.get("timeout").and_then(Value::as_i64) {
                args.push(t.to_string());
            }
        }
        "eval" => {
            if let Some(expr) = action.get("expression").and_then(Value::as_str) {
                args.push(expr.to_string());
            }
        }
        "select" => {
            if let Some(sel) = action.get("selector").and_then(Value::as_str) {
                args.push(sel.to_string());
            }
            if let Some(vals) = action.get("values").and_then(Value::as_array) {
                for v in vals {
                    if let Some(s) = v.as_str() {
                        args.push(s.to_string());
                    }
                }
            }
        }
        "fill" => {
            if let Some(fields) = action.get("fields").and_then(Value::as_object) {
                for (selector, value) in fields {
                    args.push(selector.clone());
                    args.push(value.as_str().unwrap_or_default().to_string());
                }
            }
        }
        "read" | "text" | "dom" => {
            if let Some(sel) = action.get("selector").and_then(Value::as_str) {
                if !sel.is_empty() {
                    args.push(sel.to_string());
                }
            }
            if let Some(tokens) = action.get("max_tokens").and_then(Value::as_i64) {
                if tokens > 0 {
                    args.push(format!("--tokens={tokens}"));
                }
            }
        }
        "axtree" => {
            if action
                .get("interactive")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("-i".to_string());
            }
            if action.get("diff").and_then(Value::as_bool).unwrap_or(false) {
                args.push("--diff".to_string());
            }
        }
        // No-arg commands: observe, tabs, close, back, forward, reload, etc.
        _ => {}
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_action_options_defaults_to_non_retrying_required_steps() {
        let options = parse_action_options(&json!({"tool": "click"}));
        assert_eq!(options.wait, None);
        assert_eq!(options.retries, 0);
        assert_eq!(options.retry_delay, 0);
        assert!(!options.optional);
    }

    #[test]
    fn parse_action_options_reads_retry_fields() {
        let options = parse_action_options(&json!({
            "tool": "click",
            "wait": 750,
            "retries": 2,
            "retry_delay": 300,
            "optional": true
        }));
        assert_eq!(options.wait, Some(750));
        assert_eq!(options.retries, 2);
        assert_eq!(options.retry_delay, 300);
        assert!(options.optional);
    }

    #[test]
    fn compute_wait_ms_prefers_explicit_wait() {
        assert_eq!(compute_wait_ms("click", true, Some(900), 200), 900);
    }

    #[test]
    fn compute_wait_ms_only_applies_smart_wait_on_success() {
        assert_eq!(compute_wait_ms("click", true, None, 150), 650);
        assert_eq!(compute_wait_ms("click", false, None, 150), 150);
        assert_eq!(compute_wait_ms("read", true, None, 150), 150);
    }
}
