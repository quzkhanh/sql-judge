use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

mod comparator;
mod verdict;

pub use comparator::{ComparisonMismatch, ComparisonOutcome, compare_query_results};
pub use verdict::{ExecutionFailure, Verdict, aggregate_verdicts, map_execution_failure};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum SqlValue {
    Null,
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Decimal(String),
    Float { bits: u64 },
    Text(String),
    Bytes(Vec<u8>),
    Date(String),
    Timestamp(String),
    Timestamptz(String),
    Uuid(String),
    Json(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogicalType {
    Integer,
    Bigint,
    Numeric,
    Float,
    Text,
    Boolean,
    Date,
    Timestamp,
    Timestamptz,
    Uuid,
    Json,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub logical_type: LogicalType,
}

impl Column {
    pub fn new(name: impl Into<String>, logical_type: LogicalType) -> Self {
        Self {
            name: name.into(),
            logical_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryResult {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<SqlValue>>,
}

impl QueryResult {
    pub fn new(columns: Vec<Column>, rows: Vec<Vec<SqlValue>>) -> Result<Self, ContractError> {
        let result = Self { columns, rows };
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        validate_columns(&self.columns, "result")?;

        for (row_index, row) in self.rows.iter().enumerate() {
            if row.len() != self.columns.len() {
                return Err(ContractError::RowWidth {
                    row: row_index,
                    actual: row.len(),
                    expected: self.columns.len(),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    Postgresql,
    Sqlserver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RowOrder {
    Ordered,
    Unordered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputPolicy {
    pub row_order: RowOrder,
    pub strict_column_names: bool,
    pub strict_column_types: bool,
    pub columns: Vec<Column>,
}

impl OutputPolicy {
    pub fn new(
        row_order: RowOrder,
        strict_column_names: bool,
        strict_column_types: bool,
        columns: Vec<Column>,
    ) -> Self {
        Self {
            row_order,
            strict_column_names,
            strict_column_types,
            columns,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        validate_columns(&self.columns, "output")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryLimits {
    pub statement_timeout_ms: u32,
    pub max_rows: u32,
    pub max_bytes: u32,
}

impl QueryLimits {
    pub fn new(statement_timeout_ms: u32, max_rows: u32, max_bytes: u32) -> Self {
        Self {
            statement_timeout_ms,
            max_rows,
            max_bytes,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if !(100..=10_000).contains(&self.statement_timeout_ms) {
            return Err(ContractError::InvalidLimit {
                field: "statement_timeout_ms",
                value: self.statement_timeout_ms,
            });
        }
        if !(1..=10_000).contains(&self.max_rows) {
            return Err(ContractError::InvalidLimit {
                field: "max_rows",
                value: self.max_rows,
            });
        }
        if !(1024..=16 * 1024 * 1024).contains(&self.max_bytes) {
            return Err(ContractError::InvalidLimit {
                field: "max_bytes",
                value: self.max_bytes,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProblemContract {
    pub id: String,
    pub dialect: Dialect,
    pub statement: String,
    pub output: OutputPolicy,
    pub limits: QueryLimits,
}

impl ProblemContract {
    pub fn new(
        id: impl Into<String>,
        dialect: Dialect,
        statement: impl Into<String>,
        output: OutputPolicy,
        limits: QueryLimits,
    ) -> Result<Self, ContractError> {
        let contract = Self {
            id: id.into(),
            dialect,
            statement: statement.into(),
            output,
            limits,
        };
        contract.validate()?;
        Ok(contract)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if !is_problem_id(&self.id) {
            return Err(ContractError::InvalidProblemId(self.id.clone()));
        }
        if self.statement.trim().is_empty() {
            return Err(ContractError::EmptyStatement);
        }
        self.output.validate()?;
        self.limits.validate()
    }
}

fn validate_columns(columns: &[Column], context: &'static str) -> Result<(), ContractError> {
    if columns.is_empty() {
        return Err(ContractError::NoColumns { context });
    }

    for column in columns {
        if column.name.trim().is_empty() {
            return Err(ContractError::EmptyColumnName { context });
        }
    }

    for (index, column) in columns.iter().enumerate() {
        if columns[..index]
            .iter()
            .any(|previous| previous.name == column.name)
        {
            return Err(ContractError::DuplicateColumnName {
                context,
                name: column.name.clone(),
            });
        }
    }

    Ok(())
}

fn is_problem_id(value: &str) -> bool {
    if value.is_empty() || value.len() > 80 {
        return false;
    }

    value.split('-').all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
            && part
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_lowercase())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    NoColumns {
        context: &'static str,
    },
    EmptyColumnName {
        context: &'static str,
    },
    DuplicateColumnName {
        context: &'static str,
        name: String,
    },
    RowWidth {
        row: usize,
        actual: usize,
        expected: usize,
    },
    InvalidLimit {
        field: &'static str,
        value: u32,
    },
    InvalidProblemId(String),
    EmptyStatement,
}

impl Display for ContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoColumns { context } => write!(formatter, "{context} must declare columns"),
            Self::EmptyColumnName { context } => {
                write!(formatter, "{context} column name is empty")
            }
            Self::DuplicateColumnName { context, name } => {
                write!(formatter, "{context} contains duplicate column {name}")
            }
            Self::RowWidth {
                row,
                actual,
                expected,
            } => write!(
                formatter,
                "row {row} has {actual} values, expected {expected}"
            ),
            Self::InvalidLimit { field, value } => {
                write!(formatter, "invalid {field} limit: {value}")
            }
            Self::InvalidProblemId(id) => write!(formatter, "invalid problem id: {id}"),
            Self::EmptyStatement => formatter.write_str("problem statement is empty"),
        }
    }
}

impl std::error::Error for ContractError {}
