use crate::input::CheckpointCursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerClass {
    CheckpointReady,
    Heartbeat,
    RepeatedFailure,
    ManualReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedulerPolicy {
    pub checkpoint_cooldown: usize,
    pub heartbeat_interval: usize,
    pub warning_debounce: usize,
    pub repeated_failure_threshold: usize,
}

impl Default for SchedulerPolicy {
    fn default() -> Self {
        Self {
            checkpoint_cooldown: 2,
            heartbeat_interval: 3,
            warning_debounce: 2,
            repeated_failure_threshold: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchedulerState {
    pub last_evaluated: Option<CheckpointCursor>,
    pub checkpoints_since_last_evaluation: usize,
    pub last_visible_warning_fingerprint: Option<String>,
    pub checkpoints_since_last_visible_warning: usize,
    pub consecutive_flagged_checkpoints: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationDecision {
    pub evaluate: bool,
    pub visible_warning_allowed: bool,
    pub reason: DecisionReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionReason {
    InitialCheckpoint,
    CooldownSatisfied,
    HeartbeatIntervalReached,
    RepeatedFailureFastPath,
    ManualReview,
    CooldownDeferred,
    WarningDebounced,
}

#[derive(Debug, Clone)]
pub struct ReplayScheduler {
    policy: SchedulerPolicy,
    state: SchedulerState,
}

impl ReplayScheduler {
    pub fn new(policy: SchedulerPolicy) -> Self {
        Self {
            policy,
            state: SchedulerState::default(),
        }
    }

    pub fn state(&self) -> &SchedulerState {
        &self.state
    }

    pub fn observe(
        &mut self,
        cursor: CheckpointCursor,
        trigger: TriggerClass,
        checkpoint_flagged: bool,
        warning_fingerprint: Option<&str>,
    ) -> EvaluationDecision {
        let next_eval_gap = self.state.checkpoints_since_last_evaluation + 1;
        let next_warning_gap = self.state.checkpoints_since_last_visible_warning + 1;
        let consecutive_flagged = if checkpoint_flagged {
            self.state.consecutive_flagged_checkpoints + 1
        } else {
            0
        };

        let (evaluate, reason) = if matches!(trigger, TriggerClass::ManualReview) {
            (true, DecisionReason::ManualReview)
        } else if self.state.last_evaluated.is_none() {
            (true, DecisionReason::InitialCheckpoint)
        } else if matches!(trigger, TriggerClass::RepeatedFailure)
            && consecutive_flagged >= self.policy.repeated_failure_threshold
        {
            (true, DecisionReason::RepeatedFailureFastPath)
        } else if matches!(trigger, TriggerClass::Heartbeat)
            && next_eval_gap >= self.policy.heartbeat_interval
        {
            (true, DecisionReason::HeartbeatIntervalReached)
        } else if next_eval_gap >= self.policy.checkpoint_cooldown {
            (true, DecisionReason::CooldownSatisfied)
        } else {
            (false, DecisionReason::CooldownDeferred)
        };

        let visible_warning_allowed = if checkpoint_flagged {
            let duplicate_warning = warning_fingerprint
                .zip(self.state.last_visible_warning_fingerprint.as_deref())
                .is_some_and(|(current, last)| {
                    current == last && next_warning_gap <= self.policy.warning_debounce
                });
            !duplicate_warning
        } else {
            false
        };

        self.state.consecutive_flagged_checkpoints = consecutive_flagged;
        self.state.checkpoints_since_last_evaluation = if evaluate { 0 } else { next_eval_gap };
        if evaluate {
            self.state.last_evaluated = Some(cursor);
        }

        if checkpoint_flagged && visible_warning_allowed {
            self.state.last_visible_warning_fingerprint =
                warning_fingerprint.map(ToOwned::to_owned);
            self.state.checkpoints_since_last_visible_warning = 0;
        } else {
            self.state.checkpoints_since_last_visible_warning = next_warning_gap;
        }

        let reason = if checkpoint_flagged && !visible_warning_allowed {
            DecisionReason::WarningDebounced
        } else {
            reason
        };

        EvaluationDecision {
            evaluate,
            visible_warning_allowed,
            reason,
        }
    }
}
