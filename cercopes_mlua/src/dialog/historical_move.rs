use super::*;

#[derive(Debug, Clone)]
pub(crate) struct LuaHistoricalObligations {
    pub person0: bool,
    pub hmove: LuaHistoricalMove,
}

impl LuaHistoricalObligations {
    pub fn borrow(&self) -> Ref<HistoricalObligations<DialogTypes>> {
        Ref::map(
            self.hmove.borrow(),
            |hmove| if self.person0 {
                &hmove.person0_obligations
            } else {
                &hmove.person1_obligations
            },
        )
    }

    pub fn borrow_mut(&self) -> RefMut<HistoricalObligations<DialogTypes>> {
        RefMut::map(
            self.hmove.borrow_mut(),
            |hmove| if self.person0 {
                &mut hmove.person0_obligations
            } else {
                &mut hmove.person1_obligations
            },
        )
    }
}

impl UserData for LuaHistoricalObligations {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("historical_move", |_, hobls| {
            Ok(hobls.hmove.clone())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("push", |_, hobls, (dialog_move, urgency, time_to_live)| {
            hobls.borrow_mut().push(dialog_move, urgency, time_to_live);
            Ok(())
        });
        methods.add_method("remove_pushed", |_, hobls, dialog_move| {
            hobls.borrow_mut().pushed.remove(&dialog_move);
            Ok(())
        });
        methods.add_method("get_pushed", |_, hobls, dialog_move| {
            if hobls.borrow().pushed.contains_key(&dialog_move) {
                Ok(Some(LuaPOMetaData {
                    pushed_obligation: dialog_move,
                    location: LuaPOMLocation::HistoricalObligations(hobls.clone()),
                }))
            } else {
                Ok(None)
            }
        });
        methods.add_method("pushed_len", |_, hobls, ()| {
            Ok(hobls.borrow().pushed.len())
        });
        methods.add_method("iter_pushed", |lua, hobls, ()| {
            let mut visited = AHashSet::new();
            let iter = lua.create_function_mut(move |lua, (hobls, _): (Self, LuaValue)| {
                for dialog_move in hobls.borrow().pushed.keys() {
                    if !visited.contains(dialog_move) {
                        visited.insert(dialog_move.clone());
                        return (dialog_move.clone(), LuaPOMetaData {
                            pushed_obligation: dialog_move.clone(),
                            location: LuaPOMLocation::HistoricalObligations(hobls.clone()),
                        }).to_lua_multi(lua);
                    }
                }

                Ok(MultiValue::new())
            })?;
            Ok((iter, hobls.clone(), Nil))
        });
        methods.add_method("address", |_, hobls, dialog_move| {
            hobls.borrow_mut().addressed.insert(dialog_move);
            Ok(())
        });
        methods.add_method("remove_addressed", |_, hobls, dialog_move| {
            hobls.borrow_mut().addressed.remove(&dialog_move);
            Ok(())
        });
        methods.add_method("addressed_len", |_, hobls, ()| {
            Ok(hobls.borrow().addressed.len())
        });
        methods.add_method("iter_addressed", |lua, hobls, ()| {
            let mut visited = AHashSet::new();
            let iter = lua.create_function_mut(move |lua, (hobls, _): (Self, LuaValue)| {
                for dialog_move in hobls.borrow().addressed.iter() {
                    if !visited.contains(dialog_move) {
                        visited.insert(dialog_move.clone());
                        return (visited.len(), dialog_move.clone()).to_lua_multi(lua);
                    }
                }

                Ok(MultiValue::new())
            })?;
            Ok((iter, hobls.clone(), Nil))
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) enum LuaHistoricalMove {
    Inline(RcRef<HistoricalMove<DialogTypes>>),
    Conversation {
        index: usize,
        conversation: LuaConversation,
    },
}

impl LuaHistoricalMove {
    pub fn new_inline(hmove: HistoricalMove<DialogTypes>) -> Self {
        LuaHistoricalMove::Inline(RcRef::new(hmove.into()))
    }

    pub fn borrow(&self) -> Ref<HistoricalMove<DialogTypes>> {
        match self {
            LuaHistoricalMove::Inline(inline) => inline.borrow(),
        &LuaHistoricalMove::Conversation { index, ref conversation } => Ref::map(
                conversation.borrow(),
                |data| &data.history[index],
            ),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<HistoricalMove<DialogTypes>> {
        match self {
            LuaHistoricalMove::Inline(inline) => inline.borrow_mut(),
            &LuaHistoricalMove::Conversation { index, ref conversation } => RefMut::map(
                conversation.borrow_mut(),
                |data| &mut data.history[index],
            ),
        }
    }
}

impl UserData for LuaHistoricalMove {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("words", |_, hmove| {
            Ok(hmove.borrow().utterance.clone())
        });
        fields.add_field_method_set("words", |_, hmove, words| {
            hmove.borrow_mut().utterance = words;
            Ok(())
        });
        fields.add_field_method_get("speaker", |_, hmove| {
            Ok(LuaSpeaker(hmove.borrow().speaker))
        });
        fields.add_field_method_set("speaker", |_, hmove, speaker| {
            hmove.borrow_mut().speaker = LuaSpeaker::into(speaker);
            Ok(())
        });
        fields.add_field_method_get("person0_obligations", |_, hmove| {
            Ok(LuaHistoricalObligations {
                person0: true,
                hmove: hmove.clone(),
            })
        });
        fields.add_field_method_get("person1_obligations", |_, hmove| {
            Ok(LuaHistoricalObligations {
                person0: false,
                hmove: hmove.clone(),
            })
        });
        fields.add_field_method_get("topic_state", |_, hmove| {
            Ok(LuaTopicState(LuaTopicStateLocation::HistoricalMove(hmove.clone())))
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("all_addressed_obligations", |lua, hmove, ()| {
            let mut visited = AHashSet::new();
            let iter = lua.create_function_mut(move |lua, (hmove, _): (Self, LuaValue)| {
                for dialog_move in hmove.borrow().all_addressed_obligations() {
                    if !visited.contains(dialog_move) {
                        visited.insert(dialog_move.clone());
                        return (
                            visited.len(),
                            dialog_move.clone(),
                        ).to_lua_multi(lua);
                    }
                }

                Ok(MultiValue::new())
            })?;
            Ok((iter, hmove.clone(), Nil))
        });
        methods.add_method("was_move_satisfied", |_, hmove, dialog_move| {
            Ok(hmove.borrow().was_move_satisfied(&dialog_move))
        });
        methods.add_method("get_speaker_obligations", |_, hmove, speaker| {
            Ok(LuaHistoricalObligations {
                person0: LuaSpeaker(Speaker::Person0) == speaker,
                hmove: hmove.clone(),
            })
        });
        methods.add_method("get_my_obligations", |_, hmove, ()| {
            Ok(LuaHistoricalObligations {
                person0: hmove.borrow().speaker == Speaker::Person0,
                hmove: hmove.clone(),
            })
        });
        methods.add_method("get_others_obligations", |_, hmove, ()| {
            Ok(LuaHistoricalObligations {
                person0: hmove.borrow().speaker != Speaker::Person0,
                hmove: hmove.clone(),
            })
        });
    }
}
