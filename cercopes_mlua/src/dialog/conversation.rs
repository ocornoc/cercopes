use rand::distributions::Bernoulli;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, From, Into)]
pub(crate) struct LuaSpeaker(pub Speaker);

impl ToLua<'_> for LuaSpeaker {
    fn to_lua(self, _: & Lua) -> LuaResult<LuaValue> {
        Ok(LuaValue::Boolean(self.0 == Speaker::Person0))
    }
}

impl FromLua<'_> for LuaSpeaker {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        Ok(if bool::from_lua(lua_value, lua)? {
            Speaker::Person0
        } else {
            Speaker::Person1
        }.into())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LuaTopicMetadata {
    pub topic: Topic,
    pub participant: LuaParticipantState,
}

impl LuaTopicMetadata {
    pub fn borrow(&self) -> Ref<TopicMetadata> {
        {
            self.participant.borrow_mut().topics.entry(self.topic.clone()).or_default();
        }
        Ref::map(self.participant.borrow(), |participant| {
            &participant.topics[&self.topic]
        })
    }

    pub fn borrow_mut(&self) -> RefMut<TopicMetadata> {
        RefMut::map(self.participant.borrow_mut(), |participant| {
            participant.topics.entry(self.topic.clone()).or_default()
        })
    }
}

impl UserData for LuaTopicMetadata {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("participant", |_, metadata| {
            Ok(metadata.participant.clone())
        });
        fields.add_field_method_get("times_introduced", |_, metadata| {
            Ok(metadata.borrow().times_introduced)
        });
        fields.add_field_method_set("times_introduced", |_, metadata, times_introduced| {
            metadata.borrow_mut().times_introduced = times_introduced;
            Ok(())
        });
        fields.add_field_method_get("times_addressed", |_, metadata| {
            Ok(metadata.borrow().times_addressed)
        });
        fields.add_field_method_set("times_addressed", |_, metadata, times_addressed| {
            metadata.borrow_mut().times_addressed = times_addressed;
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("introduce", |_, metadata, ()| {
            metadata.borrow_mut().introduce();
            Ok(())
        });
        methods.add_method("address", |_, metadata, ()| {
            metadata.borrow_mut().address();
            Ok(())
        });
    }
}

#[derive(Debug, Clone, From)]
pub(crate) enum LuaPOMLocation {
    Participant(LuaParticipantState),
    HistoricalObligations(LuaHistoricalObligations),
}

#[derive(Debug, Clone)]
pub(crate) struct LuaPOMetaData {
    pub pushed_obligation: DialogMove,
    pub location: LuaPOMLocation,
}

impl LuaPOMetaData {
    pub fn borrow(&self) -> Option<Ref<PushedObligationMetadata>> {
        match &self.location {
            LuaPOMLocation::Participant(participant) => {
                let participant = participant.borrow();
                if participant.pushed_obligations.contains_key(&self.pushed_obligation) {
                    Some(Ref::map(
                        participant,
                        |participant| &participant.pushed_obligations[&self.pushed_obligation],
                    ))
                } else {
                    None
                }
            },
            LuaPOMLocation::HistoricalObligations(hobls) => {
                let hobls = hobls.borrow();
                if hobls.pushed.contains_key(&self.pushed_obligation) {
                    Some(Ref::map(
                        hobls,
                        |hobls| &hobls.pushed[&self.pushed_obligation],
                    ))
                } else {
                    None
                }
            },
        }
    }

    pub fn borrow_mut(&self) -> Option<RefMut<PushedObligationMetadata>> {
        match &self.location {
            LuaPOMLocation::Participant(participant) => {
                let participant = participant.borrow_mut();
                if participant.pushed_obligations.contains_key(&self.pushed_obligation) {
                    Some(RefMut::map(
                        participant,
                        |participant| participant.pushed_obligations
                            .get_mut(&self.pushed_obligation)
                            .unwrap(),
                    ))
                } else {
                    None
                }
            },
            LuaPOMLocation::HistoricalObligations(hobls) => {
                let hobls = hobls.borrow_mut();
                if hobls.pushed.contains_key(&self.pushed_obligation) {
                    Some(RefMut::map(
                        hobls,
                        |hobls| hobls.pushed.get_mut(&self.pushed_obligation).unwrap(),
                    ))
                } else {
                    None
                }
            },
        }
    }
}

impl UserData for LuaPOMetaData {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("participant", |_, metadata| {
            if let LuaPOMLocation::Participant(ref participant) = metadata.location {
                Ok(Some(participant.clone()))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("obligations", |_, metadata| {
            if let LuaPOMLocation::HistoricalObligations(ref obls) = metadata.location {
                Ok(Some(obls.clone()))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("obligation", |_, metadata| {
            Ok(metadata.pushed_obligation.clone())
        });
        fields.add_field_method_get("urgency", |_, metadata| {
            Ok(metadata.borrow().map(|metadata| metadata.urgency))
        });
        fields.add_field_method_set("urgency", |_, metadata, urgency| {
            if let Some(mut metadata) = metadata.borrow_mut() {
                metadata.urgency = urgency;
            }
            Ok(())
        });
        fields.add_field_method_get("time_to_live", |_, metadata| {
            Ok(metadata.borrow().map(|metadata| metadata.time_to_live))
        });
        fields.add_field_method_set("time_to_live", |_, metadata, time_to_live| {
            if let Some(mut metadata) = metadata.borrow_mut() {
                metadata.time_to_live = time_to_live;
            }
            Ok(())
        });
        fields.add_field_method_get("times_pushed", |_, metadata| {
            Ok(metadata.borrow().map(|metadata| metadata.times_pushed))
        });
        fields.add_field_method_set("times_pushed", |_, metadata, times_pushed| {
            if let Some(mut metadata) = metadata.borrow_mut() {
                metadata.times_pushed = times_pushed;
            }
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("push", |_, methods, ()| {
            if let Some(mut metadata) = methods.borrow_mut() {
                metadata.push();
            }

            Ok(())
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LuaParticipantState {
    pub person0: bool,
    pub conversation: LuaConversation,
}

impl LuaParticipantState {
    pub fn borrow(&self) -> Ref<ParticipantState<DialogTypes>> {
        Ref::map(self.conversation.borrow(), |conversation| if self.person0 {
            &conversation.person0
        } else {
            &conversation.person1
        })
    }

    pub fn borrow_mut(&self) -> RefMut<ParticipantState<DialogTypes>> {
        RefMut::map(self.conversation.borrow_mut(), |conversation| if self.person0 {
            &mut conversation.person0
        } else {
            &mut conversation.person1
        })
    }
}

impl UserData for LuaParticipantState {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("conversation", |_, participant| {
            Ok(participant.conversation.clone())
        });
        fields.add_field_method_get("character", |_, participant| {
            Ok(participant.borrow().character.clone())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("insert_topic", |_, participant, topic| {
            participant.borrow_mut().topics.entry(topic).or_default();
            Ok(())
        });
        methods.add_method("remove_topic", |_, participant, topic| {
            participant.borrow_mut().topics.remove(&topic);
            Ok(())
        });
        methods.add_method("get_topic", |_, participant, topic| {
            if participant.borrow().topics.contains_key(&topic) {
                Ok(Some(LuaTopicMetadata {
                    topic,
                    participant: participant.clone(),
                }))
            } else {
                Ok(None)
            }
        });
        methods.add_method("topics_len", |_, participant, ()| {
            Ok(participant.borrow().topics.len())
        });
        methods.add_method("iter_topics", |lua, participant, ()| {
            let mut visited = AHashSet::new();
            let iter = lua.create_function_mut(move |lua, (participant, _): (Self, LuaValue)| {
                for topic in participant.borrow().topics.keys() {
                    if !visited.contains(topic) {
                        visited.insert(topic.clone());
                        return (topic.clone(), LuaTopicMetadata {
                            topic: topic.clone(),
                            participant: participant.clone(),
                        }).to_lua_multi(lua);
                    }
                }

                Ok(MultiValue::new())
            })?;
            Ok((iter, participant.clone(), Nil))
        });
        methods.add_method(
            "insert_obligation",
            |_, participant, (dialog_move, data): (_, LuaTable)| {
                let urgency = data.get("urgency")?;
                let time_to_live = data.get("time_to_live")?;
                let times_pushed = data.get("times_pushed")?;
                participant
                    .borrow_mut().pushed_obligations
                    .entry(dialog_move)
                    .or_insert(PushedObligationMetadata {
                        urgency,
                        time_to_live,
                        times_pushed,
                    });
                Ok(())
            },
        );
        methods.add_method("remove_obligation", |_, participant, dialog_move| {
            participant.borrow_mut().pushed_obligations.remove(&dialog_move);
            Ok(())
        });
        methods.add_method("get_obligation", |_, participant, dialog_move| {
            if participant.borrow().pushed_obligations.contains_key(&dialog_move) {
                Ok(Some(LuaPOMetaData {
                    pushed_obligation: dialog_move,
                    location: participant.clone().into(),
                }))
            } else {
                Ok(None)
            }
        });
        methods.add_method("obligations_len", |_, participant, ()| {
            Ok(participant.borrow().pushed_obligations.len())
        });
        methods.add_method("iter_obligations", |lua, participant, ()| {
            let mut visited = AHashSet::new();
            let iter = lua.create_function_mut(move |lua, (participant, _): (Self, LuaValue)| {
                for dialog_move in participant.borrow().pushed_obligations.keys() {
                    if !visited.contains(dialog_move) {
                        visited.insert(dialog_move.clone());
                        return (dialog_move.clone(), LuaPOMetaData {
                            pushed_obligation: dialog_move.clone(),
                            location: participant.clone().into(),
                        }).to_lua_multi(lua);
                    }
                }

                Ok(MultiValue::new())
            })?;
            Ok((iter, participant.clone(), Nil))
        });
    }
}

#[derive(Debug, Clone, From)]
pub(crate) enum LuaTopicStateLocation {
    Conversation(LuaConversation),
    HistoricalMove(LuaHistoricalMove),
}

#[derive(Debug, Clone, From)]
pub(crate) struct LuaTopicState(pub LuaTopicStateLocation);

impl LuaTopicState {
    pub fn borrow(&self) -> Ref<TopicState<DialogTypes>> {
        match &self.0 {
            LuaTopicStateLocation::Conversation(conversation) => Ref::map(
                conversation.borrow(),
                |conversation| &conversation.topic_state,
            ),
            LuaTopicStateLocation::HistoricalMove(hmove) => Ref::map(
                hmove.borrow(),
                |historical_move| &historical_move.topic_state,
            ),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<TopicState<DialogTypes>> {
        match &self.0 {
            LuaTopicStateLocation::Conversation(conversation) => RefMut::map(
                conversation.borrow_mut(),
                |conversation| &mut conversation.topic_state,
            ),
            LuaTopicStateLocation::HistoricalMove(hmove) => RefMut::map(
                hmove.borrow_mut(),
                |historical_move| &mut historical_move.topic_state,
            ),
        }
    }
}

impl UserData for LuaTopicState {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("conversation", |_, topic_state| {
            if let LuaTopicStateLocation::Conversation(ref conversation) = topic_state.0 {
                Ok(Some(conversation.clone()))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("historical_move", |_, topic_state| {
            if let LuaTopicStateLocation::HistoricalMove(ref hmove) = topic_state.0 {
                Ok(Some(hmove.clone()))
            } else {
                Ok(None)
            }
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("can_be_addressed", |_, topic_state, topic| {
            Ok(topic_state.borrow().can_be_addressed(&topic))
        });
        methods.add_method("needs_addressing", |_, topic_state, topic| {
            Ok(topic_state.borrow().needs_addressing(&topic))
        });
        methods.add_method("can_be_introduced", |_, topic_state, topic| {
            Ok(topic_state.borrow().can_be_introduced(&topic))
        });
        methods.add_method("introduce", |_, topic_state, topic| {
            topic_state.borrow_mut().introduced.insert(topic);
            Ok(())
        });
        methods.add_method("is_introduced", |_, topic_state, topic| {
            Ok(topic_state.borrow().introduced.contains(&topic))
        });
        methods.add_method("introduced_len", |_, topic_state, ()| {
            Ok(topic_state.borrow().introduced.len())
        });
        methods.add_method("remove_introduced", |_, topic_state, topic| {
            topic_state.borrow_mut().introduced.remove(&topic);
            Ok(())
        });
        methods.add_method("iter_introduced", |lua, topic_state, ()| {
            let mut visited = AHashSet::new();
            let iter = lua.create_function_mut(move |lua, (topic_state, _): (Self, Topic)| {
                for topic in topic_state.borrow().introduced.iter() {
                    if !visited.contains(topic) {
                        visited.insert(topic.clone());
                        return (visited.len(), topic.clone()).to_lua_multi(lua);
                    }
                }

                Ok(MultiValue::new())
            })?;
            Ok((iter, topic_state.clone(), Nil))
        });
        methods.add_method("address", |_, topic_state, topic| {
            topic_state.borrow_mut().addressed.insert(topic);
            Ok(())
        });
        methods.add_method("is_addressed", |_, topic_state, topic| {
            Ok(topic_state.borrow().addressed.contains(&topic))
        });
        methods.add_method("addressed_len", |_, topic_state, ()| {
            Ok(topic_state.borrow().addressed.len())
        });
        methods.add_method("remove_addressed", |_, topic_state, topic| {
            topic_state.borrow_mut().addressed.remove(&topic);
            Ok(())
        });
        methods.add_method("iter_addressed", |lua, topic_state, ()| {
            let mut visited = AHashSet::new();
            let iter = lua.create_function_mut(move |lua, (topic_state, _): (Self, Topic)| {
                for topic in topic_state.borrow().addressed.iter() {
                    if !visited.contains(topic) {
                        visited.insert(topic.clone());
                        return (visited.len(), topic.clone()).to_lua_multi(lua);
                    }
                }

                Ok(MultiValue::new())
            })?;
            Ok((iter, topic_state.clone(), Nil))
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LuaConversation(pub RcRef<Conversation<DialogTypes>>);

impl LuaConversation {
    pub fn borrow(&self) -> Ref<Conversation<DialogTypes>> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Conversation<DialogTypes>> {
        self.0.borrow_mut()
    }
}

impl UserData for LuaConversation {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("initiator", |_, conversation| {
            Ok(LuaSpeaker::from(conversation.borrow().initiator))
        });
        fields.add_field_method_set("initiator", |_, conversation, initiator| {
            conversation.borrow_mut().initiator = LuaSpeaker::into(initiator);
            Ok(())
        });
        fields.add_field_method_get("speaker", |_, conversation| {
            Ok(LuaSpeaker::from(conversation.borrow().speaker))
        });
        fields.add_field_method_set("speaker", |_, conversation, speaker| {
            conversation.borrow_mut().speaker = LuaSpeaker::into(speaker);
            Ok(())
        });
        fields.add_field_method_get("person0", |_, conversation| {
            Ok(LuaParticipantState {
                person0: true,
                conversation: conversation.clone(),
            })
        });
        fields.add_field_method_set("person0", |_, conversation, pstate: LuaParticipantState| {
            conversation.borrow_mut().person0 = pstate.borrow_mut().clone();
            Ok(())
        });
        fields.add_field_method_get("person1", |_, conversation| {
            Ok(LuaParticipantState {
                person0: false,
                conversation: conversation.clone(),
            })
        });
        fields.add_field_method_set("person1", |_, conversation, pstate: LuaParticipantState| {
            conversation.borrow_mut().person0 = pstate.borrow_mut().clone();
            Ok(())
        });
        fields.add_field_method_get("topic_state", |_, conversation| {
            Ok(LuaTopicState(LuaTopicStateLocation::Conversation(conversation.clone())))
        });
        fields.add_field_method_set("topic_state", |_, conversation, topic: LuaTopicState| {
            conversation.borrow_mut().topic_state = topic.borrow().clone();
            Ok(())
        });
        fields.add_field_method_get("done", |_, conversation| {
            Ok(conversation.borrow().done)
        });
        fields.add_field_method_set("done", |_, conversation, done| {
            conversation.borrow_mut().done = done;
            Ok(())
        });
        fields.add_field_method_set("lull_continue_chance", |_, conversation, lull| {
            conversation.borrow_mut().lull_continue_chance = Bernoulli::new(lull).unwrap();
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_speaker_state", |_, conversation, speaker: LuaSpeaker| {
            Ok(LuaParticipantState {
                person0: speaker.0 == Speaker::Person0,
                conversation: conversation.clone(),
            })
        });
        methods.add_method("get_my_state", |_, conversation, ()| {
            let conversation_ref = conversation.borrow();
            Ok(LuaParticipantState {
                person0: conversation_ref.speaker == Speaker::Person0,
                conversation: conversation.clone(),
            })
        });
        methods.add_method("get_others_state", |_, conversation, ()| {
            let conversation_ref = conversation.borrow();
            Ok(LuaParticipantState {
                person0: conversation_ref.speaker != Speaker::Person0,
                conversation: conversation.clone(),
            })
        });
        methods.add_method("history_len", |_, conversation, ()| {
            Ok(conversation.borrow().history.len())
        });
        methods.add_method("get_historical_move", |_, conversation, mut index: usize| {
            index = index.wrapping_sub(1);
            let conversation_ref = conversation.borrow();
            if index < conversation_ref.history.len() {
                Ok(Some(LuaHistoricalMove::Conversation {
                    index,
                    conversation: conversation.clone(),
                }))
            } else {
                Ok(None)
            }
        });
        methods.add_method("remove_historical_move", |_, conversation, index: Option<_>| {
            let mut conversation = conversation.borrow_mut();
            let index = index.unwrap_or(conversation.history.len()).wrapping_sub(1);
            if index < conversation.history.len() {
                let removed = conversation.history.remove(index);
                Ok(Some(LuaHistoricalMove::Inline(RcRef::new(removed.into()))))
            } else {
                Ok(None)
            }
        });
        methods.add_method("history_iter", |lua, conversation, ()| {
            let iter = lua.create_function(|lua, (conversation, index): (Self, usize)| {
                let conversation_ref = conversation.borrow();
                if index < conversation_ref.history.len() {
                    (index + 1, LuaHistoricalMove::Conversation {
                        index,
                        conversation: conversation.clone(),
                    }).to_lua_multi(lua)
                } else {
                    Ok(MultiValue::new())
                }
            })?;
            Ok((iter, conversation.clone(), 0_usize))
        });
        methods.add_method("insert_goal", |_, conversation, goal: LuaGoalState| {
            let mut conversation = conversation.borrow_mut();
            conversation.goals.push(Goal::new(goal));
            Ok(())
        });
        methods.add_method("goals_len", |_, conversation, ()| {
            Ok(conversation.borrow().goals.len())
        });
    }
}
