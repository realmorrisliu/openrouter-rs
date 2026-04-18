//! # Typed Tool Agent
//!
//! A practical tool-calling agent loop built with `typed_tool::<T>()`,
//! typed argument parsing, and explicit tool execution.
//!
//! ## Usage
//!
//! ```bash
//! export OPENROUTER_API_KEY=sk-or-v1-...
//! cargo run --example typed_tool_agent
//! ```

use std::{env, error::Error};

use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{
        Role, ToolCall,
        typed_tool::{TypedTool, TypedToolParams},
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

const MAX_AGENT_STEPS: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Environment {
    Production,
    Staging,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct DeploymentStatusParams {
    service: String,
    environment: Environment,
}

impl TypedTool for DeploymentStatusParams {
    fn name() -> &'static str {
        "get_deployment_status"
    }

    fn description() -> &'static str {
        "Look up the current deployment status for a service and environment"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct RunbookLookupParams {
    service: String,
    symptom: String,
}

impl TypedTool for RunbookLookupParams {
    fn name() -> &'static str {
        "lookup_runbook"
    }

    fn description() -> &'static str {
        "Fetch the most relevant incident runbook for a service symptom"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY environment variable not set");
    let model = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs typed tool agent example")
        .build()?;

    let mut messages = vec![
        Message::new(
            Role::System,
            "You are an operations agent for a Rust service team. Use tools before giving guidance when deployment state or runbook context is missing.",
        ),
        Message::new(
            Role::User,
            "Checkout is showing elevated 5xx errors in production. Check deployment status, pull the most relevant runbook, then tell me the next action.",
        ),
    ];

    for step in 1..=MAX_AGENT_STEPS {
        let request = ChatCompletionRequest::builder()
            .model(model.clone())
            .messages(messages.clone())
            .typed_tool::<DeploymentStatusParams>()
            .typed_tool::<RunbookLookupParams>()
            .tool_choice_auto()
            .parallel_tool_calls(false)
            .max_tokens(700)
            .build()?;

        let response = client.chat().create(&request).await?;
        let Some(choice) = response.choices.first() else {
            println!("OpenRouter returned no choices");
            return Ok(());
        };

        if let Some(tool_calls) = choice.tool_calls() {
            println!("step {step}: executing {} tool call(s)", tool_calls.len());
            messages.push(Message::assistant_with_tool_calls(
                choice.content().unwrap_or(""),
                tool_calls.to_vec(),
            ));

            for tool_call in tool_calls {
                let tool_result = execute_tool_call(tool_call)?;
                println!("  {} -> {}", tool_call.name(), tool_result);
                messages.push(Message::tool_response_named(
                    tool_call.id(),
                    tool_call.name(),
                    tool_result,
                ));
            }

            continue;
        }

        println!("final answer:\n");
        println!("{}", choice.content().unwrap_or("(empty response)"));
        return Ok(());
    }

    println!("agent stopped after reaching the configured step limit");
    Ok(())
}

fn execute_tool_call(tool_call: &ToolCall) -> Result<String, Box<dyn Error>> {
    if tool_call.is_tool::<DeploymentStatusParams>() {
        let params = DeploymentStatusParams::from_json_value(serde_json::from_str(
            tool_call.arguments_json(),
        )?)?;

        let status = json!({
            "service": params.service,
            "environment": params.environment,
            "latest_release": "checkout-2026-04-17.2",
            "rolled_out_by": "deploy-bot",
            "health": "degraded",
            "notes": [
                "error rate increased 6 minutes after the latest rollout",
                "one canary node is above the 5xx alert threshold"
            ]
        });

        return Ok(serde_json::to_string_pretty(&status)?);
    }

    if tool_call.is_tool::<RunbookLookupParams>() {
        let params = tool_call.parse_params::<RunbookLookupParams>()?;

        let runbook = json!({
            "service": params.service,
            "symptom": params.symptom,
            "runbook_id": "rb-checkout-rollback",
            "summary": "If 5xx rate spikes immediately after a rollout, stop further rollout, compare config drift, and prepare rollback.",
            "first_steps": [
                "pause the current deployment wave",
                "compare environment variables and feature flags against the last healthy release",
                "rollback if the error rate keeps climbing for more than 5 minutes"
            ]
        });

        return Ok(serde_json::to_string_pretty(&runbook)?);
    }

    Ok(format!(
        "{{\"warning\":\"unhandled tool: {}\"}}",
        tool_call.name()
    ))
}
