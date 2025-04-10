use openrouter_rs::config::OpenRouterConfig;

#[test]
fn test_default_config() {
    let config = OpenRouterConfig::default();

    // Verify enabled presets
    assert_eq!(
        config.models.enable,
        vec![
            "preset:thinking".to_string(),
            "preset:coding".to_string(),
            "preset:free".to_string()
        ]
    );

    // Verify thinking preset models
    assert_eq!(
        config.models.presets.get("thinking").unwrap(),
        &vec![
            "openai/o3-mini-high".to_string(),
            "anthropic/claude-3.7-sonnet:thinking".to_string(),
            "deepseek/deepseek-r1".to_string()
        ]
    );

    // Verify coding preset models
    assert_eq!(
        config.models.presets.get("coding").unwrap(),
        &vec![
            "openai/gpt-4o-mini".to_string(),
            "openai/gpt-4o-2024-11-20".to_string(),
            "openai/o3-mini-high".to_string(),
            "anthropic/claude-3.7-sonnet".to_string(),
            "anthropic/claude-3.7-sonnet:thinking".to_string(),
            "deepseek/deepseek-chat-v3-0324".to_string(),
            "google/gemini-2.0-flash-001".to_string(),
            "google/gemini-2.5-pro-preview-03-25".to_string(),
            "openrouter/quasar-alpha".to_string()
        ]
    );

    // Verify free preset models
    assert_eq!(
        config.models.presets.get("free").unwrap(),
        &vec![
            "deepseek/deepseek-r1:free".to_string(),
            "deepseek/deepseek-chat-v3-0324:free".to_string(),
            "google/gemini-2.5-pro-exp-03-25:free".to_string()
        ]
    );

    // Verify preset count
    assert_eq!(config.models.presets.len(), 3);

    // Verify resolution of model presets
    assert_eq!(config.models.resolved_models.len(), 13)
}
