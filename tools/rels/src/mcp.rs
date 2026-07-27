//! `rels mcp serve` — stdio MCP server exposing the bazel-registry
//! + sibling rules_* repos as semantic tools for AI clients.
//!
//! Protocol: JSON-RPC 2.0 over line-delimited stdin/stdout. Each
//! request is one JSON object per line; responses likewise. Logs
//! go to stderr only — never stdout, which is reserved for the
//! protocol.
//!
//! Tools exposed in v0.1:
//!
//!   * `list_modules`   — every registered module + version list
//!     + maintainer + homepage.
//!
//!   * `get_changelog`  — args: `{module: string}` → contents of
//!     `<workspaces_root>/<module>/CHANGELOG.md`.
//!
//!   * `get_stardoc`    — args: `{module: string, file?: string}`.
//!     With `file`: returns that markdown file under `docs/`.
//!     Without: returns the list of available doc files.
//!
//!   * `search_symbols` — args: `{query: string, module?: string}`.
//!     greps the named module (or every module, if omitted) for
//!     Starlark rule / macro / provider definitions matching
//!     `query`. Returns up to 50 matches.
//!
//! No `initialize`-time capability negotiation is required beyond
//! `tools = {}`. Sampling, resources, prompts are deliberately
//! out-of-scope for v0.1.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::common::{Env, RegistryModule};

/// Hardcoded protocol version we speak. Mirror the MCP spec.
const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Subcommand, Debug)]
pub enum McpCommand {
    /// Start the stdio MCP server (blocks until stdin closes).
    Serve,
}

pub fn run(env: &Env, cmd: McpCommand) -> Result<()> {
    match cmd {
        McpCommand::Serve => serve(env),
    }
}

fn serve(env: &Env) -> Result<()> {
    eprintln!(
        "rels mcp: serving on stdio (registry={}, workspaces={})",
        env.registry_root.display(),
        env.workspaces_root.display(),
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut initialized = false;

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("rels mcp: malformed JSON-RPC on stdin: {} ({})", e, line);
                // No id available → send a parse-error notification.
                send_error(&mut stdout, None, -32700, &format!("parse error: {}", e))?;
                continue;
            }
        };

        // Notifications (no `id`) — just process side effects.
        if req.id.is_none() {
            if req.method == "notifications/initialized" {
                initialized = true;
                eprintln!("rels mcp: client initialized");
            }
            continue;
        }

        let id = req.id.clone().unwrap();
        let response = match req.method.as_str() {
            "initialize" => handle_initialize(),
            "tools/list" => handle_tools_list(),
            "tools/call" => handle_tools_call(env, &req, initialized),
            "ping" => Ok(json!({})),
            other => Err(McpError::method_not_found(other)),
        };

        match response {
            Ok(result) => send_result(&mut stdout, id, result)?,
            Err(err) => send_error(&mut stdout, Some(id), err.code, &err.message)?,
        }
    }

    eprintln!("rels mcp: stdin closed; exiting");
    Ok(())
}

// -----------------------------------------------------------------------------
// JSON-RPC 2.0 envelopes
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)] // `jsonrpc` field validated implicitly by being a 2.0 spec impl
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

fn send_result(stdout: &mut io::StdoutLock<'_>, id: Value, result: Value) -> Result<()> {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    };
    let line = serde_json::to_string(&resp)?;
    writeln!(stdout, "{}", line)?;
    stdout.flush()?;
    Ok(())
}

fn send_error(
    stdout: &mut io::StdoutLock<'_>,
    id: Option<Value>,
    code: i32,
    message: &str,
) -> Result<()> {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0",
        id: id.unwrap_or(Value::Null),
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
        }),
    };
    let line = serde_json::to_string(&resp)?;
    writeln!(stdout, "{}", line)?;
    stdout.flush()?;
    Ok(())
}

struct McpError {
    code: i32,
    message: String,
}

impl McpError {
    fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("method not found: {}", method),
        }
    }
    fn invalid_params(detail: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: detail.into(),
        }
    }
    fn internal(detail: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: detail.into(),
        }
    }
}

// -----------------------------------------------------------------------------
// initialize / tools/list / tools/call
// -----------------------------------------------------------------------------

fn handle_initialize() -> Result<Value, McpError> {
    Ok(json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "rels",
            "version": env!("CARGO_PKG_VERSION"),
        }
    }))
}

fn handle_tools_list() -> Result<Value, McpError> {
    Ok(json!({ "tools": tool_descriptors() }))
}

fn handle_tools_call(env: &Env, req: &JsonRpcRequest, initialized: bool) -> Result<Value, McpError> {
    if !initialized {
        // Soft warning — some clients call tools before notifying.
        eprintln!("rels mcp: tools/call before notifications/initialized; continuing anyway");
    }
    let name = req
        .params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("missing tools/call.params.name"))?;
    let empty = json!({});
    let args = req.params.get("arguments").unwrap_or(&empty);

    let text = match name {
        "list_modules" => tool_list_modules(env)?,
        "get_changelog" => tool_get_changelog(env, args)?,
        "get_stardoc" => tool_get_stardoc(env, args)?,
        "search_symbols" => tool_search_symbols(env, args)?,
        other => {
            return Err(McpError::invalid_params(format!(
                "unknown tool: {}",
                other,
            )))
        }
    };

    Ok(json!({
        "content": [{ "type": "text", "text": text }]
    }))
}

// -----------------------------------------------------------------------------
// Tool descriptors
// -----------------------------------------------------------------------------

fn tool_descriptors() -> Value {
    json!([
        {
            "name": "list_modules",
            "description": "List every module registered in the bazel-registry. Returns name, homepage, maintainer, and the full ordered version list per module.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "get_changelog",
            "description": "Return the CHANGELOG.md contents for one registered module (read from the sibling rules_* checkout).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module": {
                        "type": "string",
                        "description": "Module name, e.g. 'rules_jsonschema'."
                    }
                },
                "required": ["module"]
            }
        },
        {
            "name": "get_stardoc",
            "description": "Return a stardoc-rendered markdown reference file under <module>/docs/. With `file`, returns that file. Without, lists available files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "module": {
                        "type": "string",
                        "description": "Module name, e.g. 'rules_uv'."
                    },
                    "file": {
                        "type": "string",
                        "description": "Optional file name under docs/ (e.g. 'uv_defs.md'). Omit to list available files."
                    }
                },
                "required": ["module"]
            }
        },
        {
            "name": "search_symbols",
            "description": "Grep .bzl files for rule, macro, or provider definitions matching `query`. Returns up to 50 matches with file path + line.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Substring to look for (case-sensitive). Typically a rule or macro name."
                    },
                    "module": {
                        "type": "string",
                        "description": "Optional module to restrict the search to. Omit to grep every registered module."
                    }
                },
                "required": ["query"]
            }
        }
    ])
}

// -----------------------------------------------------------------------------
// Tool impls
// -----------------------------------------------------------------------------

fn tool_list_modules(env: &Env) -> Result<String, McpError> {
    let modules = RegistryModule::load_all(env)
        .map_err(|e| McpError::internal(format!("load modules: {}", e)))?;
    let mut out = String::new();
    out.push_str("# Registered modules\n\n");
    for m in &modules {
        let maintainer = m
            .metadata
            .maintainers
            .first()
            .map(|x| match (&x.github, &x.email) {
                (Some(gh), _) => format!("{} (@{})", x.name, gh),
                (None, Some(em)) => format!("{} <{}>", x.name, em),
                (None, None) => x.name.clone(),
            })
            .unwrap_or_else(|| "—".to_string());
        out.push_str(&format!(
            "## {}\n- homepage: {}\n- maintainer: {}\n- versions: {}\n\n",
            m.name,
            m.metadata.homepage,
            maintainer,
            m.metadata.versions.join(", "),
        ));
    }
    Ok(out)
}

fn tool_get_changelog(env: &Env, args: &Value) -> Result<String, McpError> {
    let module = arg_str(args, "module")?;
    let path = env.checkout_path(module).join("CHANGELOG.md");
    fs::read_to_string(&path).map_err(|e| {
        McpError::invalid_params(format!(
            "no CHANGELOG.md for {} ({}): {}",
            module,
            path.display(),
            e,
        ))
    })
}

fn tool_get_stardoc(env: &Env, args: &Value) -> Result<String, McpError> {
    let module = arg_str(args, "module")?;
    let docs_dir = env.checkout_path(module).join("docs");
    if !docs_dir.is_dir() {
        return Err(McpError::invalid_params(format!(
            "{}: no docs/ directory ({})",
            module,
            docs_dir.display(),
        )));
    }
    match args.get("file").and_then(|v| v.as_str()) {
        Some(f) => {
            let path = docs_dir.join(f);
            fs::read_to_string(&path).map_err(|e| {
                McpError::invalid_params(format!(
                    "could not read {}/{}: {}",
                    module, f, e,
                ))
            })
        }
        None => {
            let mut files = Vec::new();
            for entry in fs::read_dir(&docs_dir).map_err(|e| {
                McpError::internal(format!("read_dir {}: {}", docs_dir.display(), e))
            })? {
                let entry = entry.map_err(|e| McpError::internal(e.to_string()))?;
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("md") {
                    files.push(entry.file_name().to_string_lossy().to_string());
                }
            }
            files.sort();
            let mut out = format!("# {}/docs/ files\n\n", module);
            for f in files {
                out.push_str(&format!("- {}\n", f));
            }
            Ok(out)
        }
    }
}

fn tool_search_symbols(env: &Env, args: &Value) -> Result<String, McpError> {
    let query = arg_str(args, "query")?;
    let module = args.get("module").and_then(|v| v.as_str());

    let modules: Vec<RegistryModule> = match module {
        Some(name) => {
            RegistryModule::load_all(env)
                .map_err(|e| McpError::internal(format!("load modules: {}", e)))?
                .into_iter()
                .filter(|m| m.name == name)
                .collect()
        }
        None => RegistryModule::load_all(env)
            .map_err(|e| McpError::internal(format!("load modules: {}", e)))?,
    };

    if modules.is_empty() {
        return Err(McpError::invalid_params(format!(
            "no registered module named {:?}",
            module.unwrap_or(""),
        )));
    }

    let mut matches: Vec<String> = Vec::new();
    'outer: for m in modules {
        let root = env.checkout_path(&m.name);
        if !root.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // Skip bazel-* output trees + .git.
            let p = entry.path();
            if p.components().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s.starts_with("bazel-") || s == ".git"
            }) {
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("bzl") {
                continue;
            }
            let text = match fs::read_to_string(p) {
                Ok(t) => t,
                Err(_) => continue,
            };
            for (lineno, line) in text.lines().enumerate() {
                if line.contains(query) && is_definition_line(line) {
                    matches.push(format!(
                        "{}/{}:{}: {}",
                        m.name,
                        relative_to(p, &root),
                        lineno + 1,
                        line.trim(),
                    ));
                    if matches.len() >= 50 {
                        break 'outer;
                    }
                }
            }
        }
    }

    let mut out = format!("# search_symbols({:?}) — {} matches\n\n", query, matches.len());
    for m in &matches {
        out.push_str(m);
        out.push('\n');
    }
    Ok(out)
}

fn is_definition_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("def ")
        || (t.contains(" = rule(")
            || t.contains(" = provider(")
            || t.contains(" = module_extension(")
            || t.contains(" = repository_rule(")
            || t.contains(" = tag_class("))
}

fn relative_to(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, McpError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params(format!("missing argument: {}", key)))
}
