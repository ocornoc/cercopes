use super::*;
pub use fav_music_genre::*;

mod fav_music_genre;

pub const MAKE_SMALL_TALK: DialogMove = {
    const X: u64 = const_random!(u64);
    DialogMove(X)
};

pub const STATE_FEELINGS_MOVE: DialogMove = {
    const X: u64 = const_random!(u64);
    DialogMove(X)
};

pub const STATE_FEELINGS_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn state_feelings_node() -> MoveNode {
    MoveNode {
        dialog_moves: [STATE_FEELINGS_MOVE, MAKE_SMALL_TALK].into_iter().collect(),
        addressed_topics: Default::default(),
        precondition: None.into(),
        edit_historical_move: EditHistoricalMove::new(|_, _, hmove| {
            hmove.get_my_obligations().addressed.insert(STATE_FEELINGS_MOVE);
        }),
        formatter: Box::new(|_, rng, _| {
            static WORDS: &[&str] = &[
                "happy",
                "sad",
                "angry",
                "upset",
                "ecstatic",
                "excited",
            ];
            format!("I feel {}.", WORDS.choose(rng).unwrap())
        }),
        parts: Vec::new(),
    }
}

pub const ASK_FEELINGS_MOVE: DialogMove = {
    const X: u64 = const_random!(u64);
    DialogMove(X)
};

pub const ASK_FEELINGS_SIMPLE_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn ask_feelings_simple_node() -> MoveNode {
    MoveNode {
        dialog_moves: [ASK_FEELINGS_MOVE, MAKE_SMALL_TALK].into_iter().collect(),
        addressed_topics: Default::default(),
        precondition: None.into(),
        edit_historical_move: EditHistoricalMove::new(|_, _, hmove| {
            hmove.get_others_obligations().push(STATE_FEELINGS_MOVE, 0, 3);
        }),
        formatter: Box::new(|_, _, _| {
            "How are you feeling?".to_string()
        }),
        parts: Vec::new(),
    }
}

pub const BORED_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn bored_node() -> MoveNode {
    MoveNode {
        dialog_moves: [MAKE_SMALL_TALK].into_iter().collect(),
        addressed_topics: Default::default(),
        precondition: None.into(),
        edit_historical_move: None.into(),
        formatter: Box::new(|_, _, _| {
            "This conversation bores me.".to_string()
        }),
        parts: Vec::new(),
    }
}

/*
pub const CLOTHING: Topic = {
    const X: u64 = const_random!(u64);
    Topic(X)
};
*/
