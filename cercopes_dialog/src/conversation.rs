use std::ops::Not;
use super::*;

/// Data regarding how many times a particular [topic](DialogTrait::Topic) was introduced or
/// addressed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicMetadata {
    /// How many times this topic has been introduced by this speaker in this conversation.
    pub times_introduced: u32,
    /// How many times this topic has been addressed by this speaker in this conversation.
    pub times_addressed: u32,
}

impl TopicMetadata {
    /// Marks the [topic](DialogTrait::Topic) as introduced in the metadata.
    pub fn introduce(&mut self) {
        self.times_introduced += 1;
    }

    /// Marks the [topic](DialogTrait::Topic) as addressed in the metadata.
    pub fn address(&mut self) {
        self.times_addressed += 1;
    }
}

/// Data regarding an obligation pushed by a [dialog move](HistoricalMove).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushedObligationMetadata {
    /// How 'urgent' the pushed obligation is.
    pub urgency: i32,
    /// How long this pushed obligation is valid or relevant
    ///
    /// Measured in moves. So, for example, if the TTL of an obligation is 3, the speaker has up to
    /// and including 3 moves taken in the conversation to address this until the obligation simply
    /// expires.
    pub time_to_live: u32,
    /// How many times this obligation has been pushed since it was last fulfilled.
    pub times_pushed: u32,
}

impl PushedObligationMetadata {
    /// Mark this obligation as pushed in the metadata.
    pub fn push(&mut self) {
        self.times_pushed += 1;
    }

    fn timestep(&mut self) -> bool {
        if let Some(time_to_live) = self.time_to_live.checked_sub(1) {
            self.time_to_live = time_to_live;
            false
        } else {
            true
        }
    }
}

/// A [conversation](Conversation) participant's current state.
///
/// This tracks metadata for [topics](TopicMetadata) and [pushed
/// obligations](PushedObligationMetadata), as well as [character state](DialogTrait::Character).
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "
        D::Topic: Serialize,
        D::DialogMove: Serialize,
        D::Character: Serialize,
    ",
    deserialize = "
        D::Topic: for<'a> Deserialize<'a>,
        D::DialogMove: for<'a> Deserialize<'a>,
        D::Character: for<'a> Deserialize<'a>,
    ",
))]
pub struct ParticipantState<D: DialogTrait> {
    /// Topics that have been addressed or introduced by this participant.
    pub topics: AHashMap<D::Topic, TopicMetadata>,
    /// Obligations that have been pushed to this participant.
    pub pushed_obligations: AHashMap<D::DialogMove, PushedObligationMetadata>,
    /// The participant whose state this represents.
    pub character: D::Character,
}

impl<D: DialogTrait> ParticipantState<D> {
    /// Create a representation for participant state for a particular character.
    pub fn new(character: D::Character) -> Self {
        ParticipantState {
            topics: Default::default(),
            pushed_obligations: Default::default(),
            character,
        }
    }

    fn timestep(&mut self) {
        let mut remove = Vec::new();
        for (dialog_move, obligation) in self.pushed_obligations.iter_mut() {
            if obligation.timestep() {
                remove.push(dialog_move.clone());
            }
        }
        for dialog_move in remove {
            self.pushed_obligations.remove(&dialog_move);
        }
    }

    fn merge_historical_obligations(&mut self, hobl: &HistoricalObligations<D>) {
        for addressed in hobl.addressed.iter() {
            self.pushed_obligations.remove(addressed);
        }

        for (dialog_move, meta) in hobl.pushed.iter() {
            let original_meta = self.pushed_obligations
                .entry(dialog_move.clone())
                .or_insert(PushedObligationMetadata {
                    urgency: i32::MIN,
                    time_to_live: 0,
                    times_pushed: 0,
                });
            original_meta.times_pushed += meta.times_pushed;
            original_meta.urgency = original_meta.urgency.max(meta.urgency);
            original_meta.time_to_live = original_meta.time_to_live.max(meta.time_to_live);
        }
    }
}

impl<D: DialogTrait> Debug for ParticipantState<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("ParticipantMetadata")
            .field("topics_addressed", &self.topics)
            .field("pushed_obligations", &self.pushed_obligations)
            .finish()
    }
}

impl<D: DialogTrait> Clone for ParticipantState<D>
where
    D::Character: Clone,
{
    fn clone(&self) -> Self {
        ParticipantState {
            topics: self.topics.clone(),
            pushed_obligations: self.pushed_obligations.clone(),
            character: self.character.clone(),
        }
    }
}

/// Identifiers for a speaker in a [conversation](Conversation).
///
/// The assignment of being [person 0](Speaker::Person0) vs [person 1](Speaker::Person1) is
/// completely arbitrary and is left to the users of the library to assign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Speaker {
    ///
    Person0,
    ///
    Person1,
}

impl Not for Speaker {
    type Output = Speaker;

    fn not(self) -> Self::Output {
        match self {
            Speaker::Person0 => Speaker::Person1,
            Speaker::Person1 => Speaker::Person0,
        }
    }
}

/// Data regarding pushed and addressed [obligations](DialogTrait::DialogMove) in a [historical
/// move](HistoricalMove).
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "D::DialogMove: Serialize",
    deserialize = "D::DialogMove: for<'a> Deserialize<'a>",
))]
pub struct HistoricalObligations<D: DialogTrait> {
    /// The new obligations pushed to this participant.
    pub pushed: AHashMap<D::DialogMove, PushedObligationMetadata>,
    /// The obligations addressed for this participant.
    pub addressed: AHashSet<D::DialogMove>,
}

impl<D: DialogTrait> HistoricalObligations<D> {
    /// [Push a new obligation](PushedObligationMetadata) to be addressed.
    pub fn push(&mut self, dialog_move: D::DialogMove, urgency: i32, time_to_live: u32) {
        let obligation = self.pushed
            .entry(dialog_move)
            .or_insert(PushedObligationMetadata {
                urgency,
                time_to_live,
                times_pushed: 0,
            });
        obligation.push();
        obligation.urgency = obligation.urgency.max(urgency);
        obligation.time_to_live = obligation.time_to_live.max(time_to_live);
    }
}

impl<D: DialogTrait> Debug for HistoricalObligations<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("HistoricalObligations")
            .field("pushed", &self.pushed)
            .field("addressed", &self.addressed)
            .finish()
    }
}

impl<D: DialogTrait> Default for HistoricalObligations<D> {
    fn default() -> Self {
        Self {
            pushed: Default::default(),
            addressed: Default::default(),
        }
    }
}

impl<D: DialogTrait> Clone for HistoricalObligations<D> {
    fn clone(&self) -> Self {
        HistoricalObligations {
            pushed: self.pushed.clone(),
            addressed: self.addressed.clone(),
        }
    }
}

/// A structure keeping track of whether a [topic](DialogTrait::Topic) has been introduced and
/// whether it has been addressed.
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "D::Topic: Serialize",
    deserialize = "D::Topic: for<'a> Deserialize<'a>",
))]
pub struct TopicState<D: DialogTrait> {
    /// The topics that have been introduced.
    pub introduced: AHashSet<D::Topic>,
    /// The topics that have been addressed.
    pub addressed: AHashSet<D::Topic>,
}

impl<D: DialogTrait> TopicState<D> {
    /// Returns true if the [topic](DialogTrait::Topic) has yet to be addressed.
    #[inline]
    pub fn can_be_addressed(&self, topic: &D::Topic) -> bool {
        !self.addressed.contains(topic)
    }

    /// Returns true if the [topic](DialogTrait::Topic) has been introduced but hasn't been
    /// addressed.
    #[inline] 
    pub fn needs_addressing(&self, topic: &D::Topic) -> bool {
        self.introduced.contains(topic) && self.can_be_addressed(topic)
    }

    /// Returns true if the [topic](DialogTrait::Topic) has neither been introduced nor has been
    /// addressed.
    #[inline]
    pub fn can_be_introduced(&self, topic: &D::Topic) -> bool {
        !self.introduced.contains(topic) && self.can_be_addressed(topic)
    }
}

impl<D: DialogTrait> Debug for TopicState<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("TopicState")
            .field("introduced", &self.introduced)
            .field("addressed", &self.addressed)
            .finish()
    }
}

impl<D: DialogTrait> Default for TopicState<D> {
    fn default() -> Self {
        Self {
            introduced: Default::default(),
            addressed: Default::default(),
        }
    }
}

impl<D: DialogTrait> Clone for TopicState<D> {
    fn clone(&self) -> Self {
        TopicState {
            introduced: self.introduced.clone(),
            addressed: self.addressed.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "D::Topic: Serialize, D::DialogMove: Serialize",
    deserialize = "D::Topic: for<'a> Deserialize<'a>, D::DialogMove: for<'a> Deserialize<'a>",
))]
pub struct HistoricalMove<D: DialogTrait> {
    /// An utterance is the exact words spoken as part of a piece of dialog.
    pub utterance: String,
    /// The participant who performed this move.
    pub speaker: Speaker,
    /// The obligation data regarding Person 0.
    pub person0_obligations: HistoricalObligations<D>,
    /// The obligation data regarding Person 1.
    pub person1_obligations: HistoricalObligations<D>,
    /// The data regarding the [topics](DialogTrait::Topic) spoken in this move.
    pub topic_state: TopicState<D>,
}

impl<D: DialogTrait> HistoricalMove<D> {
    /// Get an iterator over the [obligations](DialogTrait::DialogMove) addressed by the speakers
    /// in this move, without duplicates.
    pub fn all_addressed_obligations(&self) -> impl Iterator<Item = &D::DialogMove> + Clone {
        self.person0_obligations.addressed.union(&self.person1_obligations.addressed)
    }

    /// Query whether a [dialog move](DialogTrait::DialogMove) was satisfied by this move.
    pub fn was_move_satisfied(&self, dialog_move: &D::DialogMove) -> bool {
        self.person0_obligations.addressed.contains(dialog_move)
        || self.person1_obligations.addressed.contains(dialog_move)
    }

    /// Get the [historical obligations](HistoricalObligations) for a particular speaker.
    #[inline]
    pub fn get_speaker_obligations(&mut self, speaker: Speaker) -> &mut HistoricalObligations<D> {
        match speaker {
            Speaker::Person0 => &mut self.person0_obligations,
            Speaker::Person1 => &mut self.person1_obligations,
        }
    }

    /// Get the [historical obligations](HistoricalObligations) for the speaker who made this move.
    #[inline]
    pub fn get_my_obligations(&mut self) -> &mut HistoricalObligations<D> {
        self.get_speaker_obligations(self.speaker)
    }

    /// Get the [historical obligations](HistoricalObligations) for the speaker who *didn't* make
    /// this move.
    #[inline]
    pub fn get_others_obligations(&mut self) -> &mut HistoricalObligations<D> {
        self.get_speaker_obligations(!self.speaker)
    }
}

impl<D: DialogTrait> Debug for HistoricalMove<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("HistoricalMove")
            .field("words", &self.utterance)
            .field("speaker", &self.speaker)
            .field("person0_obligations", &self.person0_obligations)
            .field("person1_obligations", &self.person1_obligations)
            .field("topic_state", &self.topic_state)
            .finish()
    }
}

impl<D: DialogTrait> Display for HistoricalMove<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&self.utterance)
    }
}

impl<D: DialogTrait> Clone for HistoricalMove<D> {
    fn clone(&self) -> Self {
        HistoricalMove {
            utterance: self.utterance.clone(),
            speaker: self.speaker.clone(),
            person0_obligations: self.person0_obligations.clone(),
            person1_obligations: self.person1_obligations.clone(),
            topic_state: self.topic_state.clone(),
        }
    }
}

pub struct Conversation<D: DialogTrait> {
    /// The participant that started the conversation.
    pub initiator: Speaker,
    /// The participant that will next speak.
    pub speaker: Speaker,
    /// The participant state for person 0.
    pub person0: ParticipantState<D>,
    /// The participant state for person 1.
    pub person1: ParticipantState<D>,
    /// The state of all introduced and addressed topics.
    pub topic_state: TopicState<D>,
    /// The history of all moves taken in this conversation.
    pub history: Vec<HistoricalMove<D>>,
    /// All of the goals for this conversation.
    pub goals: Vec<Goal<D>>,
    /// Is the conversation over?
    pub done: bool,
    /// The chance for the conversation to continue with the default dialog move when there is a
    /// lull in the conversation.
    pub lull_continue_chance: Bernoulli,
}

impl<D: DialogTrait> Conversation<D> {
    pub(crate) fn new(
        initiator: Speaker,
        frames: &[Frame<D>],
        lull_continue_chance: Bernoulli,
        person0: D::Character,
        person1: D::Character,
    ) -> Self {
        let mut conversation = Conversation {
            initiator,
            speaker: initiator,
            person0: ParticipantState::new(person0),
            person1: ParticipantState::new(person1),
            topic_state: Default::default(),
            history: Default::default(),
            goals: Default::default(),
            done: false,
            lull_continue_chance,
        };
        for frame in frames {
            (frame.state)(&mut conversation);
        }
        conversation
    }

    pub(crate) fn timestep(&mut self) {
        self.person0.timestep();
        self.person1.timestep();
    }

    fn introduce_topic(&mut self, topic: &D::Topic) {
        self.topic_state.introduced.insert(topic.clone());
        if self.speaker == Speaker::Person0 {
            &mut self.person0
        } else {
            &mut self.person1
        }.topics.entry(topic.clone()).or_default().introduce();
    }

    fn address_topic(&mut self, topic: &D::Topic) {
        self.topic_state.addressed.insert(topic.clone());
        if self.speaker == Speaker::Person0 {
            &mut self.person0
        } else {
            &mut self.person1
        }.topics.entry(topic.clone()).or_default().address();
    }

    pub(crate) fn update_for_move(&mut self, hmove: HistoricalMove<D>) {
        let mut goals = std::mem::take(&mut self.goals);
        for goal in goals.iter_mut() {
            goal.state.made_move(&self, &hmove);
        }
        self.goals = goals;
        for topic in hmove.topic_state.introduced.iter() {
            self.introduce_topic(topic);
        }
        for topic in hmove.topic_state.addressed.iter() {
            self.address_topic(topic);
        }
        self.person0.merge_historical_obligations(&hmove.person0_obligations);
        self.person1.merge_historical_obligations(&hmove.person1_obligations);
        self.history.push(hmove);
    }

    pub(crate) fn get_next_speaker_topics(&self, rng: &mut Rng) -> Vec<D::Topic> {
        let mut topics = self.topic_state.introduced
            .difference(&self.topic_state.addressed)
            .cloned()
            .collect::<Vec<_>>();
        if topics.len() > 1 {
            topics.shuffle(rng);
        }
        topics
    }

    pub(crate) fn get_next_speaker_moves(&self, rng: &mut Rng) -> Vec<D::DialogMove> {
        let mut pushed_obligations = if self.speaker == Speaker::Person0 {
            &self.person0
        } else {
            &self.person1
        }.pushed_obligations.iter().collect::<Vec<_>>();
        pushed_obligations.sort_unstable_by(|(_, l), (_, r)|
            l.urgency.cmp(&r.urgency).then(l.time_to_live.cmp(&r.time_to_live).reverse())
        );
        let mut dialog_moves = Vec::with_capacity(pushed_obligations.len());
        dialog_moves.extend(self.goals
            .iter()
            .map(|goal| goal.state.next_step(self))
            .flatten()
            .filter(|goal| goal.pursuer.agrees_with(self.speaker))
            .map(|goal| goal.dialog_move)
        );
        if dialog_moves.len() > 1 {
            dialog_moves.shuffle(rng);
        }
        dialog_moves.extend(pushed_obligations
            .into_iter()
            .map(|(d, _)| d.clone())
        );
        dialog_moves
    }

    /// Get a reference to the [participant state](ParticipantState) for a particular speaker.
    #[inline]
    pub fn get_speaker_state(&self, speaker: Speaker) -> &ParticipantState<D> {
        match speaker {
            Speaker::Person0 => &self.person0,
            Speaker::Person1 => &self.person1,
        }
    }

    /// Get an exclusive reference to the [participant state](ParticipantState) for a particular
    /// speaker.
    #[inline]
    pub fn get_speaker_state_mut(&mut self, speaker: Speaker) -> &mut ParticipantState<D> {
        match speaker {
            Speaker::Person0 => &mut self.person0,
            Speaker::Person1 => &mut self.person1,
        }
    }

    /// Get a reference to the [participant state](ParticipantState) for the speaker who will next
    /// speak.
    #[inline]
    pub fn get_my_state(&self) -> &ParticipantState<D> {
        self.get_speaker_state(self.speaker)
    }

    /// Get an exclusive reference to the [participant state](ParticipantState) for the speaker who
    /// will next speak.
    #[inline]
    pub fn get_my_state_mut(&mut self) -> &mut ParticipantState<D> {
        self.get_speaker_state_mut(self.speaker)
    }

    /// Get a reference to the [participant state](ParticipantState) for the speaker who will next
    /// listen.
    #[inline]
    pub fn get_others_state(&self) -> &ParticipantState<D> {
        self.get_speaker_state(!self.speaker)
    }

    /// Get an exclusive reference to the [participant state](ParticipantState) for the speaker who
    /// will next listen.
    #[inline]
    pub fn get_others_state_mut(&mut self) -> &mut ParticipantState<D> {
        self.get_speaker_state_mut(!self.speaker)
    }
}

impl<D: DialogTrait> Debug for Conversation<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("ConversationState")
            .field("initiator", &self.initiator)
            .field("speaker", &self.speaker)
            .field("person0", &self.person0)
            .field("person1", &self.person1)
            .field("topic_state", &self.topic_state)
            .field("history", &self.history)
            .field("goals", &self.goals)
            .field("done", &self.done)
            .field("lull_continue_chance", &self.lull_continue_chance)
            .finish()
    }
}

impl<D: DialogTrait> Clone for Conversation<D>
where
    ParticipantState<D>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            initiator: self.initiator.clone(),
            speaker: self.speaker.clone(),
            person0: self.person0.clone(),
            person1: self.person1.clone(),
            topic_state: self.topic_state.clone(),
            history: self.history.clone(),
            goals: self.goals.clone(),
            done: self.done.clone(),
            lull_continue_chance: self.lull_continue_chance.clone(),
        }
    }
}
