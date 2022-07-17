use derive_more::Display;
use super::*;

pub mod small_talk;
pub mod greet;

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy)]
#[display = "topic #{_0}"]
pub struct Topic(u64);

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy)]
#[display = "expander node #{_0}"]
pub struct ExpanderNode(u64);

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy)]
#[display = "dialog move #{_0}"]
pub struct DialogMove(u64);

impl Default for DialogMove {
    fn default() -> Self {
        small_talk::MAKE_SMALL_TALK
    }
}

pub const NOTHING_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

fn nothing_node() -> MoveNode {
    MoveNode {
        dialog_moves: Default::default(),
        addressed_topics: Default::default(),
        precondition: Default::default(),
        edit_historical_move: Default::default(),
        formatter: Box::new(|_, _, _| String::new()),
        parts: Default::default(),
    }
}

pub fn manager_nodes() -> impl IntoIterator<Item = (ExpanderNode, MoveNode)> {
    [
        (NOTHING_NODE, nothing_node()),
        (greet::SIMPLE_GREET_NODE, greet::simple_greet_node()),
        (greet::SIMPLE_HELLO_NODE, greet::simple_hello_node()),
        (greet::SIMPLE_HI_NODE, greet::simple_hi_node()),
        (small_talk::BORED_NODE, small_talk::bored_node()),
        (small_talk::STATE_FEELINGS_NODE, small_talk::state_feelings_node()),
        (small_talk::ASK_FEELINGS_SIMPLE_NODE, small_talk::ask_feelings_simple_node()),
        (small_talk::RAW_FAV_MUSIC_GENRE_NODE, small_talk::raw_fav_music_genre()),
        (small_talk::STATE_FAV_MUSIC_GENRE_NODE, small_talk::state_fav_music_genre()),
        (small_talk::ASK_FAV_MUSIC_GENRE_NODE, small_talk::ask_fav_music_genre()),
    ]
}
