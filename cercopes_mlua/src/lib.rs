use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};
use std::ptr::NonNull;
use mlua::{
    Result as LuaResult, Error as LuaError, ToLua, FromLua, UserData, Value as LuaValue,
    Table as LuaTable, String as LuaString, Lua, UserDataFields, UserDataMethods,
    Nil, AnyUserData, MultiValue, ToLuaMulti, Function,
    RegistryKey,
};
use ahash::AHashSet;
use derive_more::{From, Into, Deref, DerefMut, Display, AsRef, AsMut};
use registry::*;

mod registry;
pub mod dialog;
pub mod knowledge;

type RcRef<T> = Rc<RefCell<T>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LuaStatePtr(NonNull<Lua>);

impl LuaStatePtr {
    #[inline]
    pub fn new<'lua>(lua: &'lua Lua) -> Self {
        LuaStatePtr(unsafe { NonNull::new_unchecked(lua as *const Lua as *mut Lua) })
    }

    #[inline]
    pub fn get<'lua>(self) -> &'lua Lua {
        unsafe { self.0.as_ref() }
    }
}

impl From<&'_ Lua> for LuaStatePtr {
    #[inline]
    fn from(lua: &Lua) -> Self {
        LuaStatePtr::new(lua)
    }
}

pub fn initialize_lua(lua: &Lua) -> LuaResult<()> {
    lua.set_hook(mlua::HookTriggers::on_returns(), |lua, _| {
        lua.expire_registry_values();
        Ok(())
    })?;
    knowledge::initialize_lua(lua)?;
    dialog::initialize_lua(lua)?;
    Ok(())
}

pub fn create_lua() -> LuaResult<Lua> {
    let lua = Lua::new();
    initialize_lua(&lua)?;
    Ok(lua)
}
