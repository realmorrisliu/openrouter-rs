use openrouter_rs::types::PaginationOptions;

#[test]
fn test_pagination_to_query_pairs_orders_offset_then_limit() {
    let pagination = PaginationOptions::with_offset_and_limit(10, 20);
    let pairs = pagination.to_query_pairs();

    assert_eq!(
        pairs,
        vec![
            ("offset", String::from("10")),
            ("limit", String::from("20"))
        ]
    );
}

#[test]
fn test_pagination_to_query_pairs_omits_unset_values() {
    let pagination = PaginationOptions::with_limit(30);
    let pairs = pagination.to_query_pairs();

    assert_eq!(pairs, vec![("limit", String::from("30"))]);
}
