use super::*;

#[derive(Debug, Deref, DerefMut)]
pub(crate) struct LuaEvidenceKind(pub LuaEvidence);

impl LuaEvidenceKind {
    pub fn borrow_mut(&self) -> Option<RefMut<EvidenceKind<KnowledgeTypes>>> {
        Some(RefMut::map(self.0.borrow_mut()?, |evidence| &mut evidence.kind))
    }
}

macro_rules! lua_ek_field_is_ {
    ($fields:expr, $name:literal, $kind:pat) => {
        $fields.add_field_method_get(concat!("is_", $name), |_, evidence_kind| {
            Ok(matches!(evidence_kind.borrow_mut().as_deref(), Some($kind)))
        });
    };
}

macro_rules! lua_ek_field_get_set {
    ($fields:expr, $name:literal, $kind:pat, $result:tt) => {
        #[allow(unused_parens)] // Necessary because rust-analyzer freaks out on a | in the pattern
        $fields.add_field_method_get($name, |_, evidence_kind| {
            if let Some($kind) = evidence_kind.borrow_mut().as_deref() {
                Ok(Some($result.clone()))
            } else {
                Ok(None)
            }
        });
        #[allow(unused_parens)] // Necessary because rust-analyzer freaks out on a | in the pattern
        $fields.add_field_method_set($name, |_, evidence_kind, new_val| {
            if let Some($kind) = evidence_kind.borrow_mut().as_deref_mut() {
                *$result = new_val;
            }

            Ok(())
        });
    };
}

impl UserData for LuaEvidenceKind {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("evidence", |_, evidence_kind| {
            Ok(evidence_kind.0.clone())
        });
        lua_ek_field_is_!(fields, "statement", EvidenceKind::Statement {..});
        lua_ek_field_is_!(fields, "overheard", EvidenceKind::Overheard {..});
        lua_ek_field_is_!(fields, "observation", EvidenceKind::Observation {..});
        lua_ek_field_is_!(fields, "transference", EvidenceKind::Transference {..});
        lua_ek_field_is_!(fields, "confabulation", EvidenceKind::Confabulation);
        lua_ek_field_is_!(fields, "lie", EvidenceKind::Lie {..});
        lua_ek_field_is_!(fields, "implantation", EvidenceKind::Implantation);
        lua_ek_field_is_!(fields, "declaration", EvidenceKind::Declaration {..});
        lua_ek_field_is_!(fields, "mutation", EvidenceKind::Mutation {..});
        lua_ek_field_get_set! {
            fields, "source",
            (EvidenceKind::Statement { source, .. }
            | EvidenceKind::Overheard { source, .. }),
            source
        }
        lua_ek_field_get_set! {
            fields, "location",
            (EvidenceKind::Declaration { location, .. }
            | EvidenceKind::Lie { location, .. }
            | EvidenceKind::Observation { location }
            | EvidenceKind::Overheard { location, .. }
            | EvidenceKind::Statement { location, .. }),
            location
        }
        lua_ek_field_get_set! {
            fields, "recipient",
            (EvidenceKind::Declaration { recipient, .. }
            | EvidenceKind::Lie { recipient, .. }
            | EvidenceKind::Overheard { recipient, .. }),
            recipient
        }
        lua_ek_field_get_set! {
            fields, "reminded_of",
            EvidenceKind::Transference { reminded_of },
            reminded_of
        }
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("into_table", |_, evidence_kind, ()| {
            if let Some(evidence_kind) = evidence_kind.borrow_mut() {
                Ok(Some(EvidenceKindBuilder::from(evidence_kind.clone())))
            } else {
                Ok(None)
            }
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LuaEvidence {
    pub value_data: LuaFacetValueData,
    pub index: usize,
    pub mutations: usize,
}

impl LuaEvidence {
    pub fn new<'lua>(self, lua: &'lua Lua) -> LuaResult<AnyUserData<'lua>> {
        lua.create_userdata(self)
    }

    fn get_evidence(
        mut evidence: &mut Evidence<KnowledgeTypes>,
        mutations: usize,
    ) -> Option<&mut Evidence<KnowledgeTypes>> {
        for _ in 0..mutations {
            if let EvidenceKind::Mutation { ref mut previous } = evidence.kind {
                evidence = previous;
            } else {
                return None;
            }
        }

        Some(evidence)
    }

    pub fn borrow_mut(&self) -> Option<RefMut<Evidence<KnowledgeTypes>>> {
        let mut value_data = self.value_data.borrow_mut()?;
        let mut evidence = value_data.evidence.get_mut(self.index)?;
        evidence = Self::get_evidence(evidence, self.mutations)?;
        let evidence_ptr = evidence as *mut Evidence<KnowledgeTypes>;
        Some(RefMut::map(value_data, |_| unsafe { evidence_ptr.as_mut().unwrap_unchecked() }))
    }
}

impl UserData for LuaEvidence {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("value_data", |_, evidence| {
            Ok(evidence.value_data.clone())
        });
        fields.add_field_method_get("previous", |_, evidence| {
            if evidence.index > 0 {
                let evidence = LuaEvidence {
                    mutations: evidence.mutations - 1,
                    ..evidence.clone()
                };
                if evidence.borrow_mut().is_some() {
                    Ok(Some(evidence))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_get("data", |_, evidence| {
            if let Some(evidence) = evidence.borrow_mut() {
                Ok(Some(evidence.data.clone()))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_set("data", |_, evidence, data| {
            if let Some(mut evidence) = evidence.borrow_mut() {
                evidence.data = data;
            }

            Ok(())
        });
        fields.add_field_method_get("strength", |_, evidence| {
            if let Some(evidence) = evidence.borrow_mut() {
                Ok(Some(evidence.strength))
            } else {
                Ok(None)
            }
        });
        fields.add_field_method_set("strength", |_, evidence, strength| {
            if let Some(mut evidence) = evidence.borrow_mut() {
                evidence.strength = strength;
            }

            Ok(())
        });
        fields.add_field_method_get("kind", |_, evidence| {
            Ok(LuaEvidenceKind(evidence.clone()))
        });
        fields.add_field_method_set("kind", |_, evidence, kind| {
            if let Some(mut evidence) = evidence.borrow_mut() {
                evidence.kind = EvidenceKindBuilder::into(kind);
            }

            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("principal_evidence", |_, evidence, ()| {
            let mut mutations = evidence.mutations;
            if let Some(evidence) = evidence.borrow_mut() {
                let mut evidence = &*evidence;
                while let EvidenceKind::Mutation { ref previous } = evidence.kind {
                    evidence = previous;
                    mutations += 1;
                }
            } else {
                return Ok(None);
            }
            Ok(Some(LuaEvidence {
                mutations,
                ..evidence.clone()
            }))
        });
        methods.add_method("mutate", |_, evidence, ()| {
            if let Some(mut evidence) = evidence.borrow_mut() {
                evidence.mutate();
            }

            Ok(())
        });
        methods.add_method("into_table", |_, evidence, ()| {
            if let Some(evidence) = evidence.borrow_mut() {
                Ok(Some(EvidenceBuilder::from(evidence.clone())))
            } else {
                Ok(None)
            }
        });
    }
}
