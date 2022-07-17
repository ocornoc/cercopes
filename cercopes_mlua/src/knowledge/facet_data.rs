use super::*;

#[derive(Debug, Clone)]
pub(crate) struct LuaFacetData {
    pub model: LuaEvidenceModel,
    pub facet: Facet,
}

impl LuaFacetData {
    pub fn new<'lua>(self, lua: &'lua Lua) -> LuaResult<AnyUserData<'lua>> {
        lua.create_userdata(self)
    }

    pub fn borrow_mut(&self) -> Option<RefMut<FacetData<KnowledgeTypes>>> {
        let model = self.model.borrow_mut();
        if model.regarding.is_facet_relevant(&self.facet) {
            Some(RefMut::map(model, |model| model.get_facet_data(self.facet.clone())))
        } else {
            None
        }
    }
}

impl UserData for LuaFacetData {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("facet", |_, facet_data| {
            Ok(facet_data.facet.clone())
        });
        fields.add_field_method_get("model", |_, facet_data| {
            Ok(facet_data.model.clone())
        });
        fields.add_field_method_get("truth", |_, facet_data| {
            if let Some(facet_data) = facet_data.borrow_mut() {
                Ok(Some(facet_data.truth.clone()))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("strongest", |_, facet_data| {
            if let Some(facet_data) = facet_data.borrow_mut() {
                Ok(Some(facet_data.strongest.clone()))
            } else {
                Ok(None)
            }
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method("__index", |lua, facet_data, value: FacetValue| {
            if facet_data.facet == value.facet() {
                LuaFacetValueData {
                    facet_data: facet_data.clone(),
                    value,
                }.new(lua).map(Some)
            } else {
                Ok(None)
            }
        });
        methods.add_meta_method("__len", |_, facet_data, ()| {
            if let Some(facet_data) = facet_data.borrow_mut() {
                Ok(Some(facet_data.values.len()))
            } else {
                Ok(None)
            }
        });
        methods.add_function("iter_values", |lua, facet_data: AnyUserData| {
            let mut visited = AHashSet::with_capacity(20);
            let iter = lua.create_function_mut(move |lua, (facet_data, _): (_, FacetValue)| {
                let facet_data_borrow_opt = LuaFacetData::borrow_mut(&facet_data);
                if let Some(ref facet_data_borrow) = facet_data_borrow_opt {
                    for value in facet_data_borrow.values.keys() {
                        if !visited.contains(&value.hash_string) {
                            visited.insert(value.hash_string.clone());
                            let value = value.clone();
                            std::mem::drop(facet_data_borrow_opt);
                            return (
                                value.clone(),
                                LuaFacetValueData {
                                    facet_data,
                                    value,
                                }.new(lua)?,
                            ).to_lua_multi(lua);
                        }
                    }
                }

                ().to_lua_multi(lua)
            })?;
            Ok((iter, facet_data, Nil))
        });
        methods.add_method("recompute_strongest", |_, facet_data, ()| {
            if let Some(mut facet_data) = facet_data.borrow_mut() {
                facet_data.recompute_strongest();
            }

            Ok(())
        });
        methods.add_method("recompute_total_strengths", |_, facet_data, ()| {
            if let Some(mut facet_data) = facet_data.borrow_mut() {
                facet_data.recompute_total_strengths();
            }

            Ok(())
        });
        methods.add_method("update_truth", |_, facet_data, ()| {
            // help Rust realize it should un-borrow the model
            let regarding = {
                facet_data.model.borrow_mut().regarding.clone()
            };
            if let Some(mut facet_data) = facet_data.borrow_mut() {
                facet_data.update_truth(&regarding)
            }

            Ok(())
        });
    }
}
