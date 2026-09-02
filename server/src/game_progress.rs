use chrono::{DateTime, Duration, Utc};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProblemStatus {
    Locked,
    Available,
    Cleared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunStatus {
    Active,
    Cleared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ProgressStatus {
    NotStarted,
    Active,
    Cleared,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProblemState {
    pub(crate) problem_id: Uuid,
    pub(crate) room_id: Uuid,
    pub(crate) depends_on_problem_id: Option<Uuid>,
    pub(crate) is_required: bool,
    pub(crate) status: ProblemStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveRunState {
    pub(crate) run_id: Uuid,
    pub(crate) room_id: Uuid,
    pub(crate) started_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Progress {
    pub(crate) status: ProgressStatus,
    pub(crate) cleared_count: usize,
    pub(crate) required_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClearProblemPlan {
    pub(crate) target_problem_status: ProblemStatus,

    // Some(now)なら対象problemを今回更新する。
    // Noneなら既clearなのでUPDATEしない。
    pub(crate) problem_cleared_at: Option<DateTime<Utc>>,

    pub(crate) unlocked_problem_ids: Vec<Uuid>,
    pub(crate) progress: Progress,
    pub(crate) run_status: RunStatus,

    // Some(now)ならrunを今回clearedへ更新する。
    pub(crate) run_cleared_at: Option<DateTime<Utc>>,

    pub(crate) elapsed: Duration,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub(crate) enum ClearProblemError {
    #[error("problem was not found in the active run")]
    ProblemNotFound,

    #[error("problem is locked")]
    ProblemLocked,

    #[error("elapsed duration must not be negative")]
    InvalidElapsed,
}

/// Computes the state changes caused by clearing one problem.
///
/// This function does not access or update the database. The caller must apply
/// the returned plan inside the transaction that records the correct judgement.
pub(crate) fn plan_problem_clear(
    run: &ActiveRunState,
    problems: &[ProblemState],
    target_problem_id: Uuid,
    now: DateTime<Utc>,
) -> Result<ClearProblemPlan, ClearProblemError> {
    let target = problems
        .iter()
        .find(|problem| problem.problem_id == target_problem_id && problem.room_id == run.room_id)
        .ok_or(ClearProblemError::ProblemNotFound)?;

    if target.status == ProblemStatus::Locked {
        return Err(ClearProblemError::ProblemLocked);
    }

    let elapsed = now.signed_duration_since(run.started_at);
    if elapsed < Duration::zero() {
        return Err(ClearProblemError::InvalidElapsed);
    }

    let is_newly_cleared = target.status == ProblemStatus::Available;

    let problem_cleared_at = is_newly_cleared.then_some(now);

    let unlocked_problem_ids = if is_newly_cleared {
        problems
            .iter()
            .filter(|problem| {
                problem.room_id == run.room_id
                    && problem.status == ProblemStatus::Locked
                    && problem.depends_on_problem_id == Some(target_problem_id)
            })
            .map(|problem| problem.problem_id)
            .collect()
    } else {
        Vec::new()
    };

    let required_count = problems
        .iter()
        .filter(|problem| problem.room_id == run.room_id && problem.is_required)
        .count();

    let cleared_count = problems
        .iter()
        .filter(|problem| {
            if problem.room_id != run.room_id || !problem.is_required {
                return false;
            }

            problem.status == ProblemStatus::Cleared
                || (is_newly_cleared && problem.problem_id == target_problem_id)
        })
        .count();

    let should_clear_run = required_count > 0 && cleared_count == required_count;

    let (run_status, run_cleared_at, progress_status) = if should_clear_run {
        (RunStatus::Cleared, Some(now), ProgressStatus::Cleared)
    } else {
        (RunStatus::Active, None, ProgressStatus::Active)
    };

    let progress = Progress {
        status: progress_status,
        cleared_count,
        required_count,
    };

    Ok(ClearProblemPlan {
        target_problem_status: ProblemStatus::Cleared,
        problem_cleared_at,
        unlocked_problem_ids,
        progress,
        run_status,
        run_cleared_at,
        elapsed,
    })
}

/// Converts an internally validated duration to the API elapsed milliseconds.
pub(crate) fn duration_to_elapsed_ms(elapsed: Duration) -> Result<u64, ClearProblemError> {
    u64::try_from(elapsed.num_milliseconds()).map_err(|_| ClearProblemError::InvalidElapsed)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        ActiveRunState, ClearProblemError, ProblemState, ProblemStatus, Progress, ProgressStatus,
        RunStatus, duration_to_elapsed_ms, plan_problem_clear,
    };

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn room_id() -> Uuid {
        id(1)
    }

    fn started_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 24, 10, 0, 0)
            .single()
            .expect("test start time should be valid")
    }

    fn decision_time() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 24, 10, 0, 5)
            .single()
            .expect("test decision time should be valid")
    }

    fn active_run() -> ActiveRunState {
        ActiveRunState {
            run_id: id(10),
            room_id: room_id(),
            started_at: started_at(),
        }
    }

    fn problem(
        problem_id: Uuid,
        room_id: Uuid,
        depends_on_problem_id: Option<Uuid>,
        is_required: bool,
        status: ProblemStatus,
    ) -> ProblemState {
        ProblemState {
            problem_id,
            room_id,
            depends_on_problem_id,
            is_required,
            status,
        }
    }

    fn problem_chain() -> Vec<ProblemState> {
        let first_problem_id = id(101);
        let second_problem_id = id(102);
        let third_problem_id = id(103);
        let final_problem_id = id(104);

        vec![
            problem(
                first_problem_id,
                room_id(),
                None,
                true,
                ProblemStatus::Available,
            ),
            problem(
                second_problem_id,
                room_id(),
                Some(first_problem_id),
                true,
                ProblemStatus::Locked,
            ),
            problem(
                third_problem_id,
                room_id(),
                Some(second_problem_id),
                true,
                ProblemStatus::Locked,
            ),
            problem(
                final_problem_id,
                room_id(),
                None,
                true,
                ProblemStatus::Available,
            ),
        ]
    }

    #[test]
    fn available_problem_is_cleared() {
        let run = active_run();
        let problems = problem_chain();

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("available problem should be cleared");

        assert_eq!(plan.target_problem_status, ProblemStatus::Cleared);
        assert_eq!(plan.problem_cleared_at, Some(decision_time()));
        assert_eq!(plan.elapsed, Duration::seconds(5));
    }

    #[test]
    fn direct_locked_dependent_is_unlocked() {
        let run = active_run();
        let problems = problem_chain();

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("available problem should be cleared");

        assert_eq!(plan.unlocked_problem_ids, vec![id(102)]);
    }

    #[test]
    fn indirect_dependent_remains_locked() {
        let run = active_run();
        let problems = problem_chain();

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("available problem should be cleared");

        assert!(!plan.unlocked_problem_ids.contains(&id(103)));
    }

    #[test]
    fn locked_problem_is_rejected() {
        let run = active_run();
        let problems = problem_chain();

        let error = plan_problem_clear(&run, &problems, id(102), decision_time())
            .expect_err("locked problem should be rejected");

        assert_eq!(error, ClearProblemError::ProblemLocked);
    }

    #[test]
    fn missing_problem_is_rejected() {
        let run = active_run();
        let problems = problem_chain();

        let error = plan_problem_clear(&run, &problems, id(999), decision_time())
            .expect_err("missing problem should be rejected");

        assert_eq!(error, ClearProblemError::ProblemNotFound);
    }

    #[test]
    fn problem_from_another_room_is_not_found() {
        let run = active_run();
        let mut problems = problem_chain();
        let other_room_problem_id = id(201);

        problems.push(problem(
            other_room_problem_id,
            id(2),
            None,
            true,
            ProblemStatus::Available,
        ));

        let error = plan_problem_clear(&run, &problems, other_room_problem_id, decision_time())
            .expect_err("problem from another room should be rejected");

        assert_eq!(error, ClearProblemError::ProblemNotFound);
    }

    #[test]
    fn already_cleared_problem_is_not_updated() {
        let run = active_run();
        let mut problems = problem_chain();
        problems[0].status = ProblemStatus::Cleared;

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("cleared problem should be handled successfully");

        assert_eq!(plan.target_problem_status, ProblemStatus::Cleared);
        assert_eq!(plan.problem_cleared_at, None);
        assert_eq!(
            plan.progress,
            Progress {
                status: ProgressStatus::Active,
                cleared_count: 1,
                required_count: 4,
            }
        );
        assert_eq!(plan.run_status, RunStatus::Active);
        assert_eq!(plan.elapsed, Duration::seconds(5));
    }

    #[test]
    fn already_cleared_problem_unlocks_nothing() {
        let run = active_run();
        let mut problems = problem_chain();
        problems[0].status = ProblemStatus::Cleared;

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("cleared problem should be handled successfully");

        assert!(plan.unlocked_problem_ids.is_empty());
    }

    #[test]
    fn progress_counts_only_required_problems() {
        let run = active_run();
        let mut problems = problem_chain();

        problems.push(problem(
            id(105),
            room_id(),
            None,
            false,
            ProblemStatus::Cleared,
        ));

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("available problem should be cleared");

        assert_eq!(
            plan.progress,
            Progress {
                status: ProgressStatus::Active,
                cleared_count: 1,
                required_count: 4,
            }
        );
        assert_eq!(plan.run_status, RunStatus::Active);
    }

    #[test]
    fn final_problem_cleared_first_keeps_run_active() {
        let run = active_run();
        let problems = problem_chain();

        let plan = plan_problem_clear(&run, &problems, id(104), decision_time())
            .expect("final problem should be available initially");

        assert_eq!(
            plan.progress,
            Progress {
                status: ProgressStatus::Active,
                cleared_count: 1,
                required_count: 4,
            }
        );
        assert_eq!(plan.run_status, RunStatus::Active);
        assert_eq!(plan.run_cleared_at, None);
        assert!(plan.unlocked_problem_ids.is_empty());
    }

    #[test]
    fn all_required_problems_clear_the_run() {
        let run = active_run();
        let mut problems = problem_chain();

        problems[0].status = ProblemStatus::Cleared;
        problems[1].status = ProblemStatus::Cleared;
        problems[2].status = ProblemStatus::Available;
        problems[3].status = ProblemStatus::Cleared;

        let plan = plan_problem_clear(&run, &problems, id(103), decision_time())
            .expect("last required problem should be cleared");

        assert_eq!(
            plan.progress,
            Progress {
                status: ProgressStatus::Cleared,
                cleared_count: 4,
                required_count: 4,
            }
        );
        assert_eq!(plan.run_status, RunStatus::Cleared);
        assert_eq!(plan.run_cleared_at, Some(decision_time()));
    }

    #[test]
    fn negative_elapsed_is_rejected() {
        let run = active_run();
        let problems = problem_chain();
        let time_before_start = started_at() - Duration::milliseconds(1);

        let error = plan_problem_clear(&run, &problems, id(101), time_before_start)
            .expect_err("negative elapsed duration should be rejected");

        assert_eq!(error, ClearProblemError::InvalidElapsed);
    }

    #[test]
    fn elapsed_milliseconds_accept_zero_and_positive_values() {
        assert_eq!(
            duration_to_elapsed_ms(Duration::zero()).expect("zero duration should be valid"),
            0
        );
        assert_eq!(
            duration_to_elapsed_ms(Duration::milliseconds(5_001))
                .expect("positive duration should be valid"),
            5_001
        );
    }

    #[test]
    fn elapsed_milliseconds_reject_negative_values() {
        let error = duration_to_elapsed_ms(Duration::milliseconds(-1))
            .expect_err("negative duration should be rejected");

        assert_eq!(error, ClearProblemError::InvalidElapsed);
    }

    #[test]
    fn already_cleared_problem_rechecks_run_completion() {
        let run = active_run();
        let mut problems = problem_chain();

        for problem in &mut problems {
            problem.status = ProblemStatus::Cleared;
        }

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("cleared problem should trigger state re-evaluation");

        assert_eq!(plan.problem_cleared_at, None);
        assert!(plan.unlocked_problem_ids.is_empty());
        assert_eq!(
            plan.progress,
            Progress {
                status: ProgressStatus::Cleared,
                cleared_count: 4,
                required_count: 4,
            }
        );
        assert_eq!(plan.run_status, RunStatus::Cleared);
        assert_eq!(plan.run_cleared_at, Some(decision_time()));
        assert_eq!(plan.elapsed, Duration::seconds(5));
    }

    #[test]
    fn only_locked_direct_dependents_are_unlocked() {
        let run = active_run();
        let mut problems = problem_chain();

        let available_direct_dependent_id = id(105);
        let cleared_direct_dependent_id = id(106);

        problems.push(problem(
            available_direct_dependent_id,
            room_id(),
            Some(id(101)),
            false,
            ProblemStatus::Available,
        ));

        problems.push(problem(
            cleared_direct_dependent_id,
            room_id(),
            Some(id(101)),
            false,
            ProblemStatus::Cleared,
        ));

        let plan = plan_problem_clear(&run, &problems, id(101), decision_time())
            .expect("available problem should be cleared");

        assert_eq!(plan.unlocked_problem_ids, vec![id(102)]);
        assert!(
            !plan
                .unlocked_problem_ids
                .contains(&available_direct_dependent_id)
        );
        assert!(
            !plan
                .unlocked_problem_ids
                .contains(&cleared_direct_dependent_id)
        );
    }
}
