use super::*;

///
#[derive(Serialize, Deserialize)]
#[serde(bound(
    serialize = "D::Topic: Serialize, D::DialogMove: Serialize",
    deserialize = "D::Topic: for<'a> Deserialize<'a>, D::DialogMove: for<'a> Deserialize<'a>",
))]
pub enum ExpanderErr<D: DialogTrait> {
    ///
    NoExpanderForMove(D::DialogMove),
    ///
    NoExpanderForTopic(D::Topic),
    ///
    NoNodesSatisfyPreconditions,
}

impl<D: DialogTrait> Debug for ExpanderErr<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::NoExpanderForMove(arg0) => f
                .debug_tuple("NoExpanderForMove")
                .field(arg0)
                .finish(),
            Self::NoExpanderForTopic(arg0) => f
                .debug_tuple("NoExpanderForTopic")
                .field(arg0)
                .finish(),
            Self::NoNodesSatisfyPreconditions => write!(f, "NoNodesSatisfyPreconditions"),
        }
    }
}

impl<D: DialogTrait> Display for ExpanderErr<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ExpanderErr::NoExpanderForMove(dialog_move) =>
                write!(f, "There is no expander node satisfying dialog move '{:?}'", dialog_move),
            ExpanderErr::NoExpanderForTopic(topic) =>
                write!(f, "There is no expander node satisfying topic '{:?}'", topic),
            ExpanderErr::NoNodesSatisfyPreconditions =>
                write!(f, "No candidate expander nodes satisfy their preconditions"),
        }
    }
}

impl<D: DialogTrait> Error for ExpanderErr<D> {}

impl<D: DialogTrait> PartialEq for ExpanderErr<D> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NoExpanderForMove(l0), Self::NoExpanderForMove(r0)) => l0 == r0,
            (Self::NoExpanderForTopic(l0), Self::NoExpanderForTopic(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

///
pub type ExpanderResult<D, T> = Result<T, ExpanderErr<D>>;

#[cfg(feature = "send_sync")]
type EditHMF<D> = Box<
    dyn Fn(&mut Conversation<D>, &mut Rng, &mut HistoricalMove<D>) + Send + Sync
>;
#[cfg(not(feature = "send_sync"))]
type EditHMF<D> = Box<dyn Fn(&mut Conversation<D>, &mut Rng, &mut HistoricalMove<D>)>;

/// A function to edit a [historical move](HistoricalMove) when a particular [expander](MoveNode) is
/// chosen.
#[derive(From, Into)]
pub struct EditHistoricalMove<D: DialogTrait> {
    ///
    pub edit: Option<EditHMF<D>>,
}

impl<D: DialogTrait> EditHistoricalMove<D> {
    ///
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: 'static + Fn(&mut Conversation<D>, &mut Rng, &mut HistoricalMove<D>)
            + MaybeSendSync,
    {
        EditHistoricalMove {
            edit: Some(Box::new(f))
        }
    }
}

impl<D: DialogTrait> Default for EditHistoricalMove<D> {
    fn default() -> Self {
        Self { edit: None }
    }
}

impl<D: DialogTrait> Debug for EditHistoricalMove<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("EditHistoricalMove")
            .field("edit", &format!("{:p}", &self.edit))
            .finish()
    }
}

#[cfg(feature = "send_sync")]
/// For a [move node](MoveNode) that is being executed, this takes the [conversation](Conversation)
/// and the utterances generated by the move node parts to generate a new utterance.
pub type MoveNodeFormatter<D> = Box<
    dyn Fn(&Conversation<D>, &mut Rng, Vec<String>) -> String + Send + Sync
>;
#[cfg(not(feature = "send_sync"))]
/// For a [move node](MoveNode) that is being executed, this takes the [conversation](Conversation)
/// and the utterances generated by the move node parts to generate a new utterance.
pub type MoveNodeFormatter<D> = Box<dyn Fn(&Conversation<D>, &mut Rng, Vec<String>) -> String>;

/// A struct describing the behavior of an expander node.
pub struct MoveNode<D: DialogTrait> {
    /// The set of [dialog moves](DialogTrait::DialogMove) that this node addresses.
    pub dialog_moves: AHashSet<D::DialogMove>,
    /// The set of [topics](DialogTrait::Topic) that this node addresses.
    pub addressed_topics: AHashSet<D::Topic>,
    /// A precondition on this node, determining whether it can be taken or not.
    pub precondition: Precondition<D>,
    /// See [`EditHistoricalMove`].
    pub edit_historical_move: EditHistoricalMove<D>,
    /// See [`MoveNodeFormatter`].
    pub formatter: MoveNodeFormatter<D>,
    /// The nodes that are used to create an utterance from this node.
    ///
    /// `parts` is an `AND` of `OR`s; each [`Vec`] in `parts` represents a single part and each node
    /// in that [`Vec`] is a possible [expander node](DialogTrait::ExpanderNode) to fulfill that
    /// part. This is then passed to the [formatter](MoveNodeFormatter) to generate an utterance. It
    /// is undefined (but not Rust "undefined behavior") what happens if there are no choices for a
    /// part.
    pub parts: Vec<Vec<D::ExpanderNode>>,
}

impl<D: DialogTrait> Debug for MoveNode<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("MoveNode")
            .field("dialog_moves", &self.dialog_moves)
            .field("addressed_topics", &self.addressed_topics)
            .field("precondition", &self.precondition)
            .field("edit_historical_move", &self.edit_historical_move)
            .field("formatter", &format!("{:p}", &self.formatter))
            .field("parts", &self.parts)
            .finish()
    }
}

pub(crate) struct Expander<D: DialogTrait> {
    expander_nodes: AHashMap<D::ExpanderNode, MoveNode<D>>,
    addressing_move: AHashMap<D::DialogMove, AHashSet<D::ExpanderNode>>,
    addressing_topic: AHashMap<D::Topic, AHashSet<D::ExpanderNode>>,
    apart_of: AHashMap<D::ExpanderNode, AHashSet<D::ExpanderNode>>,
}

impl<D: DialogTrait> Expander<D> {
    pub fn build(&mut self) {
        self.addressing_move.clear();
        self.addressing_topic.clear();
        self.apart_of.clear();
        let make_hashset = || AHashSet::with_capacity(10);
        for (i, node) in self.expander_nodes.iter() {
            for dialog_move in node.dialog_moves.iter() {
                self.addressing_move
                    .entry(dialog_move.clone())
                    .or_insert_with(make_hashset)
                    .insert(i.clone());
            }
            for topic in node.addressed_topics.iter() {
                self.addressing_topic
                    .entry(topic.clone())
                    .or_insert_with(make_hashset)
                    .insert(i.clone());
            }
            for part in node.parts.iter() {
                for part_choice in part.iter() {
                    self.apart_of
                        .entry(part_choice.clone())
                        .or_insert_with(make_hashset)
                        .insert(i.clone());
                }
            }
        }
    }

    pub fn new(dialog_nodes: AHashMap<D::ExpanderNode, MoveNode<D>>) -> Self {
        let mut expander = Expander {
            expander_nodes: dialog_nodes,
            addressing_move: AHashMap::with_capacity(100),
            addressing_topic: AHashMap::with_capacity(100),
            apart_of: AHashMap::with_capacity(100),
        };
        expander.build();
        expander
    }

    fn expand_dialog_move(
        &self,
        conversation: &Conversation<D>,
        rng: &mut Rng,
        dialog_move: &D::DialogMove,
    ) -> ExpanderResult<D, ExpansionTree<D>> {
        let satisfying = self.addressing_move[dialog_move]
            .iter()
            .map(|node| ExpansionTree {
                expander_node: node,
                move_node: &self.expander_nodes[node],
                parts: Vec::new(),
            })
            .collect::<Vec<_>>();
        if satisfying.is_empty() {
            Err(ExpanderErr::NoExpanderForMove(dialog_move.clone()))
        } else {
            self.expand_satisfying(conversation, rng, satisfying)
        }
    }

    fn expand_topic(
        &self,
        conversation: &Conversation<D>,
        rng: &mut Rng,
        topic: &D::Topic,
    ) -> ExpanderResult<D, ExpansionTree<D>> {
        let satisfying = self.addressing_topic[topic]
            .iter()
            .map(|node| ExpansionTree {
                expander_node: node,
                move_node: &self.expander_nodes[node],
                parts: Vec::new(),
            })
            .collect::<Vec<_>>();
        if satisfying.is_empty() {
            Err(ExpanderErr::NoExpanderForTopic(topic.clone()))
        } else {
            self.expand_satisfying(conversation, rng, satisfying)
        }
    }

    fn expand_satisfying<'a>(
        &'a self,
        conversation: &Conversation<D>,
        rng: &mut Rng,
        mut satisfying: Vec<ExpansionTree<'a, D>>,
    ) -> ExpanderResult<D, ExpansionTree<'a, D>> {
        satisfying.shuffle(rng);
        for mut tree in satisfying {
            if !tree.move_node.precondition.check(conversation) {
                continue;
            } else if self.forward_chain(conversation, rng, &mut tree, None).is_ok() {
                return Ok(tree);
            }
        }
        Err(ExpanderErr::NoNodesSatisfyPreconditions)
    }

    fn forward_chain<'a>(
        &'a self,
        conversation: &Conversation<D>,
        rng: &mut Rng,
        tree: &mut ExpansionTree<'a, D>,
        skip: Option<usize>,
    ) -> ExpanderResult<D, ()> {
        'next_part: for part in tree.move_node.parts.iter() {
            let mut part_ids = (0..part.len())
                .filter(|&i| Some(i) != skip)
                .collect::<Vec<_>>();
            part_ids.shuffle(rng);
            for choice in part_ids.into_iter() {
                let choice = &part[choice];
                let move_node = &self.expander_nodes[choice];
                if move_node.precondition.check(conversation) {
                    let mut part = ExpansionTree {
                        expander_node: choice,
                        move_node,
                        parts: Vec::new(),
                    };
                    self.forward_chain(conversation, rng, &mut part, None)?;
                    tree.parts.push(part);
                    continue 'next_part;
                }
            }

            return Err(ExpanderErr::NoNodesSatisfyPreconditions);
        }

        Ok(())
    }

    fn is_top_level(&self, node: &D::ExpanderNode) -> bool {
        if let Some(apart) = self.apart_of.get(node) {
            apart.is_empty()
        } else {
            true
        }
    }

    fn backward_chain<'a>(
        &'a self,
        conversation: &Conversation<D>,
        rng: &mut Rng,
        tree: &mut ExpansionTree<'a, D>,
    ) -> ExpanderResult<D, ()> {
        if self.is_top_level(&tree.expander_node) {
            return Ok(());
        }

        for parent_expander in self.apart_of[&tree.expander_node].iter() {
            let parent_node = &self.expander_nodes[parent_expander];
            if !parent_node.precondition.check(conversation) {
                continue;
            }
            for (part_id, parts) in parent_node.parts.iter().enumerate() {
                if !parts.contains(&tree.expander_node) {
                    continue;
                }
                let child = std::mem::replace(tree, ExpansionTree {
                    expander_node: parent_expander,
                    move_node: parent_node,
                    parts: Vec::new(),
                });
                self.forward_chain(conversation, rng, tree, Some(part_id))?;
                tree.parts.insert(part_id, child);
                return Ok(());
            }
        }

        Err(ExpanderErr::NoNodesSatisfyPreconditions)
    }

    fn create_historical(
        &self,
        conversation: &mut Conversation<D>,
        rng: &mut Rng,
        tree: ExpansionTree<D>,
    ) -> HistoricalMove<D> {
        let mut historical_move = HistoricalMove {
            utterance: tree.create_utterance(conversation, rng),
            speaker: conversation.speaker,
            person0_obligations: HistoricalObligations {
                pushed: AHashMap::with_capacity(5),
                addressed: AHashSet::with_capacity(10),
            },
            person1_obligations: HistoricalObligations {
                pushed: AHashMap::with_capacity(5),
                addressed: AHashSet::with_capacity(10),
            },
            topic_state: TopicState::default(),
        };
        tree.create_historical(conversation, rng, &mut historical_move);
        historical_move
    }

    fn address_tree<'a>(
        &'a self,
        conversation: &mut Conversation<D>,
        rng: &mut Rng,
        mut tree: ExpansionTree<'a, D>,
    ) -> ExpanderResult<D, HistoricalMove<D>> {
        self.backward_chain(conversation, rng, &mut tree)?;
        Ok(self.create_historical(conversation, rng, tree))
    }

    #[inline]
    pub fn address_dialog_move(
        &self,
        conversation: &mut Conversation<D>,
        rng: &mut Rng,
        dialog_move: &D::DialogMove,
    ) -> ExpanderResult<D, HistoricalMove<D>> {
        let tree = self.expand_dialog_move(conversation, rng, dialog_move)?;
        self.address_tree(conversation, rng, tree)
    }

    #[inline]
    pub fn address_topic(
        &self,
        conversation: &mut Conversation<D>,
        rng: &mut Rng,
        topic: &D::Topic,
    ) -> ExpanderResult<D, HistoricalMove<D>> {
        let tree = self.expand_topic(conversation, rng, topic)?;
        self.address_tree(conversation, rng, tree)
    }

    #[inline]
    pub fn get_node(&self, expander_node: &D::ExpanderNode) -> Option<&MoveNode<D>> {
        self.expander_nodes.get(expander_node)
    }

    #[inline]
    pub unsafe fn get_node_mut(
        &mut self,
        expander_node: &D::ExpanderNode,
    ) -> Option<&mut MoveNode<D>> {
        self.expander_nodes.get_mut(expander_node)
    }

    #[inline]
    pub fn insert(
        &mut self,
        expander_node: D::ExpanderNode,
        move_node: MoveNode<D>,
    ) -> Option<MoveNode<D>> {
        let out = self.expander_nodes.insert(expander_node, move_node);
        self.build(); // todo: we can make this far more efficient. we may not need to rebuild
        out
    }

    #[inline]
    pub fn remove(&mut self, expander_node: &D::ExpanderNode) -> Option<MoveNode<D>> {
        let out = self.expander_nodes.remove(expander_node);
        self.build(); // todo: we can make this far more efficient. we may not need to rebuild
        out
    }
}

impl<D: DialogTrait> Debug for Expander<D> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("Expander")
            .field("dialog_nodes", &self.expander_nodes)
            .field("addressing_move", &self.addressing_move)
            .field("addressing_topic", &self.addressing_topic)
            .finish()
    }
}

impl<D: DialogTrait> Extend<(D::ExpanderNode, MoveNode<D>)> for Expander<D> {
    fn extend<T: IntoIterator<Item = (D::ExpanderNode, MoveNode<D>)>>(&mut self, iter: T) {
        self.expander_nodes.extend(iter);
        self.build();
    }
}

struct ExpansionTree<'a, D: DialogTrait> {
    expander_node: &'a D::ExpanderNode,
    move_node: &'a MoveNode<D>,
    parts: Vec<Self>,
}

impl<D: DialogTrait> ExpansionTree<'_, D> {
    fn create_utterance(&self, conversation: &Conversation<D>, rng: &mut Rng) -> String {
        let parts = self.parts
            .iter()
            .map(|part| part.create_utterance(conversation, rng))
            .collect();
        (self.move_node.formatter)(conversation, rng, parts)
    }

    fn create_historical(
        self,
        conversation: &mut Conversation<D>,
        rng: &mut Rng,
        historical_move: &mut HistoricalMove<D>,
    ) {
        for part in self.parts {
            part.create_historical(conversation, rng, historical_move);
        }

        if conversation.speaker == Speaker::Person0 {
            &mut historical_move.person0_obligations.addressed
        } else {
            &mut historical_move.person1_obligations.addressed
        }.extend(self.move_node.dialog_moves.iter().cloned());
        historical_move.topic_state.addressed.extend(
            self.move_node.addressed_topics.iter().cloned(),
        );

        if let Some(ref edit) = self.move_node.edit_historical_move.edit {
            edit(conversation, rng, historical_move);
        }
    }
}
