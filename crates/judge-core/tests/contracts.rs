use judge_core::{
    Column, Dialect, LogicalType, OutputPolicy, ProblemContract, QueryLimits, QueryResult,
    RowOrder, SqlValue,
};

#[test]
fn query_result_roundtrip_preserves_strict_value_tags() {
    let result = QueryResult::new(
        vec![
            Column::new("id", LogicalType::Bigint),
            Column::new("note", LogicalType::Text),
        ],
        vec![vec![SqlValue::Signed(7), SqlValue::Null]],
    )
    .expect("valid query result");

    let encoded = serde_json::to_string(&result).expect("serialize result");
    assert!(encoded.contains(r#""type":"signed""#));
    assert!(encoded.contains(r#""type":"null""#));
    assert_eq!(
        result,
        serde_json::from_str(&encoded).expect("deserialize result")
    );
}

#[test]
fn unknown_value_tag_and_fields_are_rejected() {
    assert!(serde_json::from_str::<SqlValue>(r#"{"type":"secret","value":"x"}"#).is_err());
    assert!(
        serde_json::from_str::<Column>(r#"{"name":"id","type":"bigint","extra":true}"#).is_err()
    );
}

#[test]
fn query_result_rejects_rows_with_wrong_width() {
    let error = QueryResult::new(
        vec![Column::new("id", LogicalType::Bigint)],
        vec![vec![SqlValue::Signed(1), SqlValue::Signed(2)]],
    )
    .expect_err("row width must match columns");

    assert!(error.to_string().contains("row 0 has 2 values, expected 1"));
}

#[test]
fn problem_contract_rejects_invalid_limits_and_duplicate_columns() {
    let contract = ProblemContract::new(
        "Above Average",
        Dialect::Postgresql,
        "SELECT id FROM employees",
        OutputPolicy::new(
            RowOrder::Unordered,
            true,
            true,
            vec![
                Column::new("id", LogicalType::Bigint),
                Column::new("id", LogicalType::Bigint),
            ],
        ),
        QueryLimits::new(50, 0, 17 * 1024 * 1024),
    );

    let error = contract.expect_err("invalid problem contract");
    assert!(error.to_string().contains("problem id"));
}
