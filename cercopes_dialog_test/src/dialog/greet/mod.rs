use super::*;

pub const GREET: DialogMove = {
    const X: u64 = const_random!(u64);
    DialogMove(X)
};

pub const SIMPLE_HELLO_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn simple_hello_node() -> MoveNode {
    MoveNode {
        dialog_moves: Default::default(),
        addressed_topics: Default::default(),
        precondition: None.into(),
        edit_historical_move: None.into(),
        formatter: Box::new(|_, _, _| {
            "Hello".to_string()
        }),
        parts: Vec::new(),
    }
}

pub const SIMPLE_HI_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn simple_hi_node() -> MoveNode {
    MoveNode {
        dialog_moves: Default::default(),
        addressed_topics: Default::default(),
        precondition: None.into(),
        edit_historical_move: None.into(),
        formatter: Box::new(|_, _, _| {
            "Hi".to_string()
        }),
        parts: Vec::new(),
    }
}

pub const SIMPLE_GREET_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn simple_greet_node() -> MoveNode {
    MoveNode {
        dialog_moves: [GREET].into_iter().collect(),
        addressed_topics: Default::default(),
        precondition: None.into(),
        edit_historical_move: None.into(),
        formatter: Box::new(|_, _, parts| {
            let mut out = parts.join("");
            out += ".";
            out
        }),
        parts: vec![vec![SIMPLE_HELLO_NODE, SIMPLE_HI_NODE]],
    }
}
