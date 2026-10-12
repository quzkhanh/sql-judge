use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    SyntaxError,
    RuntimeError,
    TimeLimitExceeded,
    ResultLimitExceeded,
    PolicyViolation,
    Cancelled,
    JudgeError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionFailure {
    Setup,
    Reference,
    UserSyntax,
    UserRuntime,
    Timeout,
    ResultLimit,
    Policy,
    Cancelled,
}

pub fn map_execution_failure(failure: ExecutionFailure) -> Verdict {
    match failure {
        ExecutionFailure::Setup | ExecutionFailure::Reference => Verdict::JudgeError,
        ExecutionFailure::UserSyntax => Verdict::SyntaxError,
        ExecutionFailure::UserRuntime => Verdict::RuntimeError,
        ExecutionFailure::Timeout => Verdict::TimeLimitExceeded,
        ExecutionFailure::ResultLimit => Verdict::ResultLimitExceeded,
        ExecutionFailure::Policy => Verdict::PolicyViolation,
        ExecutionFailure::Cancelled => Verdict::Cancelled,
    }
}

pub fn aggregate_verdicts(verdicts: &[Verdict]) -> Verdict {
    verdicts
        .iter()
        .copied()
        .max_by_key(|verdict| verdict_rank(*verdict))
        .unwrap_or(Verdict::JudgeError)
}

fn verdict_rank(verdict: Verdict) -> u8 {
    match verdict {
        Verdict::Accepted => 0,
        Verdict::WrongAnswer => 10,
        Verdict::SyntaxError => 20,
        Verdict::RuntimeError => 30,
        Verdict::ResultLimitExceeded => 40,
        Verdict::TimeLimitExceeded => 50,
        Verdict::Cancelled => 60,
        Verdict::PolicyViolation => 70,
        Verdict::JudgeError => 80,
    }
}
