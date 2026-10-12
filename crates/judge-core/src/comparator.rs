use crate::{OutputPolicy, QueryResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonMismatch {
    InvalidExpectedResult,
    InvalidActualResult,
    InvalidPolicy,
    ColumnCount,
    ColumnNames,
    ColumnTypes,
    RowCount,
    RowValues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparisonOutcome {
    pub mismatch: Option<ComparisonMismatch>,
}

impl ComparisonOutcome {
    pub fn is_match(self) -> bool {
        self.mismatch.is_none()
    }

    fn matched() -> Self {
        Self { mismatch: None }
    }

    fn mismatch(reason: ComparisonMismatch) -> Self {
        Self {
            mismatch: Some(reason),
        }
    }
}

pub fn compare_query_results(
    expected: &QueryResult,
    actual: &QueryResult,
    policy: &OutputPolicy,
) -> ComparisonOutcome {
    if expected.validate().is_err() {
        return ComparisonOutcome::mismatch(ComparisonMismatch::InvalidExpectedResult);
    }
    if actual.validate().is_err() {
        return ComparisonOutcome::mismatch(ComparisonMismatch::InvalidActualResult);
    }
    if policy.validate().is_err() || policy.columns != expected.columns {
        return ComparisonOutcome::mismatch(ComparisonMismatch::InvalidPolicy);
    }
    if expected.columns.len() != actual.columns.len() {
        return ComparisonOutcome::mismatch(ComparisonMismatch::ColumnCount);
    }

    for (expected_column, actual_column) in expected.columns.iter().zip(&actual.columns) {
        if policy.strict_column_names && expected_column.name != actual_column.name {
            return ComparisonOutcome::mismatch(ComparisonMismatch::ColumnNames);
        }
        if policy.strict_column_types && expected_column.logical_type != actual_column.logical_type
        {
            return ComparisonOutcome::mismatch(ComparisonMismatch::ColumnTypes);
        }
    }

    if expected.rows.len() != actual.rows.len() {
        return ComparisonOutcome::mismatch(ComparisonMismatch::RowCount);
    }

    let rows_match = match policy.row_order {
        crate::RowOrder::Ordered => expected.rows == actual.rows,
        crate::RowOrder::Unordered => unordered_rows_match(expected, actual),
    };

    if rows_match {
        ComparisonOutcome::matched()
    } else {
        ComparisonOutcome::mismatch(ComparisonMismatch::RowValues)
    }
}

fn unordered_rows_match(expected: &QueryResult, actual: &QueryResult) -> bool {
    let mut used = vec![false; actual.rows.len()];

    expected.rows.iter().all(|expected_row| {
        actual
            .rows
            .iter()
            .enumerate()
            .find(|(index, actual_row)| !used[*index] && *actual_row == expected_row)
            .map(|(index, _)| {
                used[index] = true;
                true
            })
            .unwrap_or(false)
    })
}
