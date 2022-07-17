use super::*;

#[derive(Debug, Clone, Deref, DerefMut)]
pub(crate) struct LuaFacetValueData {
    #[deref]
    #[deref_mut]
    pub facet_data: LuaFacetData,
    pub value: FacetValue,
}

impl LuaFacetValueData {
    pub fn new<'lua>(self, lua: &'lua Lua) -> LuaResult<AnyUserData<'lua>> {
        lua.create_userdata(self)
    }

    pub fn borrow_mut(&self) -> Option<RefMut<FacetValueData<KnowledgeTypes>>> {
        let model = self.model.borrow_mut();
        if model.regarding.is_facet_relevant(&self.facet) {
            Some(RefMut::map(model, |model| model.get_value_data(self.value.clone())))
        } else {
            None
        }
    }
}

impl UserData for LuaFacetValueData {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("total_strength", |_, value_data| {
            if let Some(value_data) = value_data.borrow_mut() {
                Ok(Some(value_data.total_strength))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("facet_value", |_, value_data| {
            Ok(value_data.value.clone())
        });
        fields.add_field_method_get("facet_data", |_, value_data| {
            Ok(value_data.facet_data.clone())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("recompute_total_strength", |_, value_data, ()| {
            if let Some(mut value_data) = value_data.borrow_mut() {
                value_data.recompute_total_strength();
                Ok(Some(value_data.total_strength))
            } else {
                Ok(None)
            }
        });
        methods.add_method("insert", |lua, value_data, (evidence, index): (_, Option<usize>)| {
            let mut value_data_ref = if let Some(value_data) = value_data.borrow_mut() {
                value_data
            } else {
                return Ok(None);
            };
            let index = if let Some(orig_index) = index {
                let index = orig_index.checked_sub(1).unwrap();
                value_data_ref.evidence.insert(index, EvidenceBuilder::into(evidence));
                orig_index
            } else {
                let index = value_data_ref.evidence.len() + 1;
                value_data_ref.evidence.push(evidence.into());
                index
            };
            LuaEvidence {
                value_data: value_data.clone(),
                index,
                mutations: 0,
            }.new(lua).map(Some)
        });
        methods.add_meta_method("__len", |_, value_data, ()| {
            if let Some(value_data) = value_data.borrow_mut() {
                Ok(Some(value_data.evidence.len()))
            } else {
                Ok(None)
            }
        });
        methods.add_method("remove", |_, value_data, index: Option<usize>| {
            if let Some(mut value_data) = value_data.borrow_mut() {
                if let Some(index) = index {
                    value_data.evidence.remove(index.checked_sub(1).unwrap());
                } else {
                    value_data.evidence.pop();
                }
            }

            Ok(())
        });
        methods.add_meta_method("__index", |lua, value_data, index: usize| {
            LuaEvidence {
                value_data: value_data.clone(),
                index: index - 1,
                mutations: 0,
            }.new(lua)
        });
        methods.add_function("iter_evidence", |lua, value_data: AnyUserData| {
            let iter_function = lua.create_function(|lua, (value_data, index)| {
                if let Some(value_data) = LuaFacetValueData::borrow_mut(&value_data) {
                    if index >= value_data.evidence.len() {
                        return Nil.to_lua_multi(lua);
                    }
                } else {
                    return Nil.to_lua_multi(lua);
                }
                (
                    index + 1,
                    LuaEvidence {
                        value_data,
                        index,
                        mutations: 0,
                    }.new(lua)?,
                ).to_lua_multi(lua)
            })?;
            Ok((iter_function, value_data, 0_usize))
        });
    }
}

#[derive(Into, From)]
#[repr(transparent)]
pub(super) struct EvidenceKindBuilder(EvidenceKind<KnowledgeTypes>);

impl<'lua> ToLua<'lua> for EvidenceKindBuilder {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(0, 3)?;
        match self.0 {
            EvidenceKind::Statement {
                source,
                location,
            } => {
                table.set("kind", "statement")?;
                table.set("source", source)?;
                table.set("location", location)?;
            },
            EvidenceKind::Overheard {
                source,
                recipient,
                location,
            } => {
                table.set("kind", "overheard")?;
                table.set("source", source)?;
                table.set("recipient", recipient)?;
                table.set("location", location)?;
            },
            EvidenceKind::Observation {
                location,
            } => {
                table.set("kind", "observation")?;
                table.set("location", location)?;
            },
            EvidenceKind::Transference {
                reminded_of,
            } => {
                table.set("kind", "transference")?;
                table.set("reminded_of", reminded_of)?;
            },
            EvidenceKind::Confabulation => {
                table.set("kind", "confabulation")?;
            },
            EvidenceKind::Lie {
                recipient,
                location,
            } => {
                table.set("kind", "lie")?;
                table.set("recipient", recipient)?;
                table.set("location", location)?;
            },
            EvidenceKind::Implantation => {
                table.set("kind", "implantation")?;
            },
            EvidenceKind::Declaration {
                recipient,
                location,
            } => {
                table.set("kind", "declaration")?;
                table.set("recipient", recipient)?;
                table.set("location", location)?;
            },
            EvidenceKind::Mutation {
                previous,
            } => {
                table.set("kind", "mutation")?;
                let previous: Box<EvidenceBuilder> = unsafe { std::mem::transmute(previous) };
                table.set("previous", *previous)?;
            },
        }
        table.to_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for EvidenceKindBuilder {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        let table = match lua_value {
            Nil => {
                return Ok(EvidenceKind::Implantation.into());
            },
            LuaValue::String(s) if s == "implant" || s == "implantation" => {
                return Ok(EvidenceKind::Implantation.into());
            },
            LuaValue::String(s) if s == "confab" || s == "confabulation" => {
                return Ok(EvidenceKind::Confabulation.into());
            },
            LuaValue::Table(table) => table,
            _ => {
                return Err(LuaError::FromLuaConversionError {
                    from: "Value",
                    to: "EvidenceKindBuilder",
                    message: Some("expected to get a Table input but didn't".to_string()),
                });
            },
        };
        let kind: LuaString = table.get("kind")?;
        match kind.to_str()? {
            "statement" => Ok(EvidenceKind::Statement {
                source: table.get("source")?,
                location: table.get("location")?,
            }.into()),
            "overheard" => Ok(EvidenceKind::Overheard {
                source: table.get("source")?,
                recipient: table.get("recipient")?,
                location: table.get("location")?,
            }.into()),
            "observation" => Ok(EvidenceKind::Observation {
                location: table.get("location")?,
            }.into()),
            "transference" => Ok(EvidenceKind::Transference {
                reminded_of: table.get("reminded_of")?,
            }.into()),
            "confabulation" => Ok(EvidenceKind::Confabulation.into()),
            "lie" => Ok(EvidenceKind::Lie {
                recipient: table.get("recipient")?,
                location: table.get("location")?,
            }.into()),
            "implantation" => Ok(EvidenceKind::Implantation.into()),
            "declaration" => Ok(EvidenceKind::Declaration {
                recipient: table.get("recipient")?,
                location: table.get("location")?,
            }.into()),
            "mutation" => Ok(EvidenceKind::Mutation {
                previous: Box::new(table.get::<_, EvidenceBuilder>("previous")?.into()),
            }.into()),
            kind => Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "EvidenceKindBuilder",
                message: Some(format!("Unknown kind '{kind}'")),
            })
        }
    }
}

#[derive(Into, From)]
#[repr(transparent)]
pub(super) struct EvidenceBuilder(Evidence<KnowledgeTypes>);

impl<'lua> ToLua<'lua> for EvidenceBuilder {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(0, 3)?;
        table.set("data", self.0.data)?;
        table.set("kind", EvidenceKindBuilder::from(self.0.kind))?;
        table.set("strength", self.0.strength)?;
        table.to_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for EvidenceBuilder {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        let table = if let LuaValue::Table(table) = lua_value {
            table
        } else {
            return Err(LuaError::FromLuaConversionError {
                from: "Value",
                to: "EvidenceBuilder",
                message: Some("expected to get a Table input but didn't".to_string()),
            });
        };
        Ok(Evidence {
            data: table.get("data")?,
            kind: table.get::<_, EvidenceKindBuilder>("kind")?.into(),
            strength: table.get("strength")?,
        }.into())
    }
}
