//! The tool interface shared by the two development tracks.
//!
//! The agent layer discovers tools by [`Tool::name`], advertises them with
//! [`Tool::description`] and [`Tool::parameters_schema`], and executes the
//! model's calls through [`ToolRegistry::invoke`]. The deterministic core
//! implements the concrete tools. Both sides code against this contract, so
//! they can proceed in parallel.
//!
//! [`Tool`] is object-safe on purpose — the registry stores `Box<dyn Tool>`
//! and dispatches dynamically — so arguments and results cross the boundary
//! as [`serde_json::Value`], matching the LLM tool-call protocol.
//! Implementers with concrete types use [`TypedTool`] and get [`Tool`] via
//! the blanket impl, which handles the (de)serialisation.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Ambient state handed to every invocation: what a tool needs but should
/// not own. Tools enforce the guardrails themselves using these flags.
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Resolve relative paths against this, never the process CWD.
    pub workspace: PathBuf,
    /// Mutating tools MUST NOT write; they report intended changes instead.
    pub dry_run: bool,
    /// Confirmation gates are pre-approved (workshop mode / `--yes`).
    pub assume_yes: bool,
}

impl ToolContext {
    /// Rooted at `workspace`, both safety flags off.
    pub fn new(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: workspace.into(),
            dry_run: false,
            assume_yes: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("invalid arguments for tool `{tool}`: {source}")]
    InvalidArguments {
        tool: &'static str,
        #[source]
        source: serde_json::Error,
    },

    #[error("tool `{tool}` produced an unserialisable result: {source}")]
    InvalidResult {
        tool: &'static str,
        #[source]
        source: serde_json::Error,
    },

    #[error("tool `{tool}` failed: {message}")]
    Execution { tool: &'static str, message: String },

    /// A hard-rule guardrail (deny-list, confirmation gate, dry-run) stopped
    /// the call before it ran.
    #[error("tool `{tool}` blocked: {reason}")]
    Denied { tool: &'static str, reason: String },

    #[error("no such tool: `{0}`")]
    NotFound(String),
}

impl ToolError {
    pub fn execution(tool: &'static str, message: impl Into<String>) -> Self {
        Self::Execution {
            tool,
            message: message.into(),
        }
    }

    pub fn denied(tool: &'static str, reason: impl Into<String>) -> Self {
        Self::Denied {
            tool,
            reason: reason.into(),
        }
    }
}

/// A single capability the agent can invoke. Most implementers should prefer
/// [`TypedTool`] and rely on the blanket impl.
pub trait Tool: Send + Sync {
    /// Wire identifier: the model selects by it and the registry keys on it,
    /// so renaming one is a breaking change.
    fn name(&self) -> &'static str;

    /// One sentence; it is the model's basis for choosing this tool.
    fn description(&self) -> &'static str;

    /// JSON Schema for the accepted `args` object.
    fn parameters_schema(&self) -> Value;

    fn invoke(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError>;
}

/// [`Tool`] for implementers with concrete argument and output types.
pub trait TypedTool: Send + Sync {
    /// Wire identifier: the model selects by it and the registry keys on it,
    /// so renaming one is a breaking change.
    const NAME: &'static str;
    /// One sentence; it is the model's basis for choosing this tool.
    const DESCRIPTION: &'static str;

    type Args: DeserializeOwned;
    type Output: Serialize;

    /// JSON Schema for [`TypedTool::Args`].
    fn parameters_schema() -> Value;

    fn run(&self, ctx: &ToolContext, args: Self::Args) -> Result<Self::Output, ToolError>;
}

impl<T: TypedTool> Tool for T {
    fn name(&self) -> &'static str {
        T::NAME
    }

    fn description(&self) -> &'static str {
        T::DESCRIPTION
    }

    fn parameters_schema(&self) -> Value {
        <T as TypedTool>::parameters_schema()
    }

    fn invoke(&self, ctx: &ToolContext, args: Value) -> Result<Value, ToolError> {
        let parsed = serde_json::from_value::<T::Args>(args).map_err(|source| {
            ToolError::InvalidArguments {
                tool: T::NAME,
                source,
            }
        })?;
        let output = self.run(ctx, parsed)?;
        serde_json::to_value(output).map_err(|source| ToolError::InvalidResult {
            tool: T::NAME,
            source,
        })
    }
}

/// The set of tools the agent may call, keyed by [`Tool::name`].
#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<&'static str, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces any earlier tool registered under the same name.
    pub fn register(&mut self, tool: Box<dyn Tool>) -> &mut Self {
        self.tools.insert(tool.name(), tool);
        self
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(Box::as_ref)
    }

    /// Sorted, since the backing map is ordered.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.tools.keys().copied()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn invoke(&self, ctx: &ToolContext, name: &str, args: Value) -> Result<Value, ToolError> {
        self.get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?
            .invoke(ctx, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    /// A minimal `echo` tool used to exercise the contract from both sides.
    struct Echo;

    #[derive(Deserialize)]
    struct EchoArgs {
        message: String,
    }

    #[derive(Serialize)]
    struct EchoOut {
        echoed: String,
        dry_run: bool,
    }

    impl TypedTool for Echo {
        const NAME: &'static str = "echo";
        const DESCRIPTION: &'static str = "Echo the message back.";
        type Args = EchoArgs;
        type Output = EchoOut;

        fn parameters_schema() -> Value {
            json!({
                "type": "object",
                "properties": { "message": { "type": "string" } },
                "required": ["message"]
            })
        }

        fn run(&self, ctx: &ToolContext, args: Self::Args) -> Result<Self::Output, ToolError> {
            Ok(EchoOut {
                echoed: args.message,
                dry_run: ctx.dry_run,
            })
        }
    }

    #[test]
    fn typed_tool_adapts_to_the_boundary_trait() {
        let tool = Echo;
        assert_eq!(tool.name(), "echo");
        let ctx = ToolContext::new("/tmp/pkg");
        let out = tool.invoke(&ctx, json!({"message": "hi"})).unwrap();
        assert_eq!(out["echoed"], "hi");
        assert_eq!(out["dry_run"], false);
    }

    #[test]
    fn bad_arguments_surface_as_invalid_arguments() {
        let ctx = ToolContext::new("/tmp/pkg");
        let err = Echo.invoke(&ctx, json!({"wrong": 1})).unwrap_err();
        assert!(matches!(
            err,
            ToolError::InvalidArguments { tool: "echo", .. }
        ));
    }

    #[test]
    fn registry_dispatches_and_reports_unknown_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(Echo));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.names().collect::<Vec<_>>(), vec!["echo"]);

        let ctx = ToolContext::new("/tmp/pkg");
        let out = registry
            .invoke(&ctx, "echo", json!({"message": "yo"}))
            .unwrap();
        assert_eq!(out["echoed"], "yo");

        let err = registry.invoke(&ctx, "missing", json!({})).unwrap_err();
        assert!(matches!(err, ToolError::NotFound(name) if name == "missing"));
    }
}
