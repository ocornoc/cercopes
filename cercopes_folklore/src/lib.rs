use std::hash::Hash;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use ahash::AHashSet;
use rand::prelude::*;
use _private::*;

pub type Rng = rand_xoshiro::Xoroshiro64Star;

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

pub trait FolkloreTrait {
    type Character: Clone + Eq + Hash + MaybeSendSync;

    type FolkloreData: MaybeSendSync;

    type StoryData: MaybeSendSync;

    type MotifData: MaybeSendSync;
}

pub trait MotifBuilder<F: FolkloreTrait>: MaybeSendSync {
    fn begin_story(
        &self,
        manager: &FolkloreManager<F>,
        rng: &mut Rng,
        lore: &mut Folklore<F>,
        story: &mut Story<F>,
    ) -> Option<BuiltMotif<F>>;
}

pub struct BuiltMotif<F: FolkloreTrait> {
    pub characters: AHashSet<F::Character>,
    pub description: String,
    pub data: F::MotifData,
}

impl<F: FolkloreTrait> Clone for BuiltMotif<F>
where
    F::Character: Clone,
    F::MotifData: Clone,
{
    fn clone(&self) -> Self {
        BuiltMotif {
            characters: self.characters.clone(),
            description: self.description.clone(),
            data: self.data.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.characters.clone_from(&source.characters);
        self.description.clone_from(&source.description);
        self.data.clone_from(&source.data);
    }
}

impl<F: FolkloreTrait> Debug for BuiltMotif<F>
where
    F::Character: Debug,
    F::MotifData: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("BuiltMotif")
            .field("characters", &self.characters)
            .field("description", &self.description)
            .field("data", &self.data)
            .finish()
    }
}

pub struct Story<F: FolkloreTrait> {
    pub characters: AHashSet<F::Character>,
    pub plot: Vec<BuiltMotif<F>>,
    pub data: F::StoryData,
}

impl<F: FolkloreTrait> Clone for Story<F>
where
    F::Character: Clone,
    F::MotifData: Clone,
    F::StoryData: Clone,
{
    fn clone(&self) -> Self {
        Story {
            characters: self.characters.clone(),
            plot: self.plot.clone(),
            data: self.data.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.characters.clone_from(&source.characters);
        self.plot.clone_from(&source.plot);
        self.data.clone_from(&source.data);
    }
}

impl<F: FolkloreTrait> Debug for Story<F>
where
    F::Character: Debug,
    F::MotifData: Debug,
    F::StoryData: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("BuiltMotif")
            .field("characters", &self.characters)
            .field("plot", &self.plot)
            .field("data", &self.data)
            .finish()
    }
}

pub struct Folklore<F: FolkloreTrait> {
    pub characters: AHashSet<F::Character>,
    pub stories: Vec<Story<F>>,
    pub data: F::FolkloreData,
}

impl<F: FolkloreTrait> Clone for Folklore<F>
where
    F::Character: Clone,
    F::MotifData: Clone,
    F::StoryData: Clone,
    F::FolkloreData: Clone,
{
    fn clone(&self) -> Self {
        Folklore {
            characters: self.characters.clone(),
            stories: self.stories.clone(),
            data: self.data.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.characters.clone_from(&source.characters);
        self.stories.clone_from(&source.stories);
        self.data.clone_from(&source.data);
    }
}

impl<F: FolkloreTrait> Debug for Folklore<F>
where
    F::Character: Debug,
    F::MotifData: Debug,
    F::StoryData: Debug,
    F::FolkloreData: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f
            .debug_struct("BuiltMotif")
            .field("characters", &self.characters)
            .field("stories", &self.stories)
            .field("data", &self.data)
            .finish()
    }
}

pub struct FolkloreManager<F: FolkloreTrait> {
    pub motif_builders: Vec<Box<dyn MotifBuilder<F>>>,
}

impl<F: FolkloreTrait> FolkloreManager<F> {
    fn step_story_aux(&self, lore: &mut Folklore<F>, story: &mut Story<F>, rng: &mut Rng) -> bool {
        let mut builders = self.motif_builders.iter().collect::<Vec<_>>();
        builders.shuffle(rng);

        for builder in builders {
            if let Some(motif) = builder.begin_story(self, rng, lore, story) {
                story.plot.push(motif);
                return true;
            }
        }

        false
    }

    pub fn step_story(&self, lore: &mut Folklore<F>, story: &mut Story<F>) -> bool {
        self.step_story_aux(lore, story, &mut Rng::from_entropy())
    }

    pub fn finish_story(&self, lore: &mut Folklore<F>, story: &mut Story<F>) {
        let mut rng = Rng::from_entropy();
        while self.step_story_aux(lore, story, &mut rng) {}
        for motif in story.plot.iter() {
            story.characters.extend(motif.characters.iter().cloned());
        }
        lore.characters.extend(story.characters.iter().cloned());
    }
}
