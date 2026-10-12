use crate::{PostgresSession, QueryExecutionError};
use judge_core::{
    ComparisonMismatch, OutputPolicy, QueryLimits, QueryResult, Verdict, aggregate_verdicts,
    compare_query_results,
};

#[derive(Debug, Clone)]
pub struct JudgeCase<'a> {
    pub id: &'a str,
    pub schema_sql: &'a str,
    pub fixture_sql: &'a str,
    pub reference_sql: &'a str,
    pub output: OutputPolicy,
    pub limits: QueryLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestOutcome {
    pub id: String,
    pub verdict: Verdict,
    pub mismatch: Option<ComparisonMismatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionReport {
    pub verdict: Verdict,
    pub tests: Vec<TestOutcome>,
}

struct FixtureExecution {
    reference: Result<QueryResult, QueryExecutionError>,
    user: Option<Result<QueryResult, QueryExecutionError>>,
}

pub fn judge_submission(
    session: &mut PostgresSession,
    runner_role: &str,
    cases: &[JudgeCase<'_>],
    user_sql: &str,
) -> SubmissionReport {
    judge_submission_with_cancel(session, runner_role, cases, user_sql, || false)
}

pub fn judge_submission_with_cancel<F>(
    session: &mut PostgresSession,
    runner_role: &str,
    cases: &[JudgeCase<'_>],
    user_sql: &str,
    is_cancelled: F,
) -> SubmissionReport
where
    F: Fn() -> bool,
{
    let mut tests = Vec::with_capacity(cases.len());

    for case in cases {
        if is_cancelled() {
            return SubmissionReport {
                verdict: Verdict::Cancelled,
                tests,
            };
        }
        let execution = session.with_managed_fixture(
            runner_role,
            case.schema_sql,
            case.fixture_sql,
            |session| {
                let reference = session.execute(case.reference_sql, case.limits);
                let user = reference
                    .as_ref()
                    .ok()
                    .map(|_| session.execute(user_sql, case.limits));
                Ok(FixtureExecution { reference, user })
            },
        );

        if is_cancelled() {
            return SubmissionReport {
                verdict: Verdict::Cancelled,
                tests,
            };
        }

        let outcome = match execution {
            Err(_) => TestOutcome {
                id: case.id.to_owned(),
                verdict: Verdict::JudgeError,
                mismatch: None,
            },
            Ok(FixtureExecution {
                reference: Err(_), ..
            }) => TestOutcome {
                id: case.id.to_owned(),
                verdict: Verdict::JudgeError,
                mismatch: None,
            },
            Ok(FixtureExecution {
                reference: Ok(_),
                user: Some(Err(error)),
            }) => TestOutcome {
                id: case.id.to_owned(),
                verdict: user_error_verdict(error),
                mismatch: None,
            },
            Ok(FixtureExecution {
                reference: Ok(expected),
                user: Some(Ok(actual)),
            }) => {
                let comparison = compare_query_results(&expected, &actual, &case.output);
                TestOutcome {
                    id: case.id.to_owned(),
                    verdict: if comparison.is_match() {
                        Verdict::Accepted
                    } else {
                        Verdict::WrongAnswer
                    },
                    mismatch: comparison.mismatch,
                }
            }
            Ok(FixtureExecution {
                reference: Ok(_),
                user: None,
            }) => TestOutcome {
                id: case.id.to_owned(),
                verdict: Verdict::JudgeError,
                mismatch: None,
            },
        };
        tests.push(outcome);
    }

    let verdicts: Vec<_> = tests.iter().map(|test| test.verdict).collect();
    SubmissionReport {
        verdict: aggregate_verdicts(&verdicts),
        tests,
    }
}

fn user_error_verdict(error: QueryExecutionError) -> Verdict {
    match error {
        QueryExecutionError::PolicyViolation => Verdict::PolicyViolation,
        QueryExecutionError::SyntaxError => Verdict::SyntaxError,
        QueryExecutionError::TimeLimitExceeded => Verdict::TimeLimitExceeded,
        QueryExecutionError::ResultLimitExceeded => Verdict::ResultLimitExceeded,
        QueryExecutionError::RuntimeError => Verdict::RuntimeError,
        QueryExecutionError::InvalidLimits(_)
        | QueryExecutionError::UnsupportedType(_)
        | QueryExecutionError::ConnectionError
        | QueryExecutionError::OwnershipVerificationFailed => Verdict::JudgeError,
    }
}
