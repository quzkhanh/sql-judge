use judge_core::{
    Column, ComparisonMismatch, LogicalType, OutputPolicy, QueryResult, RowOrder, SqlValue,
    compare_query_results,
};

fn policy(row_order: RowOrder) -> OutputPolicy {
    OutputPolicy::new(
        row_order,
        true,
        true,
        vec![Column::new("value", LogicalType::Bigint)],
    )
}

fn result(values: Vec<SqlValue>) -> QueryResult {
    QueryResult::new(
        vec![Column::new("value", LogicalType::Bigint)],
        values.into_iter().map(|value| vec![value]).collect(),
    )
    .expect("valid result")
}

#[test]
fn unordered_comparison_preserves_duplicate_counts() {
    let expected = result(vec![
        SqlValue::Signed(1),
        SqlValue::Signed(1),
        SqlValue::Signed(2),
    ]);
    let actual = result(vec![
        SqlValue::Signed(1),
        SqlValue::Signed(2),
        SqlValue::Signed(2),
    ]);

    let outcome = compare_query_results(&expected, &actual, &policy(RowOrder::Unordered));

    assert!(!outcome.is_match());
    assert_eq!(outcome.mismatch, Some(ComparisonMismatch::RowValues));
}

#[test]
fn unordered_comparison_ignores_row_order() {
    let expected = result(vec![SqlValue::Signed(1), SqlValue::Signed(2)]);
    let actual = result(vec![SqlValue::Signed(2), SqlValue::Signed(1)]);

    assert!(compare_query_results(&expected, &actual, &policy(RowOrder::Unordered)).is_match());
}

#[test]
fn ordered_comparison_rejects_reordered_rows() {
    let expected = result(vec![SqlValue::Signed(1), SqlValue::Signed(2)]);
    let actual = result(vec![SqlValue::Signed(2), SqlValue::Signed(1)]);

    let outcome = compare_query_results(&expected, &actual, &policy(RowOrder::Ordered));

    assert!(!outcome.is_match());
    assert_eq!(outcome.mismatch, Some(ComparisonMismatch::RowValues));
}

#[test]
fn null_is_not_text_null() {
    let expected = result(vec![SqlValue::Null]);
    let actual = result(vec![SqlValue::Text("NULL".to_owned())]);

    assert!(!compare_query_results(&expected, &actual, &policy(RowOrder::Unordered)).is_match());
}

#[test]
fn strict_column_names_are_compared_by_position() {
    let expected = result(vec![SqlValue::Signed(1)]);
    let actual = QueryResult::new(
        vec![Column::new("other", LogicalType::Bigint)],
        vec![vec![SqlValue::Signed(1)]],
    )
    .expect("valid result");

    let outcome = compare_query_results(&expected, &actual, &policy(RowOrder::Ordered));

    assert!(!outcome.is_match());
    assert_eq!(outcome.mismatch, Some(ComparisonMismatch::ColumnNames));
}
