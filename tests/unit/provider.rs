use openrouter_rs::types::{
    DataCollectionPolicy, MaxPrice, PercentileCutoffs, PerformancePreference, PriceLimit,
    ProviderPreferences, ProviderSortBy, Quantization,
};

#[test]
fn test_provider_preferences_new_fields_scalar_serialize() {
    let mut max_price = MaxPrice::default();
    max_price.prompt = Some(PriceLimit::Number(1.25));
    max_price.completion = Some(PriceLimit::String("2.5".to_string()));
    max_price.request = Some(PriceLimit::Number(0.01));

    let mut prefs = ProviderPreferences::default();
    prefs.allow_fallbacks = Some(true);
    prefs.require_parameters = Some(true);
    prefs.data_collection = Some(DataCollectionPolicy::Deny);
    prefs.zdr = Some(true);
    prefs.enforce_distillable_text = Some(true);
    prefs.order = Some(vec!["openai".to_string(), "anthropic".to_string()]);
    prefs.only = Some(vec!["openai".to_string()]);
    prefs.ignore = Some(vec!["some-provider".to_string()]);
    prefs.quantizations = Some(vec![Quantization::Fp16]);
    prefs.sort = Some(ProviderSortBy::Price);
    prefs.max_price = Some(max_price);
    prefs.preferred_min_throughput = Some(PerformancePreference::Value(120.0));
    prefs.preferred_max_latency = Some(PerformancePreference::Value(3.5));

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
    let mut throughput = PercentileCutoffs::default();
    throughput.p50 = Some(100.0);
    throughput.p90 = Some(50.0);

    let mut latency = PercentileCutoffs::default();
    latency.p50 = Some(2.5);
    latency.p75 = Some(3.0);

    let mut prefs = ProviderPreferences::default();
    prefs.preferred_min_throughput = Some(PerformancePreference::Percentiles(throughput));
    prefs.preferred_max_latency = Some(PerformancePreference::Percentiles(latency));

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
