use std::fmt::{Debug, Formatter, Result as FmtResult};
use super::*;

#[derive(Debug, Default, From, Into)]
pub(crate) struct LuaEditHMF(pub EditHistoricalMove<DialogTypes>);

impl FromLua<'_> for LuaEditHMF {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let Some(function) = Option::<Function>::from_lua(lua_value, lua)? {
            let function = RegistryFunction::new(lua, function)?;
            let lua = LuaStatePtr::new(lua);
            Ok(EditHistoricalMove::new(move |state, _, hmove| {
                let lua = lua.get();
                let function = function.get(lua).unwrap();
                lua.scope(|_| {
                    let lua_state = LuaConversation(RcRef::new(state.clone().into()));
                    let lua_hmove = LuaHistoricalMove::new_inline(hmove.clone());
                    function.call::<_, ()>((lua_state.clone(), lua_hmove.clone())).unwrap();
                    state.clone_from(&lua_state.borrow());
                    hmove.clone_from(&lua_hmove.borrow());
                    Ok(())
                }).unwrap();
            }).into())
        } else {
            Ok(Default::default())
        }
    }
}

impl ToLua<'_> for LuaEditHMF {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        if let Some(edit) = self.0.edit {
            lua.create_function(move |_, (state, hmove)| {
                let mut state = LuaConversation::borrow_mut(&state);
                let mut hmove = LuaHistoricalMove::borrow_mut(&hmove);
                Ok(edit(&mut state, &mut StdRng::from_entropy(), &mut hmove))
            }).map(LuaValue::Function)
        } else {
            Ok(Nil)
        }
    }
}

#[derive(From, Into)]
pub(crate) struct LuaMoveNodeFormatter(pub MoveNodeFormatter<DialogTypes>);

impl Debug for LuaMoveNodeFormatter {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "LuaMoveNodeFormatter({:p})", self.0)
    }
}

impl FromLua<'_> for LuaMoveNodeFormatter {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let function = RegistryFunction::new(lua, Function::from_lua(lua_value, lua)?)?;
        let lua = LuaStatePtr::new(lua);
        Ok(LuaMoveNodeFormatter(Box::new(move |state, _, pieces| {
            let state = LuaConversation(RcRef::new(state.clone().into()));
            let lua = lua.get();
            let pieces = lua.create_sequence_from(pieces).unwrap();
            function.get(lua).unwrap().call((state, pieces)).unwrap()
        })))
    }
}

impl ToLua<'_> for LuaMoveNodeFormatter {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        lua.create_function(move |_, (state, pieces)| {
            let state = LuaConversation::borrow(&state);
            let pieces = LuaTable::sequence_values(pieces).collect::<LuaResult<_>>()?;
            Ok((self.0)(&state, &mut StdRng::from_entropy(), pieces))
        }).map(LuaValue::Function)
    }
}

#[derive(Debug, From, Into)]
pub(crate) struct LuaMoveNode(pub MoveNode<DialogTypes>);

impl FromLua<'_> for LuaMoveNode {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let table = LuaTable::from_lua(lua_value, lua)?;
        Ok(LuaMoveNode(MoveNode {
            dialog_moves: table
                .get::<_, LuaTable>("dialog_moves")?
                .sequence_values()
                .collect::<LuaResult<_>>()?,
            addressed_topics: table
                .get::<_, LuaTable>("addressed_topics")?
                .sequence_values()
                .collect::<LuaResult<_>>()?,
            precondition: table.get::<_, LuaPrecondition>("precondition")?.into(),
            edit_historical_move: table.get::<_, LuaEditHMF>("edit_historical_move")?.into(),
            formatter: table.get::<_, LuaMoveNodeFormatter>("formatter")?.into(),
            parts: table.get("parts")?,
        }))
    }
}
