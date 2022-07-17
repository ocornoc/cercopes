//! placeholder

#![warn(missing_docs)]
use std::hash::Hash;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use ahash::AHashMap;
use derive_more::{From, TryInto};
use rand::prelude::*;

pub trait Entity<K: KnowledgeTrait>: Debug + Clone {
    /// Get all possible belief facets that can be held for entity.
    ///
    /// For example, a building would have the 'wall color' and 'building type' facets, but
    /// wouldn't have a 'hair color' facet (probably).
    ///
    /// Importantly, __this is assumed to be constant__: the set of possible facets will never
    /// change. If some facets might not have [values](KnowledgeTrait::FacetValue) then it is up to
    /// the implementer of the [`FacetValue`](KnowledgeTrait::FacetValue) to represent this, such as
    /// with an `Option`.
    fn relevant_facets(&self) -> Vec<K::Facet>;

    /// Return whether a facet is relevant to an entity.
    ///
    /// This must agree with [`Entity::relevant_facets`].
    fn is_facet_relevant(&self, facet: &K::Facet) -> bool {
        self.relevant_facets().contains(facet)
    }

    /// Get the true value of a facet, if possible the facet is relevant to the entity.
    ///
    /// For every [relevant facet](Entity::is_facet_relevant), this must return `Some(_)`, and
    /// `None` otherwise.
    fn facet_truth(&self, facet: &K::Facet) -> Option<K::FacetValue>;
}

pub trait Facet<K: KnowledgeTrait>: Eq + Hash + Debug + Clone {
    /// Get all possible belief values for this facet.
    ///
    /// This needn't be exhaustive and simply initializes evidence models.
    fn initial_values(&self) -> Vec<K::FacetValue>;
}

pub trait FacetValue<K: KnowledgeTrait>: Eq + Hash + Debug + Clone {
    /// Get the facet of this value.
    fn facet(&self) -> K::Facet;

    /// Attempt to mutate a piece of evidence for a particular facet value into another facet value
    /// **of the same facet**.
    ///
    /// Returns `Some(new_belief)` if the piece of evidence is mutated into supporting a new value,
    /// and returns `None` if no mutation happens.
    ///
    /// `self` is the belief value that `evidence` supports, and both are guaranteed to be present
    /// in the `model`.
    fn try_mutate<R: Rng>(
        &self,
        model: &EvidenceModel<K>,
        evidence: &Evidence<K>,
        rng: &mut R,
    ) -> Option<Self>;
}

pub trait KnowledgeTrait: Sized {
    type Facet: Facet<Self>;

    type FacetValue: FacetValue<Self>;

    type Entity: Entity<Self>;

    type Data: Debug + Clone;
}

/// The kind of evidence and evidence data.
pub enum EvidenceKind<K: KnowledgeTrait> {
    /// A statement is something that was said *to the holder*.
    ///
    /// This "statement" could actually be a lie; there is no inherent truth to this evidence kind.
    Statement {
        /// The entity that made the statement.
        source: K::Entity,
        /// Where the holder was when the statement was made.
        location: K::Entity,
    },
    /// An overheard statement (or lie).
    Overheard {
        /// The entity that made the statement.
        source: K::Entity,
        /// The entity that the statement was made to.
        recipient: K::Entity,
        /// Where the holder was when the statement was made.
        location: K::Entity,
    },
    /// Evidence that was gained via directly observing another entity.
    Observation {
        /// Where the holder was when the entity was observed.
        location: K::Entity,
    },
    /// Evidence accidentally gained by having been reminded of another entity that is similiar.
    Transference {
        /// The entity that the holder was reminded of.
        reminded_of: K::Entity,
    },
    /// Evidence accidentally gained by making probabilistic assumptions about the distribution of
    /// particular attributes in the relevant community.
    ///
    /// For example, if I live in a town where 90% of people have black hair, I'd likely
    /// *unconsciously* assume that everyone has black hair (until I observe otherwise).
    Confabulation,
    /// A statement that the holder made that the holder knows isn't (necessarily) true.
    ///
    /// This is included to model the real-world effect of how repeating a lie enough times can
    /// make the lie seem true to the speaker.
    Lie {
        /// The entity that received the lie.
        recipient: K::Entity,
        /// Where the holder told the lie.
        location: K::Entity,
    },
    /// Artificial knowledge that was implanted during initialization or by other means.
    Implantation,
    /// A statment that the holder made that the holder thinks is true.
    ///
    /// This is included to model the real-world effect of how communicating a belief reinforces
    /// that belief.
    Declaration {
        // The entity that the statement was made to.
        recipient: K::Entity,
        /// Where the holder was when the statement was made.
        location: K::Entity,
    },
    /// Misremembered evidence.
    Mutation {
        /// The piece of evidence that this was mutated from.
        ///
        /// The entity only has access to the kind of evidence that this was originally, which is
        /// invariant to mutation.
        previous: Box<Evidence<K>>,
    },
}

impl<K: KnowledgeTrait> Clone for EvidenceKind<K> {
    fn clone(&self) -> Self {
        match self {
            Self::Statement { source, location } => Self::Statement {
                source: source.clone(),
                location: location.clone(),
            },
            Self::Overheard { source, recipient, location } => Self::Overheard {
                source: source.clone(),
                recipient: recipient.clone(),
                location: location.clone(),
            },
            Self::Observation { location } => Self::Observation {
                location: location.clone(),
            },
            Self::Transference { reminded_of } => Self::Transference {
                reminded_of: reminded_of.clone(),
            },
            Self::Confabulation => Self::Confabulation,
            Self::Lie { recipient, location } => Self::Lie {
                recipient: recipient.clone(),
                location: location.clone(),
            },
            Self::Implantation => Self::Implantation,
            Self::Declaration { recipient, location } => Self::Declaration {
                recipient: recipient.clone(),
                location: location.clone(),
            },
            Self::Mutation { previous } => Self::Mutation {
                previous: previous.clone(),
            },
        }
    }
}

impl<K: KnowledgeTrait> Debug for EvidenceKind<K> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Statement { source, location } => f
                .debug_struct("Statement")
                .field("source", source)
                .field("location", location)
                .finish(),
            Self::Overheard { source, recipient, location } => f
                .debug_struct("Overheard")
                .field("source", source)
                .field("recipient", recipient)
                .field("location", location)
                .finish(),
            Self::Observation { location } => f
                .debug_struct("Observation")
                .field("location", location)
                .finish(),
            Self::Transference { reminded_of } => f
                .debug_struct("Transference")
                .field("reminded_of", reminded_of)
                .finish(),
            Self::Confabulation => write!(f, "Confabulation"),
            Self::Lie { recipient, location } => f
                .debug_struct("Lie")
                .field("recipient", recipient)
                .field("location", location)
                .finish(),
            Self::Implantation => write!(f, "Implantation"),
            Self::Declaration { recipient, location } => f
                .debug_struct("Declaration").field("recipient", recipient)
                .field("location", location)
                .finish(),
            Self::Mutation { previous } => f
                .debug_struct("Mutation")
                .field("previous", previous)
                .finish(),
        }
    }
}

/// A particular piece of evidence.
pub struct Evidence<K: KnowledgeTrait> {
    /// Any extra data associated with this evidence.
    pub data: K::Data,
    /// The kind of evidence this represents.
    pub kind: EvidenceKind<K>,
    /// How strong this piece of evidence is.
    pub strength: f32,
}

impl<K: KnowledgeTrait> Evidence<K> {
    /// Get the "real" kind of the evidence.
    ///
    /// Mutation isn't a real evidence kind and keeps a reference to the original piece of evidence.
    pub fn principal_kind(&self) -> &EvidenceKind<K> {
        let mut kind = &self.kind;
        while let EvidenceKind::Mutation { previous } = kind {
            kind = &previous.kind;
        }
        kind
    }

    pub fn mutate(&mut self) {
        self.kind = EvidenceKind::Mutation { previous: Box::new(self.clone()) };
    }

    pub fn mutated(mut self) -> Self {
        self.mutate();
        self
    }
}

impl<K: KnowledgeTrait> Clone for Evidence<K> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            kind: self.kind.clone(),
            strength: self.strength.clone(),
        }
    }
}

impl<K: KnowledgeTrait> Debug for Evidence<K> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("Evidence")
            .field("time", &self.data)
            .field("kind", &self.kind)
            .field("strength", &self.strength)
            .finish()
    }
}

/// The evidence data for a particular belief value.
pub struct FacetValueData<K: KnowledgeTrait> {
    /// All of the evidence for this value.
    pub evidence: Vec<Evidence<K>>,
    /// The total strength of this value.
    pub total_strength: f32,
}

impl<K: KnowledgeTrait> FacetValueData<K> {
    /// Update the `total_strength` field.
    pub fn recompute_total_strength(&mut self) {
        self.total_strength = 0.0;

        for e in self.evidence.iter() {
            self.total_strength += e.strength;
        }
    }

    /// Extend some value data with another facet value's data.
    ///
    /// This will leave the other facet value's data "empty" (0 strength, no evidence). This will
    /// **not** recompute the total strength.
    pub fn extend(&mut self, other: &mut Self) {
        self.evidence.append(&mut other.evidence);
        other.total_strength = 0.0;
    }

    /// Remove a particular piece of evidence and return it.
    ///
    /// This will **not** recompute the total strength.
    pub fn remove_evidence(&mut self, i: usize) -> Evidence<K> {
        self.evidence.swap_remove(i)
    }
}

impl<K: KnowledgeTrait> Clone for FacetValueData<K> {
    fn clone(&self) -> Self {
        Self {
            evidence: self.evidence.clone(),
            total_strength: self.total_strength.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.evidence.clone_from(&source.evidence);
        self.total_strength.clone_from(&source.total_strength);
    }
}

impl<K: KnowledgeTrait> Default for FacetValueData<K> {
    fn default() -> Self {
        Self {
            evidence: Vec::new(),
            total_strength: 0.0,
        }
    }
}

impl<K: KnowledgeTrait> Debug for FacetValueData<K> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("FacetValueData")
            .field("evidence", &self.evidence)
            .field("total_strength", &self.total_strength)
            .finish()
    }
}

/// The evidence supporting a particular facet value.
pub struct FacetData<K: KnowledgeTrait> {
    /// The true value of this facet, regardless of what the holder believes.
    pub truth: K::FacetValue,
    /// The strongest-held belief value of the facet.
    pub strongest: Option<K::FacetValue>,
    /// All of the evidence for all of the facet values.
    pub values: AHashMap<K::FacetValue, FacetValueData<K>>
}

impl<K: KnowledgeTrait> FacetData<K> {
    /// Initialize the facet data.
    pub fn new(regarding: &K::Entity, facet: &K::Facet) -> Self {
        let mut data = FacetData {
            truth: regarding
                .facet_truth(facet)
                .expect("Regarding did not have a true value for a relevant facet!"),
            strongest: None,
            values: facet
                .initial_values()
                .into_iter()
                .map(|value| (value, Default::default()))
                .collect(),
        };
        data.recompute_total_strengths();
        data.recompute_strongest();
        data
    }

    /// Recompute all of the total strengths for all values.
    pub fn recompute_total_strengths(&mut self) {
        for value in self.values.values_mut() {
            value.recompute_total_strength();
        }
    }

    /// Recompute which belief is the strongest-held one.
    pub fn recompute_strongest(&mut self) {
        let mut max: Option<(&K::FacetValue, f32)> = None;

        for (value, data) in self.values.iter() {
            if data.total_strength > 0.0 {
                if let Some(ref mut max_str) = max {
                    if max_str.1 < data.total_strength {
                        *max_str = (value, data.total_strength);
                    }
                } else {
                    max = Some((value, data.total_strength));
                }
            }
        }

        self.strongest = max.map(|(value, _)| value.clone());
    }

    /// Get the belief value data (or initialize it if necessary).
    pub fn get_value_data(&mut self, value: K::FacetValue) -> &mut FacetValueData<K> {
        debug_assert_eq!(self.truth.facet(), value.facet());
        self.values.entry(value).or_default()
    }

    /// Update the true facet value for the regarded entity.
    pub fn update_truth(&mut self, regarding: &K::Entity) {
        self.truth = regarding
            .facet_truth(&self.truth.facet())
            .expect("Regarding did not have a true value for a relevant facet!");
    }

    /// Merge the data from one value into another.
    ///
    /// Leaves `take_from` empty and with no strength. Does compute `value` and `take_from`
    /// strengths, but doesn't update strongest.
    pub fn merge_values(&mut self, value: K::FacetValue, take_from: K::FacetValue) {
        if value == take_from {
            return;
        }

        let mut take_from = std::mem::take(self.get_value_data(take_from));
        let value = self.get_value_data(value);
        value.extend(&mut take_from);
        value.recompute_total_strength();
    }
}

impl<K: KnowledgeTrait> Clone for FacetData<K> {
    fn clone(&self) -> Self {
        Self {
            truth: self.truth.clone(),
            strongest: self.strongest.clone(),
            values: self.values.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.truth.clone_from(&source.truth);
        self.strongest.clone_from(&source.strongest);
        self.values.clone_from(&source.values);
    }
}

impl<K: KnowledgeTrait> Debug for FacetData<K> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("FacetData")
            .field("truth", &self.truth)
            .field("strongest", &self.strongest)
            .field("values", &self.values)
            .finish()
    }
}

pub struct ReflexiveModel<K: KnowledgeTrait> {
    /// The entity that has the mental model.
    pub holder: K::Entity,
    /// All of the facets of the entity.
    ///
    /// For example, I always know my own hair color, no matter what any other person tells me.
    pub facets: AHashMap<K::Facet, K::FacetValue>,
    //id: UniqueId,
}

impl<K: KnowledgeTrait> ReflexiveModel<K> {
    /// Create and initialize a new reflexive mental model.
    pub fn new(holder: K::Entity) -> Self {
        ReflexiveModel {
            facets: holder
                .relevant_facets()
                .into_iter()
                .filter_map(|facet| {
                    let data = holder
                        .facet_truth(&facet)
                        .expect("Holder did not have a true value for a relevant facet!");
                    Some((facet, data))
                })
                .collect(),
            holder,
            //id: unique_u64(),
        }
    }

    /// Update the true facet values for this model.
    pub fn update_truths(&mut self) {
        for (facet, value) in self.facets.iter_mut() {
            *value = self.holder
                .facet_truth(facet)
                .expect("Holder did not have a true value for a relevant facet!");
        }
    }
}

impl<K: KnowledgeTrait> Clone for ReflexiveModel<K> {
    fn clone(&self) -> Self {
        Self {
            holder: self.holder.clone(),
            facets: self.facets.clone(),
            //id: unique_u64(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.holder.clone_from(&source.holder);
        self.facets.clone_from(&source.facets);
    }
}

impl<K: KnowledgeTrait> Debug for ReflexiveModel<K> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("ReflexiveModel")
            .field("holder", &self.holder)
            .field("facets", &self.facets)
            .finish()
    }
}

pub struct EvidenceModel<K: KnowledgeTrait> {
    /// The entity that has the mental model.
    pub holder: K::Entity,
    /// The entity that this mental model is about.
    pub regarding: K::Entity,
    /// All of the facets of the regarded entity.
    pub facets: AHashMap<K::Facet, FacetData<K>>,
}

impl<K: KnowledgeTrait> EvidenceModel<K> {
    /// Create an evidence-based mental model for an entity regarding another entity.
    pub fn new(holder: K::Entity, regarding: K::Entity) -> Self {
        EvidenceModel {
            facets: regarding
                .relevant_facets()
                .into_iter()
                .map(|facet| {
                    let data = FacetData::new(&regarding, &facet);
                    (facet, data)
                })
                .collect(),
            holder,
            regarding,
        }
    }

    /// Get the facet data (or initialize it) for a particular facet.
    pub fn get_facet_data(&mut self, facet: K::Facet) -> &mut FacetData<K> {
        debug_assert!(self.regarding.is_facet_relevant(&facet));
        let regarding = &self.regarding;
        self.facets
            .entry(facet)
            .or_insert_with_key(|facet| FacetData::new(regarding, facet))
    }

    /// Get the data regarding a particular facet value, initializing it if necessary.
    pub fn get_value_data(&mut self, value: K::FacetValue) -> &mut FacetValueData<K> {
        self.get_facet_data(value.facet()).get_value_data(value)
    }

    pub fn recompute_total_strengths(&mut self) {
        for data in self.facets.values_mut() {
            data.recompute_total_strengths();
        }
    }

    pub fn recompute_strongest(&mut self) {
        for data in self.facets.values_mut() {
            data.recompute_strongest();
        }
    }

    /// Update all of the true facet values in this model.
    pub fn update_truths(&mut self) {
        for data in self.facets.values_mut() {
            data.update_truth(&self.regarding);
        }
    }

    pub fn get_strongest_belief(&self, facet: &K::Facet) -> Option<&K::FacetValue> {
        self.facets.get(facet)?.strongest.as_ref()
    }

    pub fn mutate<R: Rng>(&mut self, rng: &mut R) {
        let mut to_mutate = Vec::with_capacity(self.facets.len() * 2);

        for (facet, facet_data) in self.facets.iter() {
            for (value, value_data) in facet_data.values.iter() {
                for (i, evidence) in value_data.evidence.iter().enumerate() {
                    if let Some(new_value) = value.try_mutate(self, evidence, rng) {
                        debug_assert_eq!(*facet, new_value.facet());
                        to_mutate.push((value.clone(), new_value, i));
                    }
                }
            }
        }

        for (value, new_value, i) in to_mutate.into_iter().rev() {
            let evidence = self.get_value_data(value).remove_evidence(i).mutated();
            self.get_value_data(new_value).evidence.push(evidence);
        }
    }

    /// Insert a piece of [evidence](Evidence) into the model.
    ///
    /// Does **NOT** recompute total strengths nor the strongest beliefs.
    pub fn insert_evidence(&mut self, value: K::FacetValue, evidence: Evidence<K>) {
        self.get_value_data(value).evidence.push(evidence);
    }
}

impl<K: KnowledgeTrait> Clone for EvidenceModel<K> {
    fn clone(&self) -> Self {
        Self {
            holder: self.holder.clone(),
            regarding: self.regarding.clone(),
            facets: self.facets.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.holder.clone_from(&source.holder);
        self.regarding.clone_from(&source.regarding);
        self.facets.clone_from(&source.facets);
    }
}

impl<K: KnowledgeTrait> Debug for EvidenceModel<K> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("EvidenceModel")
            .field("holder", &self.holder)
            .field("regarding", &self.regarding)
            .field("facets", &self.facets)
            .finish()
    }
}

/// A mental model about an entity.
///
/// Can either be a [reflexive model](ReflexiveModel) (a mental model about oneself) or an [evidence
/// model](EvidenceModel) (a mental model about another entity using an evidence-based approach).
#[derive(From, TryInto)]
pub enum MentalModel<K: KnowledgeTrait> {
    Reflexive(ReflexiveModel<K>),
    Evidence(EvidenceModel<K>),
}

impl<K: KnowledgeTrait> MentalModel<K> {
    /// Create a new reflexive mental model.
    pub fn new_reflexive(holder: K::Entity) -> Self {
        ReflexiveModel::new(holder).into()
    }

    /// Create a new evidence-based mental model.
    pub fn new_evidence(holder: K::Entity, regarding: K::Entity) -> Self {
        EvidenceModel::new(holder, regarding).into()
    }

    /// Create a new mental model, with the kind depending on whether `holder` and `regarding` are
    /// equal.
    pub fn new(holder: K::Entity, regarding: K::Entity) -> Self
    where
        K::Entity: PartialEq,
    {
        if holder == regarding {
            Self::new_reflexive(holder)
        } else {
            Self::new_evidence(holder, regarding)
        }
    }

    /// Get the mental model holder.
    pub fn holder(&self) -> &K::Entity {
        match self {
            MentalModel::Reflexive(reflexive) => &reflexive.holder,
            MentalModel::Evidence(evidence) => &evidence.holder,
        }
    }

    /// Update all of the true facet values in this model.
    pub fn update_truths(&mut self) {
        match self {
            MentalModel::Reflexive(reflexive) => {
                reflexive.update_truths();
            },
            MentalModel::Evidence(evidence) => {
                evidence.update_truths();
            },
        }
    }

    pub fn recompute_total_strengths(&mut self) {
        if let MentalModel::Evidence(evidence) = self {
            evidence.recompute_total_strengths();
        }
    }

    pub fn recompute_strongest(&mut self) {
        if let MentalModel::Evidence(evidence) = self {
            evidence.recompute_strongest();
        }
    }

    pub fn get_strongest_belief(&self, facet: &K::Facet) -> Option<&K::FacetValue> {
        match self {
            MentalModel::Reflexive(reflexive) => reflexive.facets.get(facet),
            MentalModel::Evidence(evidence) => evidence.get_strongest_belief(facet),
        }
    }

    /// Insert a piece of [evidence](Evidence) into the model.
    ///
    /// Does **NOT** recompute total strengths nor the strongest beliefs.
    pub fn insert_evidence(&mut self, value: K::FacetValue, evidence: Evidence<K>) {
        if let MentalModel::Evidence(model) = self {
            model.insert_evidence(value, evidence);
        }
    }
}

impl<K: KnowledgeTrait> Clone for MentalModel<K> {
    fn clone(&self) -> Self {
        match self {
            Self::Reflexive(arg0) => Self::Reflexive(arg0.clone()),
            Self::Evidence(arg0) => Self::Evidence(arg0.clone()),
        }
    }
}

impl<K: KnowledgeTrait> Debug for MentalModel<K> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Reflexive(arg0) => f.debug_tuple("Reflexive").field(arg0).finish(),
            Self::Evidence(arg0) => f.debug_tuple("Evidence").field(arg0).finish(),
        }
    }
}
