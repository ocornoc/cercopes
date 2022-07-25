//! placeholder

#![warn(missing_docs)]

use std::hash::Hash;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::error::Error;
use std::marker::PhantomData;
use rand::{prelude::*, distributions::Bernoulli};
use ahash::{AHashMap, AHashSet};
use serde::*;
use derive_more::*;
use dyn_clone::DynClone;
use _private::*;
pub use conversation::*;
use goal::*;
pub use expander::*;

mod conversation;
mod expander;
pub mod goal;

type Rng = rand_xoshiro::Xoroshiro64Star;

mod _private {
    #[cfg(not(feature = "send_sync"))]
    pub trait MaybeSend {}
    
    #[cfg(not(feature = "send_sync"))]
    impl<T: ?Sized> MaybeSend for T {}

    #[cfg(feature = "send_sync")]
    pub trait MaybeSend: Send {}

    #[cfg(feature = "send_sync")]
    impl<T: ?Sized + Send> MaybeSend for T {}

    #[cfg(not(feature = "send_sync"))]
    pub trait MaybeSync {}

    #[cfg(not(feature = "send_sync"))]
    impl<T: ?Sized> MaybeSync for T {}

    #[cfg(feature = "send_sync")]
    pub trait MaybeSync: Sync {}

    #[cfg(feature = "send_sync")]
    impl<T: ?Sized + Sync> MaybeSync for T {}

    pub trait MaybeSendSync: MaybeSend + MaybeSync {}
    
    impl<T: ?Sized + MaybeSend + MaybeSync> MaybeSendSync for T {}
}

#[cfg(feature = "send_sync")]
type PreconditionFn<D> = Option<Box<dyn Fn(&Conversation<D>) -> bool + Send + Sync>>;

#[cfg(not(feature = "send_sync"))]
type PreconditionFn<D> = Option<Box<dyn Fn(&Conversation<D>) -> bool>>;

/// A precondition for a [dialog move](DialogTrait::DialogMove) or [frame](Frame).
///
/// If it is `None`, then it is assumed to always be true.
#[derive(From, Into)]
pub struct Precondition<D: DialogTrait> {
    ///
    pub condition: PreconditionFn<D>,
}

impl<D: DialogTrait> Precondition<D> {
    /// Create a new precondition from a precondition function.
    pub fn new<F>(condition: F) -> Self
    where
        F: 'static + Fn(&Conversation<D>) -> bool + MaybeSendSync,
    {
        Precondition {
            condition: Some(Box::new(condition)),
        }
    }

    /// Evaluate the precondition for some [conversation](Conversation).
    pub fn check(&self, conversation: &Conversation<D>) -> bool {
        if let Some(ref condition) = self.condition {
            condition(conversation)
        } else {
            true
        }
    }
}

impl<D: DialogTrait> Debug for Precondition<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if let Some(ref condition) = self.condition {
            write!(f, "Precondition({condition:p})")
        } else {
            f.write_str("Precondition(always true)")
        }
    }
}

impl<D: DialogTrait> Default for Precondition<D> {
    fn default() -> Self {
        None.into()
    }
}

/// A trait used for collecting the relevant types for the conversation system,
pub trait DialogTrait {
    /// A topic for a [conversation](Conversation).
    ///
    /// This is used for keeping track of what has been addressed. For example, if a pair talked
    /// about the weather, it would be a bit strange to bring it up again in the same conversation.
    type Topic: Eq + Hash + Debug + Clone;

    /// A dialog move is an intention.
    ///
    /// Dialog moves do not necessarily represent exact words and instead more closely represent
    /// the *intentions* of words. For example, `greet` would be a dialog move, whereas "Hi!" and
    /// "Hello." would be utterances performing that dialog move.
    type DialogMove: Eq + Hash + Debug + Clone;

    /// An expander node identifier.
    ///
    /// Expander nodes used for turning [dialog moves](DialogTrait::DialogMove) into utterances.
    type ExpanderNode: Eq + Hash + Debug + Clone;

    /// Extra data that can be used for tracking character state or identifying participants.
    type Character;
}


pub struct DialogManager<D: DialogTrait> {
    expander: Expander<D>,
    /// The [frames](Frame) in this dialog manager.
    ///
    /// Modifying this list of frames may or may not have an effect on [conversations](Conversation)
    /// that were initialized prior to the modification.
    pub frames: Vec<Frame<D>>,
}

impl<D: DialogTrait> DialogManager<D> {
    /// Create a new dialog manager from a collection of [move nodes](MoveNode) and a collection of
    /// [frames](Frame).
    #[inline]
    pub fn new(
        expanders: impl IntoIterator<Item = (D::ExpanderNode, MoveNode<D>)>,
        frames: impl IntoIterator<Item = Frame<D>>,
    ) -> Self {
        DialogManager {
            expander: Expander::new(expanders.into_iter().collect()),
            frames: frames.into_iter().collect(),
        }
    }

    ///
    #[inline]
    pub fn rebuild_expander(&mut self) {
        self.expander.build();
    }

    /// Create a new [conversation](Conversation) using this dialog manager.
    #[inline]
    pub fn new_conversation(
        &self,
        initiator: Speaker,
        lull_continue_chance: Bernoulli,
        person0: D::Character,
        person1: D::Character,
    ) -> Conversation<D> {
        Conversation::new(initiator, &self.frames, lull_continue_chance, person0, person1)
    }

    fn attempt_to_make_move(
        &self,
        rng: &mut Rng,
        conversation: &mut Conversation<D>,
        dialog_move: &D::DialogMove,
    ) -> ExpanderResult<D, bool> {
        match self.expander.address_dialog_move(conversation, rng, dialog_move) {
            Ok(hmove) => {
                conversation.update_for_move(hmove);
                Ok(true)
            },
            Err(ExpanderErr::NoNodesSatisfyPreconditions) => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn attempt_to_address_topic(
        &self,
        rng: &mut Rng,
        conversation: &mut Conversation<D>,
        topic: &D::Topic
    ) -> ExpanderResult<D, bool> {
        match self.expander.address_topic(conversation, rng, topic) {
            Ok(hmove) => {
                conversation.update_for_move(hmove);
                Ok(true)
            },
            Err(ExpanderErr::NoNodesSatisfyPreconditions) => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn attempt_to_speak(
        &self,
        rng: &mut Rng,
        conversation: &mut Conversation<D>,
    ) -> ExpanderResult<D, bool> {
        for topic in conversation.get_next_speaker_topics(rng) {
            if self.attempt_to_address_topic(rng, conversation, &topic)? {
                return Ok(true);
            }
        }

        for dialog_move in conversation.get_next_speaker_moves(rng).into_iter().rev() {
            if self.attempt_to_make_move(rng, conversation, &dialog_move)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Step a [conversation](Conversation).
    ///
    /// 
    pub fn step_conversation(
        &self,
        conversation: &mut Conversation<D>,
        lull_move: &D::DialogMove,
    ) -> ExpanderResult<D, ()> {
        if conversation.done {
            return Ok(());
        }
        conversation.timestep();
        let mut rng = Rng::from_entropy();
        if self.attempt_to_speak(&mut rng, conversation)? || {
            conversation.speaker = !conversation.speaker;
            self.attempt_to_speak(&mut rng, conversation)?
        } {  // one of the participants managed to speak
            conversation.speaker = !conversation.speaker;
        } else if conversation.lull_continue_chance.sample(&mut rng) {
            // there's a lull in the conversation and we're continuing it
            if self.attempt_to_make_move(&mut rng, conversation, lull_move)? || {
                conversation.speaker = !conversation.speaker;
                self.attempt_to_make_move(&mut rng, conversation, lull_move)?
            } { // try to make small talk
                conversation.speaker = !conversation.speaker;
            } else { // nobody could make small talk
                conversation.done = true;
            }
        } else {
            conversation.done = true;
        }
        Ok(())
    }

    /// Get a reference to a [move node](MoveNode);
    #[inline]
    pub fn get_move_node(&self, expander_node: &D::ExpanderNode) -> Option<&MoveNode<D>> {
        self.expander.get_node(expander_node)
    }

    /// Get an exclusive reference to a [move node](MoveNode).
    ///
    /// # Safety
    ///
    /// If the caller modifies the move node, they must
    /// [rebuild the node expander](DialogManager::rebuild_expander).
    #[inline]
    pub unsafe fn get_move_node_mut(
        &mut self,
        expander_node: &D::ExpanderNode,
    ) -> Option<&mut MoveNode<D>> {
        self.expander.get_node_mut(expander_node)
    }

    /// Insert a new [move node](MoveNode) into the dialog manager.
    #[inline]
    pub fn insert_move_node(
        &mut self,
        expander_node: D::ExpanderNode,
        move_node: MoveNode<D>,
    ) -> Option<MoveNode<D>> {
        self.expander.insert(expander_node, move_node)
    }

    /// Remove a [move node](MoveNode) from the dialog manager.
    ///
    /// Returns the removed move node if there was one to remove.
    #[inline]
    pub fn remove_move_node(&mut self, expander_node: &D::ExpanderNode) -> Option<MoveNode<D>> {
        self.expander.remove(expander_node)
    }
}

impl<D: DialogTrait> Debug for DialogManager<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("DialogManager")
            .field("expander", &self.expander)
            .field("frames", &self.frames)
            .finish()
    }
}

impl<D: DialogTrait> Extend<Frame<D>> for DialogManager<D> {
    fn extend<T: IntoIterator<Item = Frame<D>>>(&mut self, iter: T) {
        self.frames.extend(iter);
    }
}

impl<D: DialogTrait> Extend<(D::ExpanderNode, MoveNode<D>)> for DialogManager<D> {
    fn extend<T: IntoIterator<Item = (D::ExpanderNode, MoveNode<D>)>>(&mut self, iter: T) {
        self.expander.extend(iter);
    }
}
