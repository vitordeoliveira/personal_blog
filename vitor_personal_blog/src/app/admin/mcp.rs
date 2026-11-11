//! The owner's MCP server, at `/admin/mcp`, through which AI agents, such as the
//! marketing ones, read how the blog does: its visits, each post's readers, and the
//! posts themselves. It takes the token oxploy gives the site. In oxploy, the app's MCP
//! tab turns it on at this path, and Caddy then hides the path from the site's domains,
//! so only oxploy reaches it.
//!
//! It speaks the Model Context Protocol over Streamable HTTP in its simplest form: each
//! POST carries one JSON-RPC message and gets plain JSON back, with no session and no
//! event stream, which is all a server with only tools needs. Each tool is a file of its
//! own in `src/app/admin/mcp/`, with a line in `definitions` and in `call_tool`.

mod common;
mod post_performance;
mod read_post;
mod site_visits;

use std::time::Duration;

use serde_json::{Value, json};
use topcoat::{
    Result,
    context::Cx,
    router::{
        HeaderValue, StatusCode,
        content::Json,
        header::WWW_AUTHENTICATE,
        request::Bytes,
        response::{IntoResponse, Response},
        route,
    },
};

use self::common::ToolError;
use crate::features::mcp::is_authorized;

/// How long a request without the right token waits before the answer, which slows
/// guessing.
const WRONG_TOKEN_WAIT: Duration = Duration::from_secs(1);

/// The protocol versions the server speaks, newest first. The few methods it answers
/// read the same in all of them.
const PROTOCOL_VERSIONS: [&str; 3] = ["2025-11-25", "2025-06-18", "2025-03-26"];

/// JSON-RPC's error codes.
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;

/// What the server tells an agent about itself when it connects.
const INSTRUCTIONS: &str = "vitor.ws is the blog of Vitor de Oliveira, a software \
    engineer at RockawayX who writes about Rust, crypto and AI. Its success is measured \
    by its visits, which the site counts itself. site_visits reports the visits of a \
    period, next to the period before it, with where visitors came from; \
    post_performance how each published post did in a period: how many read it, how \
    many it brought to the site, from where and from which campaign; and read_post a \
    post's text, to write about it. A campaign is the utm_source of the link a visitor \
    followed, so a link shared with one, such as https://vitor.ws/blog/<slug>?utm_source=linkedin, \
    tells which share brought readers. A visitor is a person on one day.";

/// Receives one MCP message from the owner, and answers it.
#[route(POST)]
pub async fn receive(cx: &Cx, body: Bytes) -> Result<Response> {
    if !is_authorized(cx) {
        tokio::time::sleep(WRONG_TOKEN_WAIT).await;
        return (
            StatusCode::UNAUTHORIZED,
            [(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"))],
            "Unauthorized",
        )
            .into_response(cx);
    }
    let answer = match serde_json::from_slice(&body) {
        Ok(message) => answer(cx, &message).await,
        Err(_) => Some(failure(Value::Null, PARSE_ERROR, "The body is not JSON.")),
    };
    match answer {
        Some(answer) => Json(answer).into_response(cx),
        // A notification gets no answer, only the acknowledgement the protocol asks for.
        None => StatusCode::ACCEPTED.into_response(cx),
    }
}

/// A JSON-RPC error: its code and what went wrong.
#[derive(Debug)]
struct Failure {
    code: i64,
    message: String,
}

impl Failure {
    fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// The answer to one JSON-RPC message, or `None` for a notification, such as
/// `notifications/initialized`, which gets none.
async fn answer(cx: &Cx, message: &Value) -> Option<Value> {
    let id = message.get("id").cloned();
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        let id = id.unwrap_or(Value::Null);
        return Some(failure(
            id,
            INVALID_REQUEST,
            "Expected one JSON-RPC request.",
        ));
    };
    let id = id?;
    let params = message.get("params").unwrap_or(&Value::Null);
    let outcome = match method {
        "initialize" => Ok(initialize(params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": definitions() })),
        "tools/call" => call_tool(cx, params).await,
        _ => Err(Failure::new(
            METHOD_NOT_FOUND,
            format!("Unknown method: {method}."),
        )),
    };
    Some(match outcome {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => failure(id, error.code, &error.message),
    })
}

/// The answer to `initialize`: the protocol version, the client's when the server
/// speaks it or else the newest, and what the server offers.
fn initialize(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let version = requested
        .filter(|version| PROTOCOL_VERSIONS.contains(version))
        .unwrap_or(PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "vitor-ws",
            "title": "vitor.ws",
            "version": super::super::VERSION,
            "websiteUrl": "https://vitor.ws",
        },
        "instructions": INSTRUCTIONS,
    })
}

/// The name, description and arguments of every tool, for `tools/list`.
fn definitions() -> Vec<Value> {
    vec![
        site_visits::definition(),
        post_performance::definition(),
        read_post::definition(),
    ]
}

/// Runs the tool `params` names. A call the tool refuses comes back as the tool's own
/// error, which the agent reads and can correct; an unknown tool, or a failure of the
/// database or the posts' files, is an error of the protocol.
async fn call_tool(cx: &Cx, params: &Value) -> Result<Value, Failure> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let outcome = match name {
        site_visits::NAME => site_visits::call(cx, arguments).await,
        post_performance::NAME => post_performance::call(cx, arguments).await,
        read_post::NAME => read_post::call(cx, arguments).await,
        _ => {
            return Err(Failure::new(
                INVALID_PARAMS,
                format!("Unknown tool: {name}."),
            ));
        }
    };
    match outcome {
        Ok(data) => Ok(tool_result(data.to_string(), false)),
        Err(ToolError::Refused(why)) => Ok(tool_result(why, true)),
        Err(ToolError::Failed(error)) => {
            eprintln!("vitor_personal_blog: the MCP tool {name} failed: {error}");
            Err(Failure::new(
                INTERNAL_ERROR,
                "The tool failed. Try again later.",
            ))
        }
    }
}

/// A tool's result: its text, and whether it is an error.
fn tool_result(text: String, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
}

/// A JSON-RPC error response.
fn failure(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use topcoat::context::CxTestBuilder;

    use super::*;

    async fn send(message: Value) -> Option<Value> {
        answer(&CxTestBuilder::new().build(), &message).await
    }

    async fn call(name: &str, arguments: Value) -> Value {
        let message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        });
        send(message).await.unwrap()
    }

    #[tokio::test]
    async fn speaks_the_clients_version_or_its_newest() {
        let initialize = |version: &str| {
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": version },
            })
        };
        let answer = send(initialize("2025-06-18")).await.unwrap();
        assert_eq!(answer["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(answer["id"], 1);
        let answer = send(initialize("2030-01-01")).await.unwrap();
        assert_eq!(answer["result"]["protocolVersion"], PROTOCOL_VERSIONS[0]);
        let instructions = answer["result"]["instructions"].as_str().unwrap();
        for tool in ["site_visits", "post_performance", "read_post"] {
            assert!(instructions.contains(tool), "{tool}");
        }
    }

    #[tokio::test]
    async fn lists_its_tools_as_read_only() {
        let answer = send(json!({ "jsonrpc": "2.0", "id": "a", "method": "tools/list" }))
            .await
            .unwrap();
        assert_eq!(answer["id"], "a");
        let tools = answer["result"]["tools"].as_array().unwrap();
        for tool in tools {
            assert_eq!(tool["inputSchema"]["type"], "object", "{}", tool["name"]);
            assert_eq!(
                tool["annotations"]["readOnlyHint"], true,
                "{}",
                tool["name"]
            );
        }
        let names: Vec<_> = tools
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["site_visits", "post_performance", "read_post"]);
    }

    #[tokio::test]
    async fn answers_no_notification() {
        let message = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert_eq!(send(message).await, None);
    }

    #[tokio::test]
    async fn a_refused_call_is_the_tools_own_error() {
        for (name, arguments, why) in [
            (site_visits::NAME, json!({ "days": 401 }), "days"),
            (site_visits::NAME, json!({ "dayz": 7 }), "unknown field"),
            (
                post_performance::NAME,
                json!({ "days": 7, "from": "2026-11-01" }),
                "not both",
            ),
            (
                post_performance::NAME,
                json!({ "from": "yesterday" }),
                "such as",
            ),
            (read_post::NAME, json!({}), "missing field"),
            (
                read_post::NAME,
                json!({ "slug": "../secrets" }),
                "post_performance",
            ),
        ] {
            let answer = call(name, arguments).await;
            assert_eq!(answer["result"]["isError"], true, "{answer}");
            let text = answer["result"]["content"][0]["text"].as_str().unwrap();
            assert!(text.contains(why), "{name}: {text}");
        }
    }

    #[tokio::test]
    async fn refuses_what_it_does_not_know() {
        let unknown_method = send(json!({ "jsonrpc": "2.0", "id": 1, "method": "resources/list" }))
            .await
            .unwrap();
        assert_eq!(unknown_method["error"]["code"], METHOD_NOT_FOUND);

        let unknown_tool = call("delete_visits", json!({})).await;
        assert_eq!(unknown_tool["error"]["code"], INVALID_PARAMS);

        let batch = send(json!([{ "jsonrpc": "2.0", "id": 3, "method": "ping" }]))
            .await
            .unwrap();
        assert_eq!(batch["error"]["code"], INVALID_REQUEST);
    }
}
