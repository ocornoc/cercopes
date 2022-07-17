use rand::prelude::*;
use cercopes_dialog::{*, goal::*};
use historical_move::*;
use conversation::*;
use goal::*;
use expander::*;
use manager::*;
use super::*;

mod historical_move;
mod conversation;
mod goal;
mod expander;
mod manager;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct Topic(pub String);

impl ToLua<'_> for Topic {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.to_lua(lua)
    }
}

impl FromLua<'_> for Topic {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        String::from_lua(lua_value, lua).map(Topic)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct DialogMove(pub String);

impl ToLua<'_> for DialogMove {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.to_lua(lua)
    }
}

impl FromLua<'_> for DialogMove {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        String::from_lua(lua_value, lua).map(DialogMove)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct ExpanderNode(pub String);

impl ToLua<'_> for ExpanderNode {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.to_lua(lua)
    }
}

impl FromLua<'_> for ExpanderNode {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        String::from_lua(lua_value, lua).map(ExpanderNode)
    }
}

#[derive(Debug, Clone)]
pub struct Character(pub(crate) RegistryTable);

impl ToLua<'_> for Character {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.to_lua(lua)
    }
}

impl FromLua<'_> for Character {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        RegistryTable::from_lua(lua_value, lua).map(Character)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DialogTypes;

impl DialogTrait for DialogTypes {
    type Topic = Topic;

    type DialogMove = DialogMove;

    type ExpanderNode = ExpanderNode;

    type Character = Character;
}

#[derive(Debug, Default, From, Into)]
pub(crate) struct LuaPrecondition(pub Precondition<DialogTypes>);

impl ToLua<'_> for LuaPrecondition {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        if let Some(precondition) = self.0.condition {
            lua.create_function(move |_, state: LuaConversation| {
                Ok(precondition(&state.borrow_mut()))
            }).map(LuaValue::Function)
        } else {
            Ok(Nil)
        }
    }
}

impl FromLua<'_> for LuaPrecondition {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let Some(function) = Option::<Function>::from_lua(lua_value, lua)? {
            let function = RegistryFunction::new(lua, function)?;
            let lua = LuaStatePtr::new(lua);
            Ok(Precondition::new(move |state| {
                let function = function.get(lua.get()).unwrap();
                let state = state.clone();
                function.call(LuaConversation(RcRef::new(state.into()))).unwrap()
            }).into())
        } else {
            Ok(Default::default())
        }
    }
}

pub(crate) fn initialize_lua(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    globals.set("DialogManager", LuaDialogManager::lua_new(lua)?)?;
    Ok(())
}
