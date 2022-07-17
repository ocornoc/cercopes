use super::*;

pub const FAV_MUSIC_GENRE: Topic = {
    const X: u64 = const_random!(u64);
    Topic(X)
};

pub const RAW_FAV_MUSIC_GENRE_MOVE: DialogMove = {
    const X: u64 = const_random!(u64);
    DialogMove(X)
};

pub const RAW_FAV_MUSIC_GENRE_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn raw_fav_music_genre() -> MoveNode {
    MoveNode {
        dialog_moves: [RAW_FAV_MUSIC_GENRE_MOVE].into_iter().collect(),
        addressed_topics: Default::default(),
        precondition: None.into(),
        edit_historical_move: EditHistoricalMove::new(|_, _, hmove| {
            hmove.get_my_obligations().addressed.insert(STATE_FEELINGS_MOVE);
        }),
        formatter: Box::new(|state, _, _| {
            state.get_my_state().character.0.fav_music.to_string()
        }),
        parts: Vec::new(),
    }
}

pub const STATE_FAV_MUSIC_GENRE_MOVE: DialogMove = {
    const X: u64 = const_random!(u64);
    DialogMove(X)
};

pub const STATE_FAV_MUSIC_GENRE_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn state_fav_music_genre() -> MoveNode {
    MoveNode {
        dialog_moves: [STATE_FAV_MUSIC_GENRE_MOVE, MAKE_SMALL_TALK].into_iter().collect(),
        addressed_topics: [FAV_MUSIC_GENRE].into_iter().collect(),
        precondition: Precondition::new(|state| {
            state.get_my_state().pushed_obligations.contains_key(&STATE_FAV_MUSIC_GENRE_MOVE)
            || state.topic_state.can_be_addressed(&FAV_MUSIC_GENRE)
        }),
        edit_historical_move: EditHistoricalMove::new(|state: &mut ConversationState, _, hmove| {
            hmove.get_my_obligations().addressed.insert(STATE_FAV_MUSIC_GENRE_MOVE);
            hmove.topic_state.addressed.insert(FAV_MUSIC_GENRE);
            let speaker = hmove.speaker;
            let source = state.get_speaker_state(speaker).character.0.clone();
            let my_favorite_music_genre = source.fav_music;
            // this is just a placeholder, it's not important for the demonstration
            let location = source.clone();
            let others_model = state.get_speaker_state_mut(!speaker).character.1.as_mut();
            others_model.insert_evidence(my_favorite_music_genre.into(), Evidence {
                data: (),
                kind: EvidenceKind::Statement {
                    source,
                    location,
                },
                strength: 100.0,
            });
            others_model.recompute_total_strengths();
            others_model.recompute_strongest();
        }),
        formatter: Box::new(|_, _, parts| {
            format!("My favorite genre of music is {}.", parts[0])
        }),
        parts: vec![vec![RAW_FAV_MUSIC_GENRE_NODE]],
    }
}

pub const ASK_FAV_MUSIC_GENRE_MOVE: DialogMove = {
    const X: u64 = const_random!(u64);
    DialogMove(X)
};

pub const ASK_FAV_MUSIC_GENRE_NODE: ExpanderNode = {
    const X: u64 = const_random!(u64);
    ExpanderNode(X)
};

pub fn ask_fav_music_genre() -> MoveNode {
    MoveNode {
        dialog_moves: [ASK_FAV_MUSIC_GENRE_MOVE, MAKE_SMALL_TALK].into_iter().collect(),
        addressed_topics: Default::default(),
        precondition: Precondition::new(|state| {
            state.topic_state.can_be_introduced(&FAV_MUSIC_GENRE)
        }),
        edit_historical_move: EditHistoricalMove::new(|_, _, hmove| {
            hmove.get_others_obligations().push(
                STATE_FAV_MUSIC_GENRE_MOVE,
                0,
                3,
            );
            hmove.topic_state.introduced.insert(FAV_MUSIC_GENRE);
        }),
        formatter: Box::new(|_, _, _| {
            format!("What's your favorite genre of music?")
        }),
        parts: Default::default(),
    }
}
