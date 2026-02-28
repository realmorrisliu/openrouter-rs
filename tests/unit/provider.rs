use openrouter_rs::types::{
    DataCollectionPolicy, MaxPrice, PercentileCutoffs, PerformancePreference, PriceLimit,
    ProviderPreferences, ProviderSortBy, Quantization,
};

#[test]
fn test_provider_preferences_new_fields_scalar_serialize() {
    let prefs = ProviderPreferences {
        allow_fallbacks: Some(true),
        require_parameters: Some(true),
        data_collection: Some(DataCollectionPolicy::Deny),
        zdr: Some(true),
        enforce_distillable_text: Some(true),
        order: Some(vec!["openai".to_string(), "anthropic".to_string()]),
        only: Some(vec!["openai".to_string()]),
        ignore: Some(vec!["some-provider".to_string()]),
        quantizations: Some(vec![Quantization::Fp16]),
        sort: Some(ProviderSortBy::Price),
        max_price: Some(MaxPrice {
            prompt: Some(PriceLimit::Number(1.25)),
            completion: Some(PriceLimit::String("2.5".to_string())),
            image: None,
            audio: None,
            request: Some(PriceLimit::Number(0.01)),
        }),
        preferred_min_throughput: Some(PerformancePreference::Value(120.0)),
        preferred_max_latency: Some(PerformancePreference::Value(3.5)),
    };

    let json = serde_json::to_value(&prefs).expect("provider prefs should serialize");

    assert_eq!(json["allow_fallbacks"], true);
    assert_eq!(json["require_parameters"], true);
    assert_eq!(json["data_collection"], "deny");
    assert_eq!(json["zdr"], true);
    assert_eq!(json["enforce_distillable_text"], true);
    assert_eq!(json["only"][0], "openai");
    assert_eq!(json["max_price"]["prompt"], 1.25);
    assert_eq!(json["max_price"]["completion"], "2.5");
    assert_eq!(json["max_price"]["request"], 0.01);
    assert_eq!(json["preferred_min_throughput"], 120.0);
    assert_eq!(json["preferred_max_latency"], 3.5);
}

#[test]
fn test_provider_preferences_percentile_preferences_serialize() {
    let prefs = ProviderPreferences {
        preferred_min_throughput: Some(PerformancePreference::Percentiles(PercentileCutoffs {
            p50: Some(100.0),
            p75: None,
            p90: Some(50.0),
            p99: None,
        })),
        preferred_max_latency: Some(PerformancePreference::Percentiles(PercentileCutoffs {
            p50: Some(2.5),
            p75: Some(3.0),
            p90: None,
            p99: None,
        })),
        ..Default::default()
    };

    let json = serde_json::to_value(&prefs).expect("provider prefs should serialize");

    assert_eq!(json["preferred_min_throughput"]["p50"], 100.0);
    assert_eq!(json["preferred_min_throughput"]["p90"], 50.0);
    assert_eq!(json["preferred_max_latency"]["p50"], 2.5);
    assert_eq!(json["preferred_max_latency"]["p75"], 3.0);
}

#[test]
fn test_provider_preferences_new_fields_deserialize() {
    let raw = r#"{
        "zdr": true,
        "enforce_distillable_text": true,
        "only": ["openai", "anthropic"],
        "max_price": {
            "prompt": "1.0",
            "completion": 2.0
        },
        "preferred_min_throughput": {
            "p50": 80,
            "p90": 40
        },
        "preferred_max_latency": 4
    }"#;

    let prefs: ProviderPreferences =
        serde_json::from_str(raw).expect("provider prefs should deserialize");
    let json = serde_json::to_value(&prefs).expect("provider prefs should re-serialize");

    assert_eq!(json["zdr"], true);
    assert_eq!(json["enforce_distillable_text"], true);
    assert_eq!(json["only"][0], "openai");
    assert_eq!(json["max_price"]["prompt"], "1.0");
    assert_eq!(json["max_price"]["completion"], 2.0);
    assert_eq!(json["preferred_min_throughput"]["p50"], 80.0);
    assert_eq!(json["preferred_min_throughput"]["p90"], 40.0);
    assert_eq!(json["preferred_max_latency"], 4.0);
}
