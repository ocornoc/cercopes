use super::*;

/// A specifier for who specifically can perform or execute a [goal move](GoalMove).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, From)]
pub enum GoalPursuer {
    /// Only a particular speaker can perform or execute the relevant [goal move](GoalMove).
    Speaker(Speaker),
    /// Both speakers are permissible for the relevant [goal move](GoalMove).
    Any,
}

impl GoalPursuer {
    /// Returns true if a given speaker can be considered a pursuer for a particular [goal
    /// move](GoalMove).
    pub fn agrees_with(self, speaker: Speaker) -> bool {
        if let GoalPursuer::Speaker(s) = self {
            s == speaker
        } else {
            true
        }
    }
}

/// A proposed move resulting from a goal.
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "D::DialogMove: Serialize",
    deserialize = "D::DialogMove: for<'a> Deserialize<'a>",
))]
pub struct GoalMove<D: DialogTrait> {
    /// The conversation participant meant to make the dialog move.
    pub pursuer: GoalPursuer,
    /// The dialog move being proposed by a goal.
    pub dialog_move: D::DialogMove,
}

impl<D: DialogTrait> Clone for GoalMove<D> {
    fn clone(&self) -> Self {
        Self {
            pursuer: self.pursuer.clone(),
            dialog_move: self.dialog_move.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.pursuer.clone_from(&source.pursuer);
        self.dialog_move.clone_from(&source.dialog_move);
    }
}

impl<D: DialogTrait> Debug for GoalMove<D>
where
    D::DialogMove: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("GoalMove")
            .field("pursuer", &self.pursuer)
            .field("dialog_move", &self.dialog_move)
            .finish()
    }
}

pub trait GoalState<D: DialogTrait>: DynClone + MaybeSendSync {
    /// Propose a next step for this goal.
    fn next_step(&self, state: &Conversation<D>) -> Option<GoalMove<D>>;

    /// Update the goal state with a move made in a conversation.
    ///
    /// The conversation state has all of the goals removed before being updated.
    fn made_move(&mut self, state: &Conversation<D>, history: &HistoricalMove<D>);

    /// Returns true if the goal is finished.
    fn is_satisfied(&self) -> bool;
}

dyn_clone::clone_trait_object!(<D: DialogTrait> GoalState<D>);

/// A goal plan that can propose new moves.
#[derive(From)]
pub struct Goal<D: DialogTrait> {
    ///
    pub state: Box<dyn GoalState<D>>,
}

impl<D: DialogTrait> Goal<D> {
    ///
    pub fn new<G: 'static + GoalState<D>>(g: G) -> Self {
        let g: Box<dyn GoalState<D>> = Box::new(g);
        Goal::from(g)
    }
}

impl<D: DialogTrait> Debug for Goal<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("Goal")
            .field("state", &format!("{:p}", &self.state))
            .finish()
    }
}

impl<D: DialogTrait> Clone for Goal<D> {
    fn clone(&self) -> Self {
        Goal {
            state: self.state.clone(),
        }
    }
}

#[cfg(feature = "send_sync")]
type FrameState<D> = Box<dyn Fn(&mut Conversation<D>) + Send + Sync>;

#[cfg(not(feature = "send_sync"))]
type FrameState<D> = Box<dyn Fn(&mut Conversation<D>)>;

/// A frame that can add new obligations and goals.
#[derive(From)]
pub struct Frame<D: DialogTrait> {
    ///
    pub state: FrameState<D>,
}

impl<D: DialogTrait> Frame<D> {
    ///
    pub fn new<F: 'static + Fn(&mut Conversation<D>) + MaybeSendSync>(state: F) -> Self {
        let state: FrameState<D> = Box::new(state);
        Frame::from(state)
    }
}

impl<D: DialogTrait> Debug for Frame<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("Frame")
            .field("state", &format!("{:p}", &self.state))
            .finish()
    }
}

/// Concatenate two goals, trying to perform the first and then the second.
pub struct ConcatGoals<D: DialogTrait, G1: GoalState<D>, G2: GoalState<D>> {
    ///
    pub g1: G1,
    ///
    pub g2: G2,
    _marker: PhantomData<D>,
}

impl<D: DialogTrait, G1: GoalState<D>, G2: GoalState<D>> ConcatGoals<D, G1, G2> {
    ///
    pub fn new(g1: G1, g2: G2) -> Self {
        Self {
            g1,
            g2,
            _marker: PhantomData,
        }
    }
}

impl<D, G1, G2> Clone for ConcatGoals<D, G1, G2>
where
    D: DialogTrait + MaybeSendSync,
    G1: GoalState<D> + Clone,
    G2: GoalState<D> + Clone,
{
    fn clone(&self) -> Self {
        ConcatGoals {
            g1: self.g1.clone(),
            g2: self.g2.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<D, G1, G2> GoalState<D> for ConcatGoals<D, G1, G2>
where
    D: DialogTrait + MaybeSendSync,
    G1: GoalState<D> + Clone,
    G2: GoalState<D> + Clone,
{
    fn next_step(&self, state: &Conversation<D>) -> Option<GoalMove<D>> {
        self.g1.next_step(state).or_else(|| self.g2.next_step(state))
    }

    fn made_move(&mut self, state: &Conversation<D>, history: &HistoricalMove<D>) {
        self.g1.made_move(state, history);
        self.g2.made_move(state, history);
    }

    fn is_satisfied(&self) -> bool {
        self.g1.is_satisfied() && self.g2.is_satisfied()
    }
}

/// Perform a sequence of moves.
pub struct GoalSequence<D: DialogTrait> {
    ///
    pub sequence: Vec<GoalMove<D>>,
}

impl<D: DialogTrait> Clone for GoalSequence<D>
where
    D::DialogMove: MaybeSendSync,
{
    fn clone(&self) -> Self {
        GoalSequence {
            sequence: self.sequence.clone(),
        }
    }
}

impl<D: DialogTrait> GoalState<D> for GoalSequence<D>
where
    D::DialogMove: MaybeSendSync,
{
    fn next_step(&self, _state: &Conversation<D>) -> Option<GoalMove<D>> {
        self.sequence.first().cloned()
    }

    fn made_move(&mut self, _state: &Conversation<D>, history: &HistoricalMove<D>) {
        for i in (0..self.sequence.len()).rev() {
            let goal_move = &self.sequence[i];
            if goal_move.pursuer.agrees_with(history.speaker)
                && history.was_move_satisfied(&goal_move.dialog_move)
            {
                self.sequence.remove(i);
            }
        }
    }

    fn is_satisfied(&self) -> bool {
        self.sequence.is_empty()
    }
}

/// Perform a sequence of moves.
///
/// This has a special case: if the last move is made (even if there are other moves remaining),
/// this eagerly satisfies and empties the sequence of moves.
pub struct GoalEagerSequence<D: DialogTrait> {
    ///
    pub sequence: Vec<GoalMove<D>>,
}

impl<D: DialogTrait> Clone for GoalEagerSequence<D> {
    fn clone(&self) -> Self {
        GoalEagerSequence {
            sequence: self.sequence.clone()
        }
    }
}

impl<D: DialogTrait> GoalState<D> for GoalEagerSequence<D>
where
    D::DialogMove: MaybeSendSync,
{
    fn next_step(&self, _state: &Conversation<D>) -> Option<GoalMove<D>> {
        self.sequence.first().cloned()
    }

    fn made_move(&mut self, _state: &Conversation<D>, history: &HistoricalMove<D>) {
        if let Some(goal_move) = self.sequence.last() {
            if goal_move.pursuer.agrees_with(history.speaker)
                &&  history.was_move_satisfied(&goal_move.dialog_move)
            {
                self.sequence.clear();
                return;
            }
        }

        for i in (0..self.sequence.len()).rev() {
            let goal_move = &self.sequence[i];
            if goal_move.pursuer.agrees_with(history.speaker)
                && history.was_move_satisfied(&goal_move.dialog_move)
            {
                self.sequence.remove(i);
            }
        }
    }

    fn is_satisfied(&self) -> bool {
        self.sequence.is_empty()
    }
}

/// Performs a single goal move.
struct PerformGoalMove<D: DialogTrait> {
    pub goal_move: GoalMove<D>,
    pub satisfied: bool,
}

impl<D: DialogTrait> PerformGoalMove<D> {
    pub fn new(goal_move: GoalMove<D>) -> Self {
        PerformGoalMove {
            goal_move,
            satisfied: false,
        }
    }
}

impl<D: DialogTrait> From<GoalMove<D>> for PerformGoalMove<D> {
    fn from(goal_move: GoalMove<D>) -> Self {
        PerformGoalMove::new(goal_move)
    }
}

impl<D: DialogTrait> Clone for PerformGoalMove<D> {
    fn clone(&self) -> Self {
        PerformGoalMove {
            goal_move: self.goal_move.clone(),
            satisfied: self.satisfied.clone(),
        }
    }
}

impl<D: DialogTrait> GoalState<D> for PerformGoalMove<D>
where
    D::DialogMove: MaybeSendSync,
 {
    fn next_step(&self, _: &Conversation<D>) -> Option<GoalMove<D>> {
        if self.is_satisfied() {
            None
        } else {
            Some(self.goal_move.clone())
        }
    }

    fn made_move(&mut self, _: &Conversation<D>, history: &HistoricalMove<D>) {
        if self.goal_move.pursuer.agrees_with(history.speaker) {
            self.satisfied |= history.was_move_satisfied(&self.goal_move.dialog_move);
        }
    }

    fn is_satisfied(&self) -> bool {
        self.satisfied
    }
}

/// Repeat a goal move until its been satisfied at least `max_reps` times
pub struct RepeatGoalMove<D: DialogTrait> {
    /// The goal move to repeat.
    pub goal_move: GoalMove<D>,
    /// The number of times the goal move has already been performed.
    pub reps: usize,
    /// The number of times to perform the goal move until it is finished.
    pub max_reps: usize,
}

impl<D: DialogTrait> RepeatGoalMove<D> {
    ///
    pub fn new(goal_move: GoalMove<D>, max_reps: usize) -> Self {
        RepeatGoalMove {
            goal_move,
            reps: 0,
            max_reps,
        }
    }
}

impl<D: DialogTrait> Clone for RepeatGoalMove<D> {
    fn clone(&self) -> Self {
        RepeatGoalMove {
            goal_move: self.goal_move.clone(),
            reps: self.reps.clone(),
            max_reps: self.max_reps.clone(),
        }
    }
}

impl<D: DialogTrait> GoalState<D> for RepeatGoalMove<D>
where
    <D as DialogTrait>::DialogMove: MaybeSendSync,
{
    fn next_step(&self, _: &Conversation<D>) -> Option<GoalMove<D>> {
        if self.is_satisfied() {
            None
        } else {
            Some(self.goal_move.clone())
        }
    }

    fn made_move(&mut self, _: &Conversation<D>, history: &HistoricalMove<D>) {
        if
            self.goal_move.pursuer.agrees_with(history.speaker)
            && history.was_move_satisfied(&self.goal_move.dialog_move)
        {
            self.reps += 1;
        }
    }

    fn is_satisfied(&self) -> bool {
        self.reps >= self.max_reps
    }
}
