use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryFunctionUD(Rc<RegistryKey>);

impl RegistryFunctionUD {
    fn new<'lua>(lua: &'lua Lua, new: Function<'lua>) -> LuaResult<Self> {
        Ok(RegistryFunctionUD(Rc::new(lua.create_registry_value(new)?)))
    }

    fn get<'lua>(&self, lua: &'lua Lua) -> LuaResult<Function<'lua>> {
        debug_assert!(lua.owns_registry_value(&self.0));
        lua.registry_value(&self.0)
    }

    fn set<'lua>(&self, lua: &'lua Lua, new: Function<'lua>) -> LuaResult<()> {
        debug_assert!(lua.owns_registry_value(&self.0));
        lua.replace_registry_value(&self.0, new)
    }
}

impl UserData for RegistryFunctionUD {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("inner", |lua, rfn| {
            rfn.get(lua)
        });
        fields.add_field_method_set("inner", |lua, rfn, new| {
            rfn.set(lua, new)
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method("__call", |lua, try_mutate, args| {
            try_mutate.get(lua)?.call::<mlua::MultiValue, mlua::MultiValue>(args)
        });
        methods.add_function("new", |lua, new| {
            RegistryFunctionUD::new(lua, new)
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegistryFunction(RegistryFunctionUD);

impl RegistryFunction {
    pub fn new<'lua>(lua: &'lua Lua, new: Function<'lua>) -> LuaResult<Self> {
        RegistryFunctionUD::new(lua, new).map(RegistryFunction)
    }

    pub fn get<'lua>(&self, lua: &'lua Lua) -> LuaResult<Function<'lua>> {
        self.0.get(lua)
    }

    #[allow(dead_code)]
    pub fn set<'lua>(&self, lua: &'lua Lua, new: Function<'lua>) -> LuaResult<()> {
        self.0.set(lua, new)
    }
}

impl ToLua<'_> for RegistryFunction {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.to_lua(lua)
    }
}

impl FromLua<'_> for RegistryFunction {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Function(func) = lua_value {
            RegistryFunction::new(lua, func)
        } else {
            RegistryFunctionUD::from_lua(lua_value, lua).map(RegistryFunction)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RegistryTableUD(Rc<RegistryKey>);

impl RegistryTableUD {
    fn new<'lua>(lua: &'lua Lua, new: LuaTable<'lua>) -> LuaResult<Self> {
        Ok(RegistryTableUD(Rc::new(lua.create_registry_value(new)?)))
    }

    fn get<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaTable<'lua>> {
        debug_assert!(lua.owns_registry_value(&self.0));
        lua.registry_value(&self.0)
    }

    fn set<'lua>(&self, lua: &'lua Lua, new: LuaTable<'lua>) -> LuaResult<()> {
        debug_assert!(lua.owns_registry_value(&self.0));
        lua.replace_registry_value(&self.0, new)
    }
}

impl UserData for RegistryTableUD {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("inner", |lua, rt| {
            rt.get(lua)
        });
        fields.add_field_method_set("inner", |lua, rt, new| {
            rt.set(lua, new)
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("new", |lua, new| {
            RegistryTableUD::new(lua, new)
        });
        methods.add_meta_method("__index", |lua, rt, key| {
            rt.get(lua)?.get::<LuaValue, LuaValue>(key)
        });
        methods.add_meta_method("__newindex", |lua, rt, (key, value)| {
            rt.get(lua)?.set::<LuaValue, LuaValue>(key, value)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegistryTable(RegistryTableUD);

impl RegistryTable {
    pub fn new<'lua>(lua: &'lua Lua, new: LuaTable<'lua>) -> LuaResult<Self> {
        RegistryTableUD::new(lua, new).map(RegistryTable)
    }

    #[allow(dead_code)]
    pub fn get<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaTable<'lua>> {
        self.0.get(lua)
    }

    #[allow(dead_code)]
    pub fn set<'lua>(&self, lua: &'lua Lua, new: LuaTable<'lua>) -> LuaResult<()> {
        self.0.set(lua, new)
    }
}

impl ToLua<'_> for RegistryTable {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.get(lua)?.to_lua(lua)
    }
}

impl FromLua<'_> for RegistryTable {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Table(table) = lua_value {
            RegistryTable::new(lua, table)
        } else {
            RegistryTableUD::from_lua(lua_value, lua).map(RegistryTable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implicit_registry_fn_table() -> LuaResult<()> {
        let lua = Lua::new();
        crate::initialize_lua(&lua)?;
        let table: RegistryTable = lua.load(r#"
        local counter = 0
        local shared_table = {
            ["foo"] = 10,
            ["bar"] = function()
                local old_value = counter
                counter = counter + 1
                return old_value
            end,
        }
        shared_table.baz = -1
        return shared_table
        "#).eval()?;
        lua.expire_registry_values();
        assert_eq!(table.get(&lua)?.get::<_, f32>("foo")?, 10.0);
        assert_eq!(table.get(&lua)?.get::<_, f32>("baz")?, -1.0);
        let bar: RegistryFunction = table.get(&lua)?.get("bar")?;
        assert_eq!(bar.get(&lua)?.call::<_, f32>(())?, 0.0);
        lua.expire_registry_values();
        assert_eq!(bar.get(&lua)?.call::<_, f32>(())?, 1.0);
        assert_eq!(bar.get(&lua)?.call::<_, f32>(())?, 2.0);
        Ok(())
    }
}
