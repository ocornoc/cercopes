use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, From, Into)]
pub(crate) struct LuaGoalPursuer(pub GoalPursuer);

impl ToLua<'_> for LuaGoalPursuer {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self.0 {
            GoalPursuer::Speaker(speaker) => LuaSpeaker(speaker).to_lua(lua),
            GoalPursuer::Any => Ok(Nil),
        }
    }
}

impl FromLua<'_> for LuaGoalPursuer {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if lua_value == Nil {
            Ok(GoalPursuer::Any.into())
        } else {
            Ok(GoalPursuer::Speaker(LuaSpeaker::from_lua(lua_value, lua)?.into()).into())
        }
    }
}

#[derive(Debug, Deref, DerefMut, Clone, From, Into)]
pub(crate) struct LuaGoalMove(pub GoalMove<DialogTypes>);

impl ToLua<'_> for LuaGoalMove {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table_with_capacity(0, 2)?;
        table.set("pursuer", LuaGoalPursuer(self.pursuer))?;
        table.set("dialog_move", self.dialog_move.clone())?;
        table.to_lua(lua)
    }
}

impl FromLua<'_> for LuaGoalMove {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(lua_value, lua)?;
        Ok(LuaGoalMove(GoalMove {
            pursuer: table.get::<_, LuaGoalPursuer>("pursuer")?.into(),
            dialog_move: table.get("dialog_move")?,
        }))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LuaGoalState {
    pub lua: LuaStatePtr,
    pub next_step: RegistryFunction,
    pub made_move: RegistryFunction,
    pub is_satisfied: RegistryFunction,
}

impl ToLua<'_> for LuaGoalState {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table_with_capacity(0, 3)?;
        table.set("next_step", self.next_step)?;
        table.set("made_move", self.made_move)?;
        table.set("is_satisfied", self.is_satisfied)?;
        table.to_lua(lua)
    }
}

impl FromLua<'_> for LuaGoalState {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(lua_value, lua)?;
        Ok(LuaGoalState {
            lua: LuaStatePtr::new(lua),
            next_step: table.get("next_step")?,
            made_move: table.get("made_move")?,
            is_satisfied: table.get("is_satisfied")?,
        })
    }
}

impl GoalState<DialogTypes> for LuaGoalState {
    fn next_step(&self, state: &Conversation<DialogTypes>) -> Option<GoalMove<DialogTypes>> {
        let state = state.clone();
        self.next_step
            .get(self.lua.get())
            .unwrap()
            .call::<_, Option<LuaGoalMove>>(LuaConversation(RcRef::new(state.into())))
            .unwrap()
            .map(|goal_move| goal_move.0)
    }

    fn made_move(
        &mut self,
        state: &Conversation<DialogTypes>,
        history: &HistoricalMove<DialogTypes>,
    ) {
        let state = state.clone();
        self.made_move
            .get(self.lua.get())
            .unwrap()
            .call::<_, MultiValue>((
                LuaConversation(RcRef::new(state.into())),
                LuaHistoricalMove::new_inline(history.clone()),
            ))
            .unwrap();
    }

    fn is_satisfied(&self) -> bool {
        self.is_satisfied
            .get(self.lua.get())
            .unwrap()
            .call(())
            .unwrap()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LuaFrame {
    pub lua: LuaStatePtr,
    pub modify: RegistryFunction,
}

impl LuaFrame {
    pub fn modify(self) -> impl Fn(&mut Conversation<DialogTypes>) {
        move |conversation| {
            let lua = self.lua.get();
            let modify = self.modify.get(lua).unwrap();
            let new_conversation = conversation.clone();
            lua.scope(|scope| {
                let new_conversation = scope.create_userdata(LuaConversation(
                    RcRef::new(new_conversation.into()),
                )).unwrap();
                modify.call::<_, ()>(new_conversation.clone()).unwrap();
                let new_conversation = new_conversation.borrow::<LuaConversation>().unwrap();
                conversation.clone_from(&new_conversation.borrow());
                Ok(())
            }).unwrap();
        }
    }
}

impl FromLua<'_> for LuaFrame {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        Ok(LuaFrame {
            lua: LuaStatePtr::new(lua),
            modify: RegistryFunction::from_lua(lua_value, lua)?,
        })
    }
}

impl ToLua<'_> for LuaFrame {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.modify.to_lua(lua)
    }
}
