use derive_more::{From, Display};
use super::*;
use character::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Display)]
pub enum TestFacet {
    #[display(fmt = "favorite music genre")]
    FavMusicGenre,
}

impl Entity<KnowledgeTypes> for Arc<Character> {
    fn relevant_facets(&self) -> Vec<TestFacet> {
        [
            TestFacet::FavMusicGenre,
        ].to_vec()
    }

    fn facet_truth(&self, facet: &TestFacet) -> Option<TestFacetValue> {
        Some(match facet {
            TestFacet::FavMusicGenre => self.fav_music.into(),
        })
    }
}

impl Facet<KnowledgeTypes> for TestFacet {
    fn initial_values(&self) -> Vec<TestFacetValue> {
        match self {
            TestFacet::FavMusicGenre => [
                FavMusicGenre::Jazz,
                FavMusicGenre::Rock,
                FavMusicGenre::Metal,
                FavMusicGenre::Calypso,
            ].into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, From, Display)]
pub enum TestFacetValue {
    #[display(fmt = "{_0}")]
    FavMusicGenre(FavMusicGenre),
}

impl FacetValue<KnowledgeTypes> for TestFacetValue {
    fn facet(&self) -> TestFacet {
        match self {
            TestFacetValue::FavMusicGenre(_) => TestFacet::FavMusicGenre,
        }
    }

    fn try_mutate<R: Rng>(
        &self,
        _model: &EvidenceModel<KnowledgeTypes>,
        _evidence: &Evidence<KnowledgeTypes>,
        _rng: &mut R,
    ) -> Option<Self> {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct KnowledgeTypes;

impl KnowledgeTrait for KnowledgeTypes {
    type Facet = TestFacet;

    type FacetValue = TestFacetValue;

    type Entity = Arc<Character>;

    type Data = ();
}