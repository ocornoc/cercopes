use cercopes_dialog::goal::{GoalPursuer, RepeatGoalMove};
use super::*;

pub fn default_frame() -> Frame {
    Frame::new(|state| {
        state.person0.pushed_obligations
            .entry(dialog::greet::GREET)
            .or_insert(PushedObligationMetadata {
                urgency: 1_000_000,
                time_to_live: 5,
                times_pushed: 0,
            })
            .push();
        state.person1.pushed_obligations
            .entry(dialog::greet::GREET)
            .or_insert(PushedObligationMetadata {
                urgency: 1_000_000,
                time_to_live: 5,
                times_pushed: 0,
            })
            .push();
        state.goals.push(Goal::new(RepeatGoalMove::new(
            GoalMove {
                pursuer: GoalPursuer::Any,
                dialog_move: dialog::small_talk::MAKE_SMALL_TALK,
            },
            2,
        )));
    })
}

pub fn manager_frames() -> impl IntoIterator<Item = Frame> {
    [default_frame()]
}
