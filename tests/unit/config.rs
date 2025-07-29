use openrouter_rs::config::OpenRouterConfig;

#[test]
fn test_default_config() {
    let config = OpenRouterConfig::default();

    // Verify enabled presets
    assert_eq!(
        config.models.enable,
        vec![
            "preset:programming".to_string(),
            "preset:reasoning".to_string(),
            "preset:free".to_string()
        ]
    );

    // Verify programming preset models
    assert_eq!(
        config.models.presets.get("programming").unwrap(),
        &vec![
            "anthropic/claude-sonnet-4".to_string(),
            "google/gemini-2.5-flash".to_string(),
            "qwen/qwen3-coder".to_string(),
            "google/gemini-2.5-pro".to_string(),
            "anthropic/claude-3.7-sonnet".to_string(),
            "moonshotai/kimi-k2".to_string(),
            "x-ai/grok-4".to_string(),
            "anthropic/claude-opus-4".to_string(),
            "qwen/qwen3-235b-a22b-2507".to_string(),
            "deepseek/deepseek-chat-v3-0324".to_string()
        ]
    );

    // Verify reasoning preset models
    assert_eq!(
        config.models.presets.get("reasoning").unwrap(),
        &vec![
            "anthropic/claude-sonnet-4".to_string(),
            "google/gemini-2.5-flash".to_string(),
            "google/gemini-2.5-pro".to_string(),
            "anthropic/claude-3.7-sonnet".to_string(),
            "deepseek/deepseek-r1-0528:free".to_string(),
            "google/gemini-2.5-flash-lite-preview-06-17".to_string(),
            "anthropic/claude-opus-4".to_string(),
            "deepseek/deepseek-r1:free".to_string(),
            "x-ai/grok-4".to_string(),
            "google/gemini-2.5-flash-lite".to_string()
        ]
    );

    // Verify free preset models
    assert_eq!(
        config.models.presets.get("free").unwrap(),
        &vec![
            "deepseek/deepseek-chat-v3-0324:free".to_string(),
            "qwen/qwen3-coder:free".to_string(),
            "deepseek/deepseek-r1-0528:free".to_string(),
            "qwen/qwen3-235b-a22b-2507:free".to_string(),
            "deepseek/deepseek-r1:free".to_string(),
            "tngtech/deepseek-r1t2-chimera:free".to_string(),
            "moonshotai/kimi-k2:free".to_string(),
            "tngtech/deepseek-r1t-chimera:free".to_string(),
            "google/gemini-2.0-flash-exp:free".to_string(),
            "qwen/qwen-2.5-72b-instruct:free".to_string()
        ]
    );

    // Verify preset count
    assert_eq!(config.models.presets.len(), 3);

    // Verify resolution of model presets (22 unique models after deduplication)
    assert_eq!(config.models.resolved_models.len(), 22)
}
