//! Reusable tool-calling primitives for the AI agent loops.
//!
//! The wire client ([`client`]) speaks the DeepSeek `/chat/completions`
//! tool-calling protocol. On top of it, a tool is described by a [`ToolSpec`]:
//! its model-facing definition (name, description, JSON-Schema parameters)
//! paired with the function that runs it. A [`ToolRegistry`] groups specs so an
//! agent loop can hand the model the definitions and dispatch a tool call by
//! name without a hand-maintained match arm per tool.
//!
//! Specs are generic over a context type `Ctx` — whatever a handler needs to do
//! its work (a checked-out workspace, an HTTP client, PR identifiers, ...). Each
//! consumer defines its own `Ctx` and its own set of specs; the dispatch
//! machinery is shared.

mod client;

pub use client::{Message, Tool, ToolFunction, chat};

use std::{future::Future, pin::Pin};

use serde_json::Value;

/// The future a tool handler returns: the tool's textual result, which the
/// agent loop relays back to the model as a `tool` message. Errors are carried
/// as strings (not `Result`) so the model can read and react to them.
pub type ToolFuture<'a> = Pin<Box<dyn Future<Output = String> + Send + 'a>>;

/// A tool the model can call: its wire definition plus the function that runs
/// it. `handler` borrows the shared `Ctx` for the duration of the call.
///
/// Handlers are plain function pointers rather than trait objects: a tool is
/// data + a fn, so there's nothing to gain from a `dyn Trait` and a fn pointer
/// sidesteps the boxing dance that `async fn` in a trait would need.
pub struct ToolSpec<Ctx> {
    /// Tool name, as the model calls it.
    pub name: &'static str,
    /// One-line description the model sees.
    pub description: &'static str,
    /// JSON-Schema for the tool's arguments.
    pub parameters: Value,
    /// Runs the tool against `Ctx` with the model-supplied arguments.
    pub handler: for<'a> fn(&'a Ctx, Value) -> ToolFuture<'a>,
}

/// A set of [`ToolSpec`]s sharing a context type, with dispatch by name.
pub struct ToolRegistry<Ctx> {
    specs: Vec<ToolSpec<Ctx>>,
}

impl<Ctx> ToolRegistry<Ctx> {
    pub fn new(specs: Vec<ToolSpec<Ctx>>) -> Self {
        Self { specs }
    }

    /// The wire-format tool definitions to send to the model.
    pub fn definitions(&self) -> Vec<Tool> {
        self.specs
            .iter()
            .map(|s| Tool {
                tool_type: "function".into(),
                function: ToolFunction {
                    name: s.name.into(),
                    description: s.description.into(),
                    parameters: s.parameters.clone(),
                },
            })
            .collect()
    }

    /// Run the named tool. An unknown name returns an error string (rather than
    /// failing the loop) so the model can recover — the same convention the
    /// individual handlers use for their own errors.
    pub async fn dispatch(&self, ctx: &Ctx, name: &str, args: Value) -> String {
        match self.specs.iter().find(|s| s.name == name) {
            Some(spec) => (spec.handler)(ctx, args).await,
            None => format!("unknown tool: {name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Ctx {
        greeting: &'static str,
    }

    fn echo<'a>(ctx: &'a Ctx, args: Value) -> ToolFuture<'a> {
        Box::pin(async move {
            let who = args.get("who").and_then(Value::as_str).unwrap_or("world");
            format!("{}, {who}", ctx.greeting)
        })
    }

    fn registry() -> ToolRegistry<Ctx> {
        ToolRegistry::new(vec![ToolSpec {
            name: "echo",
            description: "Echo a greeting.",
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "who": { "type": "string" } },
                "additionalProperties": false
            }),
            handler: echo,
        }])
    }

    #[test]
    fn definitions_expose_each_spec() {
        let defs = registry().definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].tool_type, "function");
        assert_eq!(defs[0].function.name, "echo");
        assert_eq!(defs[0].function.description, "Echo a greeting.");
    }

    #[tokio::test]
    async fn dispatch_runs_the_named_handler() {
        let ctx = Ctx { greeting: "hi" };
        let out = registry()
            .dispatch(&ctx, "echo", serde_json::json!({"who": "Hu Tao"}))
            .await;
        assert_eq!(out, "hi, Hu Tao");
    }

    #[tokio::test]
    async fn dispatch_unknown_tool_returns_error_string() {
        let ctx = Ctx { greeting: "hi" };
        let out = registry()
            .dispatch(&ctx, "frobnicate", serde_json::json!({}))
            .await;
        assert_eq!(out, "unknown tool: frobnicate");
    }
}
