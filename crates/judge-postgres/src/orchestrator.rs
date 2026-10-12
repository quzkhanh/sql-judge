use crate::{CancellationReason, CancellationToken, PostgresSession, QueryExecutionError};
use judge_core::{
    ComparisonMismatch, OutputPolicy, QueryLimits, QueryResult, Verdict, aggregate_verdicts,
    compare_query_results,
};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    judge_submission_inner(session, runner_role, cases, user_sql, &|| false, None)
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
    judge_submission_inner(session, runner_role, cases, user_sql, &is_cancelled, None)
}

pub fn judge_submission_with_token(
    session: &mut PostgresSession,
    runner_role: &str,
    cases: &[JudgeCase<'_>],
    user_sql: &str,
    token: &CancellationToken,
) -> SubmissionReport {
    judge_submission_inner(
        session,
        runner_role,
        cases,
        user_sql,
        &|| token.is_cancelled(),
        Some(token),
    )
}

pub fn judge_submission_with_watchdog(
    session: &mut PostgresSession,
    runner_role: &str,
    cases: &[JudgeCase<'_>],
    user_sql: &str,
    watchdog_after: Duration,
) -> SubmissionReport {
    let token = CancellationToken::default();
    judge_submission_with_watchdog_token(
        session,
        runner_role,
        cases,
        user_sql,
        &token,
        watchdog_after,
    )
}

pub fn judge_submission_with_watchdog_token(
    session: &mut PostgresSession,
    runner_role: &str,
    cases: &[JudgeCase<'_>],
    user_sql: &str,
    token: &CancellationToken,
    watchdog_after: Duration,
) -> SubmissionReport {
    let (finished, deadline) = mpsc::channel();
    let watchdog_token = token.clone();
    let watchdog = thread::spawn(move || {
        if deadline.recv_timeout(watchdog_after).is_err() {
            watchdog_token.cancel_for_watchdog();
        }
    });
    let report = judge_submission_with_token(session, runner_role, cases, user_sql, token);
    let _ = finished.send(());
    let _ = watchdog.join();
    report
}

fn judge_submission_inner(
    session: &mut PostgresSession,
    runner_role: &str,
    cases: &[JudgeCase<'_>],
    user_sql: &str,
    is_cancelled: &dyn Fn() -> bool,
    cancellation: Option<&CancellationToken>,
) -> SubmissionReport {
    let mut tests = Vec::with_capacity(cases.len());

    for case in cases {
        if is_cancelled() {
            return SubmissionReport {
                verdict: cancellation_verdict(cancellation),
                tests,
            };
        }
        let execution = session.with_managed_fixture(
            runner_role,
            case.schema_sql,
            case.fixture_sql,
            |session| {
                let reference = match cancellation {
                    Some(token) => {
                        session.execute_with_cancel(case.reference_sql, case.limits, token)
                    }
                    None => session.execute(case.reference_sql, case.limits),
                };
                let user = reference.as_ref().ok().map(|_| match cancellation {
                    Some(token) => session.execute_with_cancel(user_sql, case.limits, token),
                    None => session.execute(user_sql, case.limits),
                });
                Ok(FixtureExecution { reference, user })
            },
        );

        if is_cancelled() {
            return SubmissionReport {
                verdict: cancellation_verdict(cancellation),
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

fn cancellation_verdict(token: Option<&CancellationToken>) -> Verdict {
    match token.and_then(CancellationToken::reason) {
        Some(CancellationReason::Watchdog) => Verdict::TimeLimitExceeded,
        Some(CancellationReason::User) | None => Verdict::Cancelled,
    }
}

fn user_error_verdict(error: QueryExecutionError) -> Verdict {
    match error {
        QueryExecutionError::PolicyViolation => Verdict::PolicyViolation,
        QueryExecutionError::SyntaxError => Verdict::SyntaxError,
        QueryExecutionError::TimeLimitExceeded => Verdict::TimeLimitExceeded,
        QueryExecutionError::Cancelled => Verdict::Cancelled,
        QueryExecutionError::ResultLimitExceeded => Verdict::ResultLimitExceeded,
        QueryExecutionError::RuntimeError => Verdict::RuntimeError,
        QueryExecutionError::InvalidLimits(_)
        | QueryExecutionError::UnsupportedType(_)
        | QueryExecutionError::ConnectionError
        | QueryExecutionError::OwnershipVerificationFailed => Verdict::JudgeError,
    }
}
