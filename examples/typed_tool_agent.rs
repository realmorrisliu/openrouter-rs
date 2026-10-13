//! # Typed Tool Agent
//!
//! A practical tool-calling agent loop built with `typed_tool::<T>()`,
//! typed argument parsing, and explicit tool execution. The tools return fixed
//! demonstration fixtures: no live infrastructure is queried or changed.
//! stdout is JSON lines; errors and exhausted limits exit unsuccessfully.
//! OPENROUTER_MODEL overrides the default model. OPENROUTER_REASONING accepts
//! a JSON ReasoningConfig; an explicit configuration wins over the demo default.
//!
//! ## Usage
//!
//! ```bash
//! export OPENROUTER_API_KEY=sk-or-v1-...
//! cargo run --example typed_tool_agent
//! ```

use std::{collections::HashSet, env, error::Error, time::Duration};

use openrouter_rs::{
    OpenRouterClient,
    api::chat::{ChatCompletionRequest, Message},
    types::{
        Effort, ReasoningConfig, Role, ToolCall, completion::FinishReason, typed_tool::TypedTool,
    },
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use tokio::time::{Instant, timeout_at};

const MAX_AGENT_STEPS: usize = 4;
const TUTORIAL_MODEL: &str = "deepseek/deepseek-v4.1-flash";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Environment {
    Production,
    Staging,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DeploymentStatusParams {
    service: String,
    environment: Environment,
}

impl TypedTool for DeploymentStatusParams {
    fn name() -> &'static str {
        "get_deployment_status"
    }

    fn description() -> &'static str {
        "Read a fixed demonstration deployment snapshot for the checkout service; no real infrastructure is queried"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RunbookLookupParams {
    service: String,
    symptom: String,
}

impl TypedTool for RunbookLookupParams {
    fn name() -> &'static str {
        "lookup_runbook"
    }

    fn description() -> &'static str {
        "Read a fixed demonstration runbook for the checkout service; no actions are executed"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key =
        env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY environment variable not set");
    let model = env::var("OPENROUTER_MODEL").unwrap_or_else(|_| TUTORIAL_MODEL.to_string());
    let reasoning: Option<ReasoningConfig> = match env::var("OPENROUTER_REASONING") {
        Ok(value) => Some(serde_json::from_str(&value)?),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };
    let http_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    let client = OpenRouterClient::builder()
        .api_key(api_key)
        .http_client(http_client)
        .http_referer("https://github.com/realmorrisliu/openrouter-rs")
        .x_title("openrouter-rs typed tool agent example")
        .build()?;

    run_agent(&client, &model, reasoning, Duration::from_secs(60)).await
}

async fn run_agent(
    client: &OpenRouterClient,
    model: &str,
    reasoning: Option<ReasoningConfig>,
    timeout: Duration,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + timeout;
    let mut call_ids = HashSet::new();
    let mut messages = vec![
        Message::new(
            Role::System,
            "You are an operations agent for a Rust service team. This is a simulated incident: both tools return fixed demo data. Call both tools before giving a concise recommendation. Never claim to have changed infrastructure.",
        ),
        Message::new(
            Role::User,
            "Checkout is showing elevated 5xx errors in production. Check deployment status, pull the most relevant runbook, then tell me the next action.",
        ),
    ];

    for step in 1..=MAX_AGENT_STEPS {
        let mut builder = ChatCompletionRequest::builder();
        if let Some(config) = &reasoning {
            builder.reasoning(config.clone());
        } else if model == TUTORIAL_MODEL {
            builder.reasoning_effort(Effort::None);
        }
        let request = builder
            .model(model)
            .messages(messages.clone())
            .typed_tool::<DeploymentStatusParams>()
            .typed_tool::<RunbookLookupParams>()
            .tool_choice_auto()
            .parallel_tool_calls(false)
            .max_tokens(2048)
            .build()?;

        let response = timeout_at(deadline, client.chat().create(&request))
            .await
            .map_err(|_| "Agent exceeded its total deadline")??;
        // JSON lines retain real response/usage evidence without logging credentials.
        println!(
            "{}",
            json!({"event":"completion", "step":step, "response":response})
        );
        let choice = response
            .choices
            .first()
            .ok_or("OpenRouter returned no choices")?;
        if choice.error().is_some()
            || !matches!(
                choice.finish_reason(),
                Some(FinishReason::Stop | FinishReason::ToolCalls)
            )
        {
            return Err("Incomplete or failed completion".into());
        }

        let message = choice
            .message()
            .ok_or("Expected a chat assistant message")?;
        if message.role.as_deref() != Some("assistant") {
            return Err("Expected an assistant message".into());
        }

        if let Some(tool_calls) = choice.tool_calls().filter(|calls| !calls.is_empty()) {
            if tool_calls.len() > 2 {
                return Err("Too many tool calls in one response".into());
            }
            messages.push(Message::try_from(message)?);
            for tool_call in tool_calls {
                if tool_call.id().is_empty() || !call_ids.insert(tool_call.id().to_string()) {
                    return Err("Missing or repeated tool call ID".into());
                }
                let result = execute_tool_call(tool_call);
                println!(
                    "{}",
                    json!({"event":"tool_result", "step":step,
                    "tool_call_id":tool_call.id(), "name":tool_call.name(), "result":result})
                );
                messages.push(Message::tool_response_named(
                    tool_call.id(),
                    tool_call.name(),
                    serde_json::to_string(&result)?,
                ));
            }
            continue;
        }

        if !matches!(choice.finish_reason(), Some(FinishReason::Stop)) {
            return Err("Tool-call completion contained no tool calls".into());
        }
        let content = choice
            .content()
            .filter(|text| !text.trim().is_empty())
            .ok_or("OpenRouter returned an empty final answer")?;
        println!(
            "{}",
            json!({"event":"final", "step":step, "content":content})
        );
        return Ok(());
    }
    Err("Agent reached the step limit without a final answer".into())
}

fn execute_tool_call(tool_call: &ToolCall) -> serde_json::Value {
    if tool_call.arguments_json().len() > 8192 {
        return json!({"error":"arguments_too_large"});
    }
    if tool_call.is_tool::<DeploymentStatusParams>() {
        let Ok(params) = tool_call.parse_params::<DeploymentStatusParams>() else {
            return json!({"error":"invalid_arguments"});
        };
        if params.service != "checkout" {
            return json!({"error":"unknown_service", "supported_service":"checkout"});
        }
        return json!({
            "demo": true,
            "service": params.service,
            "environment": params.environment,
            "latest_release": "checkout-demo.2",
            "health": match params.environment {
                Environment::Production => "degraded",
                Environment::Staging => "healthy",
            },
            "notes": ["Demo snapshot: production 5xx increased after the last rollout"]
        });
    }
    if tool_call.is_tool::<RunbookLookupParams>() {
        let Ok(params) = tool_call.parse_params::<RunbookLookupParams>() else {
            return json!({"error":"invalid_arguments"});
        };
        if params.service != "checkout" {
            return json!({"error":"unknown_service", "supported_service":"checkout"});
        }
        if params.symptom.trim().is_empty() || params.symptom.len() > 512 {
            return json!({"error":"invalid_symptom"});
        }
        return json!({
            "demo": true,
            "service": params.service,
            "symptom": params.symptom,
            "runbook_id": "rb-checkout-rollback",
            "first_steps": [
                "pause the current deployment wave",
                "compare configuration with the last healthy release",
                "consider rollback if the error rate remains high"
            ]
        });
    }
    json!({"error":"unknown_tool", "tool":tool_call.name()})
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Json, Router, routing::post};
    use serde_json::{Value, json};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::{net::TcpListener, task::JoinHandle};

    async fn mock(router: Router) -> (OpenRouterClient, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let job = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let client = OpenRouterClient::builder()
            .api_key("test-key")
            .base_url(format!("http://{address}"))
            .build()
            .unwrap();
        (client, job)
    }

    fn reply(content: Value, calls: Value, finish: &str) -> Value {
        json!({"id":"mock", "object":"chat.completion", "created":1, "model":"mock",
            "choices":[{"index":0,"message":{"role":"assistant","content":content,
                "tool_calls":calls,"reasoning_details":[{"type":"reasoning.encrypted","data":"opaque", "signature":"signature"}]},"finish_reason":finish}]})
    }

    fn call(id: &str, name: &str, arguments: &str) -> Value {
        serde_json::to_value(ToolCall::new(id, name, arguments)).unwrap()
    }

    #[test]
    fn dispatch_checks_arguments_and_keeps_errors_valid_json() {
        for args in [
            "{",
            r#"{"service":"checkout"}"#,
            r#"{"service":"checkout","environment":"prod"}"#,
            r#"{"service":"checkout","environment":"production","extra":true}"#,
        ] {
            assert_eq!(
                execute_tool_call(&ToolCall::new("x", DeploymentStatusParams::name(), args))["error"],
                "invalid_arguments"
            );
        }
        assert_eq!(
            execute_tool_call(&ToolCall::new("x", "unknown\"tool", "{}")),
            json!({"error":"unknown_tool", "tool":"unknown\"tool"})
        );
        assert_eq!(
            execute_tool_call(&ToolCall::new(
                "x",
                RunbookLookupParams::name(),
                r#"{"service":"checkout","symptom":" "}"#
            ))["error"],
            "invalid_symptom"
        );
        assert_eq!(
            execute_tool_call(&ToolCall::new(
                "x",
                DeploymentStatusParams::name(),
                r#"{"service":"other","environment":"production"}"#
            ))["error"],
            "unknown_service"
        );
        assert_eq!(
            execute_tool_call(&ToolCall::new(
                "x",
                DeploymentStatusParams::name(),
                &"x".repeat(8193)
            ))["error"],
            "arguments_too_large"
        );
        let value = execute_tool_call(&ToolCall::new(
            "x",
            DeploymentStatusParams::name(),
            r#"{"service":"checkout","environment":"staging"}"#,
        ));
        assert_eq!(value["health"], "healthy");
        assert_eq!(value["demo"], true);
    }

    #[tokio::test]
    async fn round_trip_preserves_assistant_and_correlated_results() {
        let requests = Arc::new(AtomicUsize::new(0));
        let observed = requests.clone();
        let (client, job) = mock(Router::new().route(
            "/chat/completions",
            post(move |Json(body): Json<Value>| {
                assert_eq!(body["tools"].as_array().unwrap().len(), 2);
                assert_eq!(body["reasoning"], json!({"effort":"low"}));
                assert_eq!(body["model"], "override-model");
                let turn = observed.fetch_add(1, Ordering::SeqCst);
                async move {
                    if turn == 0 {
                        Json(reply(
                            Value::Null,
                            json!([
                                call(
                                    "status",
                                    DeploymentStatusParams::name(),
                                    r#"{"service":"checkout","environment":"production"}"#
                                ),
                                call(
                                    "runbook",
                                    RunbookLookupParams::name(),
                                    r#"{"service":"checkout","symptom":"5xx"}"#
                                )
                            ]),
                            "tool_calls",
                        ))
                    } else {
                        let messages = body["messages"].as_array().unwrap();
                        assert_eq!(messages.len(), 5);
                        assert_eq!(messages[2]["role"], "assistant");
                        assert!(messages[2]["content"].is_null());
                        assert_eq!(
                            messages[2]["reasoning_details"][0]["signature"],
                            "signature"
                        );
                        for (i, id) in [(3, "status"), (4, "runbook")] {
                            assert_eq!(messages[i]["role"], "tool");
                            assert_eq!(messages[i]["tool_call_id"], id);
                            let result: Value =
                                serde_json::from_str(messages[i]["content"].as_str().unwrap())
                                    .unwrap();
                            assert_eq!(result["demo"], true);
                        }
                        Json(reply(
                            json!("Consider the demo rollback runbook."),
                            Value::Null,
                            "stop",
                        ))
                    }
                }
            }),
        ))
        .await;
        let config = serde_json::from_value(json!({"effort":"low"})).unwrap();
        run_agent(
            &client,
            "override-model",
            Some(config),
            Duration::from_secs(2),
        )
        .await
        .unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        job.abort();
    }

    #[tokio::test]
    async fn model_policy_does_not_override_other_models_or_explicit_defaults() {
        for (model, config, expected) in [
            (TUTORIAL_MODEL, None, Some(json!({"effort":"none"}))),
            ("other-model", None, None),
            (
                TUTORIAL_MODEL,
                Some(serde_json::from_value(json!({})).unwrap()),
                Some(json!({})),
            ),
        ] {
            let (client, job) = mock(Router::new().route(
                "/chat/completions",
                post(move |Json(body): Json<Value>| {
                    assert_eq!(body.get("reasoning").cloned(), expected);
                    async { Json(reply(json!("answer"), Value::Null, "stop")) }
                }),
            ))
            .await;
            run_agent(&client, model, config, Duration::from_secs(2))
                .await
                .unwrap();
            job.abort();
        }
    }

    #[tokio::test]
    async fn incomplete_empty_and_malformed_completions_fail() {
        let mut wrong_role = reply(json!("answer"), Value::Null, "stop");
        wrong_role["choices"][0]["message"]["role"] = json!("user");
        let cases = [
            wrong_role,
            json!({"id":"mock", "object":"chat.completion", "created":1, "model":"mock", "choices":[]}),
            reply(Value::Null, Value::Null, "stop"),
            reply(json!(" "), json!([]), "stop"),
            reply(json!("partial"), Value::Null, "length"),
            reply(json!("blocked"), Value::Null, "content_filter"),
            reply(json!("failed"), Value::Null, "error"),
            reply(Value::Null, json!([]), "tool_calls"),
            reply(
                Value::Null,
                json!([call("", "unknown", "{}")]),
                "tool_calls",
            ),
            reply(
                Value::Null,
                json!([call("same", "unknown", "{}"), call("same", "unknown", "{}")]),
                "tool_calls",
            ),
            reply(
                Value::Null,
                json!([
                    call("1", "unknown", "{}"),
                    call("2", "unknown", "{}"),
                    call("3", "unknown", "{}")
                ]),
                "tool_calls",
            ),
        ];
        for body in cases {
            let (client, job) = mock(Router::new().route(
                "/chat/completions",
                post(move || {
                    let body = body.clone();
                    async { Json(body) }
                }),
            ))
            .await;
            assert!(
                run_agent(&client, TUTORIAL_MODEL, None, Duration::from_secs(2))
                    .await
                    .is_err()
            );
            job.abort();
        }
    }

    #[tokio::test]
    async fn invalid_arguments_return_as_tool_feedback() {
        let requests = Arc::new(AtomicUsize::new(0));
        let observed = requests.clone();
        let (client, job) = mock(Router::new().route(
            "/chat/completions",
            post(move |Json(body): Json<Value>| {
                let turn = observed.fetch_add(1, Ordering::SeqCst);
                async move {
                    if turn == 0 {
                        Json(reply(
                            Value::Null,
                            json!([call(
                                "bad",
                                DeploymentStatusParams::name(),
                                r#"{"environment":"prod"}"#
                            )]),
                            "tool_calls",
                        ))
                    } else {
                        let result: Value =
                            serde_json::from_str(body["messages"][3]["content"].as_str().unwrap())
                                .unwrap();
                        assert_eq!(result["error"], "invalid_arguments");
                        assert_eq!(body["messages"][3]["tool_call_id"], "bad");
                        Json(reply(
                            json!("Could not get the required context."),
                            Value::Null,
                            "stop",
                        ))
                    }
                }
            }),
        ))
        .await;
        run_agent(&client, TUTORIAL_MODEL, None, Duration::from_secs(2))
            .await
            .unwrap();
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        job.abort();
    }

    #[tokio::test]
    async fn repeated_calls_hit_the_step_limit() {
        let requests = Arc::new(AtomicUsize::new(0));
        let observed = requests.clone();
        let (client, job) = mock(Router::new().route(
            "/chat/completions",
            post(move || {
                let index = observed.fetch_add(1, Ordering::SeqCst);
                async move {
                    Json(reply(
                        Value::Null,
                        json!([call(&format!("call-{index}"), "unknown", "{}")]),
                        "tool_calls",
                    ))
                }
            }),
        ))
        .await;
        let error = run_agent(&client, TUTORIAL_MODEL, None, Duration::from_secs(2))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("step limit"));
        assert_eq!(requests.load(Ordering::SeqCst), MAX_AGENT_STEPS);
        job.abort();
    }

    #[tokio::test]
    async fn upstream_failure_and_total_deadline_fail() {
        let (client, job) = mock(Router::new().route(
            "/chat/completions",
            post(|| async {
                (
                    axum::http::StatusCode::BAD_GATEWAY,
                    Json(json!({"error":{"code":502,"message":"failure"}})),
                )
            }),
        ))
        .await;
        assert!(
            run_agent(&client, TUTORIAL_MODEL, None, Duration::from_secs(2))
                .await
                .is_err()
        );
        job.abort();
        let (client, job) = mock(Router::new().route(
            "/chat/completions",
            post(|| async {
                tokio::time::sleep(Duration::from_secs(5)).await;
                Json(reply(json!("late"), Value::Null, "stop"))
            }),
        ))
        .await;
        let start = Instant::now();
        let error = run_agent(&client, TUTORIAL_MODEL, None, Duration::from_millis(100))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("deadline"));
        assert!(start.elapsed() < Duration::from_secs(2));
        job.abort();
    }
}
