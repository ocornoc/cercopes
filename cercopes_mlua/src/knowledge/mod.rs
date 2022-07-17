use cercopes_knowledge::{
    Evidence, EvidenceKind, EvidenceModel, Facet as FacetTrait, FacetValueData, FacetData,
    FacetValue as FacetValueTrait, Entity as EntityTrait, KnowledgeTrait, ReflexiveModel,
};
use rand::prelude::*;
pub(crate) use evidence::*;
pub(crate) use value_data::*;
pub(crate) use facet_data::*;
pub(crate) use model::*;
use super::*;

mod evidence;
mod value_data;
mod facet_data;
mod model;

#[derive(Debug, Display, Clone, From, Into, PartialEq, Eq, Hash, Deref, DerefMut, AsRef, AsMut)]
/// A Lua facet.
pub struct Facet(pub String);

impl ToLua<'_> for Facet {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.to_lua(lua)
    }
}

impl FromLua<'_> for Facet {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        Ok(Facet(FromLua::from_lua(lua_value, lua)?))
    }
}

impl FacetTrait<KnowledgeTypes> for Facet {
    fn initial_values(&self) -> Vec<FacetValue> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct FacetValue {
    pub(crate) lua: LuaStatePtr,
    pub facet: Facet,
    pub hash_string: String,
    pub(crate) try_mutate: RegistryFunction,
    pub(crate) value: RegistryTable,
}

impl FacetValue {
    pub fn new_lua<'lua>(lua: &'lua Lua, table: LuaTable<'lua>) -> LuaResult<Self> {
        Ok(FacetValue {
            lua: lua.into(),
            facet: table.get("facet")?,
            hash_string: table.get("hash_string")?,
            try_mutate: table.get("try_mutate")?,
            value: RegistryTable::new(lua, table)?,
        })
    }
}

impl PartialEq for FacetValue {
    fn eq(&self, other: &Self) -> bool {
        self.hash_string == other.hash_string
    }
}

impl Eq for FacetValue {}

impl Hash for FacetValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_string.hash(state)
    }
}

impl FacetValueTrait<KnowledgeTypes> for FacetValue {
    fn facet(&self) -> Facet {
        self.facet.clone()
    }

    fn try_mutate<R: Rng>(
        &self,
        model: &EvidenceModel<KnowledgeTypes>,
        evidence: &Evidence<KnowledgeTypes>,
        _: &mut R,
    ) -> Option<Self> {
        let all_evidence = &model.facets[&self.facet()].values[self].evidence;
        for (i, old_evidence) in all_evidence.iter().enumerate() {
            let old_evidence_p = old_evidence as *const Evidence<KnowledgeTypes>;
            if old_evidence_p == evidence {
                return self.try_mutate
                    .get(self.lua.get())
                    .unwrap()
                    .call((self.clone(), model.holder.clone(), model.regarding.clone(), i + 1))
                    .unwrap();
            }
        }
        unreachable!()
    }
}

impl ToLua<'_> for FacetValue {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.value.to_lua(lua)
    }
}

impl FromLua<'_> for FacetValue {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        FacetValue::new_lua(lua, LuaTable::from_lua(lua_value, lua)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub(crate) lua: LuaStatePtr,
    pub(crate) data: RegistryTable,
    pub(crate) relevant_facets: RegistryFunction,
    pub(crate) facet_truth: RegistryFunction,
    pub(crate) is_facet_relevant: RegistryFunction,
}

impl Entity {
    pub fn new_lua<'lua>(lua: &'lua Lua, table: LuaTable<'lua>) -> LuaResult<Self> {
        Ok(Entity {
            lua: lua.into(),
            relevant_facets: table.get("relevant_facets")?,
            facet_truth: table.get("facet_truth")?,
            is_facet_relevant: table.get("is_facet_relevant")?,
            data: RegistryTable::new(lua, table)?,
        })
    }
}

impl<'lua> ToLua<'lua> for Entity {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        self.data.to_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for Entity {
    fn from_lua(lua_value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        Entity::new_lua(lua, LuaTable::from_lua(lua_value, lua)?)
    }
}

impl EntityTrait<KnowledgeTypes> for Entity {
    fn relevant_facets(&self) -> Vec<Facet> {
        let table: LuaTable = self.relevant_facets
            .get(self.lua.get())
            .unwrap()
            .call(self.clone())
            .unwrap();
        table.sequence_values().map(|value| value.unwrap()).collect()
    }

    fn facet_truth(&self, facet: &Facet) -> Option<FacetValue> {
        self.facet_truth
            .get(self.lua.get())
            .unwrap()
            .call((self.clone(), facet.clone()))
            .unwrap()
    }

    fn is_facet_relevant(&self, facet: &Facet) -> bool {
        self.is_facet_relevant
            .get(self.lua.get())
            .unwrap()
            .call((self.clone(), facet.clone()))
            .unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Data(pub(crate) RegistryTable);

impl ToLua<'_> for Data {
    fn to_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        self.0.to_lua(lua)
    }
}

impl FromLua<'_> for Data {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        RegistryTable::from_lua(lua_value, lua).map(Data)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KnowledgeTypes;

impl KnowledgeTrait for KnowledgeTypes {
    type Facet = Facet;

    type FacetValue = FacetValue;

    type Entity = Entity;

    type Data = Data;
}

pub(crate) fn initialize_lua(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    globals.set("Entity", lua.create_function(Entity::new_lua)?)?;
    globals.set("ReflexiveModel", LuaReflexiveModel::lua_new(lua)?)?;
    globals.set("EvidenceModel", LuaEvidenceModel::lua_new(lua)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facet() -> LuaResult<()> {
        let lua = Lua::new();
        crate::initialize_lua(&lua)?;
        let (foo, bar, func): (Facet, Facet, Function) = lua.load(r#"
        return "foo", "bar", function(foo, bar)
            assert(foo == "foo")
            assert(bar == "bar")
            return true
        end
        "#).eval()?;
        lua.expire_registry_values();
        lua.gc_collect()?;
        lua.gc_collect()?;
        assert_eq!(foo, foo);
        assert_eq!(bar, bar);
        assert_ne!(foo, bar);
        let success: bool = func.call((foo, bar))?;
        assert!(success);
        Ok(())
    }

    #[test]
    fn test_value() -> LuaResult<()> {
        let lua = Lua::new();
        crate::initialize_lua(&lua)?;
        let (foo, bar, func): (FacetValue, FacetValue, Function) = lua.load(r#"
        local facet = "baz"
        local foo
        local bar
        foo = {
            facet = facet,
            hash_string = tostring(math.random()),
            try_mutate = function(...)
                return bar;
            end,
        }
        bar = {
            facet = facet,
            hash_string = tostring(math.random()),
            try_mutate = function(...)
                return foo;
            end,
        }
        return foo, bar, function(foo2, bar2)
            assert(foo2.hash_string == foo.hash_string)
            assert(bar2.hash_string == bar.hash_string)
            return true
        end
        "#).eval()?;
        lua.expire_registry_values();
        lua.gc_collect()?;
        lua.gc_collect()?;
        assert_eq!(foo.facet().0, "baz");
        assert_eq!(bar.facet().0, "baz");
        assert_ne!(foo.hash_string, bar.hash_string);
        assert_ne!(foo, bar);
        assert_eq!(bar, foo.try_mutate.get(&lua)?.call(())?);
        assert_eq!(foo, bar.try_mutate.get(&lua)?.call(())?);
        let success: bool = func.call((foo, bar))?;
        assert!(success);
        Ok(())
    }
}
