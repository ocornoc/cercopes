use super::*;

#[derive(Debug, Clone, Deref, DerefMut)]
pub(crate) struct LuaReflexiveModel(RcRef<ReflexiveModel<KnowledgeTypes>>);

impl LuaReflexiveModel {
    pub fn new<'lua>(
        lua: &'lua Lua,
        model: ReflexiveModel<KnowledgeTypes>,
    ) -> LuaResult<AnyUserData<'lua>> {
        let model = lua.create_userdata(LuaReflexiveModel(Rc::new(model.into())))?;
        Ok(model)
    }

    pub(super) fn lua_new<'lua>(lua: &'lua Lua) -> LuaResult<Function<'lua>> {
        lua.create_function(|lua, holder| {
            LuaReflexiveModel::new(lua, ReflexiveModel::new(holder))
        })
    }
}

impl UserData for LuaReflexiveModel {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_function_get("holder", |_, model| {
            model.get_named_user_value::<_, Entity>("holder")
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method("__eq", |_, left, right: Self| {
            Ok(left.borrow().holder == right.borrow().holder)
        });
        methods.add_meta_method("__index", |_, model, facet| {
            let model = model.borrow();
            Ok(model.facets.get(&facet).cloned())
        });
        methods.add_meta_method("__newindex", |_, model, (facet, value)| {
            model.borrow_mut().facets.insert(facet, value);
            Ok(())
        });
        methods.add_method("update_truths", |_, model, ()| {
            model.borrow_mut().update_truths();
            Ok(())
        });
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub(crate) struct LuaEvidenceModel(RcRef<EvidenceModel<KnowledgeTypes>>);

impl LuaEvidenceModel {
    pub fn new<'lua>(
        lua: &'lua Lua,
        model: EvidenceModel<KnowledgeTypes>,
    ) -> LuaResult<AnyUserData<'lua>> {
        Ok(lua.create_userdata(LuaEvidenceModel(RcRef::new(model.into())))?)
    }

    pub(super) fn lua_new<'lua>(lua: &'lua Lua) -> LuaResult<Function<'lua>> {
        lua.create_function(|lua, (holder, regarding)| {
            LuaEvidenceModel::new(lua, EvidenceModel::new(holder, regarding))
        })
    }
}

impl UserData for LuaEvidenceModel {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("holder", |_, model| {
            Ok(model.borrow().holder.clone())
        });
        fields.add_field_method_get("regarding", |_, model| {
            Ok(model.borrow().regarding.clone())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method("__eq", |_, left, right: Self| {
            let left = left.borrow();
            let right = right.borrow();
            Ok(left.holder == right.holder && left.regarding == right.regarding)
        });
        methods.add_meta_method("__index", |lua, model, facet| {
            if model.borrow().regarding.is_facet_relevant(&facet) {
                LuaFacetData {
                    model: model.clone(),
                    facet,
                }.new(lua).map(Some)
            } else {
                Ok(None)
            }
        });
        methods.add_meta_method("__len", |_, model, ()| {
            Ok(model.borrow().facets.len())
        });
        methods.add_function("iter_facets", |lua, model: AnyUserData| {
            let mut visited = AHashSet::with_capacity(20);
            let iter = lua.create_function_mut(move |lua, (model, _): (Self, LuaValue)| {
                let model_borrow = model.borrow();
                for facet in model_borrow.facets.keys() {
                    if !visited.contains(facet) {
                        visited.insert(facet.clone());
                        let facet = facet.clone();
                        std::mem::drop(model_borrow);
                        return (facet.clone(), LuaFacetData {
                            model,
                            facet,
                        }.new(lua)?).to_lua_multi(lua);
                    }
                }
                ().to_lua_multi(lua)
            })?;
            Ok((iter, model, Nil))
        });
        methods.add_method("recompute_strongest", |_, model, ()| {
            model.borrow_mut().recompute_strongest();
            Ok(())
        });
        methods.add_method("recompute_total_strengths", |_, model, ()| {
            model.borrow_mut().recompute_total_strengths();
            Ok(())
        });
        methods.add_method("update_truth", |_, model, ()| {
            model.borrow_mut().update_truths();
            Ok(())
        });
        methods.add_method("get_strongest_belief", |_, model, facet| {
            Ok(model.borrow().get_strongest_belief(&facet).cloned())
        });
        methods.add_method("mutate", |_, model, ()| {
            model.borrow_mut().mutate(&mut thread_rng());
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garbage() -> LuaResult<()> {
        let lua = Lua::new();
        crate::initialize_lua(&lua)?;
        match lua.load(include_bytes!("../tests/model/garbage.lua").as_slice()).exec() {
            result@Ok(()) => result,
            Err(err) => {
                println!("{err}");
                Err(err)
            },
        }
    }
}
