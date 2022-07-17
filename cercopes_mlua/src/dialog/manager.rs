use rand::distributions::Bernoulli;
use super::*;

#[derive(Debug, Clone)]
pub(crate) struct LuaDialogManager(pub RcRef<DialogManager<DialogTypes>>);

impl LuaDialogManager {
    pub fn lua_new(lua: &Lua) -> LuaResult<Function<'_>> {
        lua.create_function(move |lua, (dialog_nodes, frames): (LuaTable, LuaTable)| {
            let dialog_nodes = dialog_nodes.pairs().collect::<LuaResult<Vec<_>>>()?;
            let frames = frames
                .sequence_values()
                .map(|frame| frame.and_then(|frame| LuaFrame::from_lua(frame, lua)))
                .map(|frame| frame.map(|frame| Frame {
                    state: Box::new(LuaFrame::modify(frame))
                }))
                .collect::<LuaResult<Vec<_>>>()?;
            Ok(LuaDialogManager(RcRef::new(DialogManager::new(
                dialog_nodes.into_iter().map(|(k, node)| (k, LuaMoveNode::into(node))),
                frames,
            ).into())))
        })
    }

    pub fn borrow(&self) -> Ref<DialogManager<DialogTypes>> {
        self.0.borrow()
    }

    #[allow(dead_code)]
    pub fn borrow_mut(&self) -> RefMut<DialogManager<DialogTypes>> {
        self.0.borrow_mut()
    }
}

impl UserData for LuaDialogManager {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("new_conversation", |
            _,
            manager,
            (initiator, lull_continue_chance, person0, person1),
        | {
            let manager = manager.borrow();
            let lull_continue_chance = Bernoulli::new(lull_continue_chance).unwrap();
            let conversation = manager.new_conversation(
                LuaSpeaker::into(initiator),
                lull_continue_chance,
                person0,
                person1,
            );
            Ok(LuaConversation(RcRef::new(conversation.into())))
        });
        methods.add_method("step_conversation", |_, manager, (conversation, lull_move)| {
            let manager = manager.borrow();
            let mut conversation = LuaConversation::borrow_mut(&conversation);
            manager.step_conversation(&mut conversation, &lull_move).unwrap();
            Ok(())
        });
    }
}
