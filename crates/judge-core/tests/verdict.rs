use judge_core::{ExecutionFailure, Verdict, aggregate_verdicts, map_execution_failure};

#[test]
fn reference_failure_is_a_judge_error() {
    assert_eq!(
        map_execution_failure(ExecutionFailure::Reference),
        Verdict::JudgeError
    );
    assert_eq!(
        map_execution_failure(ExecutionFailure::Setup),
        Verdict::JudgeError
    );
}

#[test]
fn user_failures_keep_distinct_verdicts() {
    assert_eq!(
        map_execution_failure(ExecutionFailure::UserSyntax),
        Verdict::SyntaxError
    );
    assert_eq!(
        map_execution_failure(ExecutionFailure::UserRuntime),
        Verdict::RuntimeError
    );
    assert_eq!(
        map_execution_failure(ExecutionFailure::ResultLimit),
        Verdict::ResultLimitExceeded
    );
}

#[test]
fn aggregate_uses_the_most_severe_fail_closed_verdict() {
    assert_eq!(
        aggregate_verdicts(&[Verdict::Accepted, Verdict::WrongAnswer]),
        Verdict::WrongAnswer
    );
    assert_eq!(
        aggregate_verdicts(&[Verdict::WrongAnswer, Verdict::JudgeError]),
        Verdict::JudgeError
    );
    assert_eq!(aggregate_verdicts(&[]), Verdict::JudgeError);
}
