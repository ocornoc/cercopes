use std::fmt::{Display, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use derive_more::Display;
use super::*;
use grammar::*;

static MASCULINE_NAMES: &[&str] = include!("masculine.txt");
static FEMININE_NAMES: &[&str] = include!("feminine.txt");
static LAST_NAMES: &[&str] = include!("last.txt");

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Display)]
pub enum FavMusicGenre {
    #[display(fmt = "jazz")]
    Jazz,
    #[display(fmt = "rock")]
    Rock,
    #[display(fmt = "metal")]
    Metal,
    #[display(fmt = "calypso")]
    Calypso,
}

impl Distribution<FavMusicGenre> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FavMusicGenre {
        [
            FavMusicGenre::Jazz,
            FavMusicGenre::Rock,
            FavMusicGenre::Metal,
            FavMusicGenre::Calypso,
        ].choose(rng).unwrap().clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum Gender {
    #[display(fmt = "masculine")]
    Masculine,
    #[display(fmt = "nonbinary")]
    Nonbinary,
    #[display(fmt = "feminine")]
    Feminine,
}

impl Distribution<Gender> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Gender {
        [
            Gender::Masculine,
            Gender::Nonbinary,
            Gender::Feminine,
        ].choose(rng).unwrap().clone()
    }
}

impl From<Gender> for SingularPronouns {
    fn from(g: Gender) -> Self {
        match g {
            Gender::Masculine => HeHimHis,
            Gender::Nonbinary => TheyThemTheirs,
            Gender::Feminine => SheHerHers,
        }
    }
}

impl Distribution<Name> for Gender {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Name {
        let first = *match self {
            Gender::Masculine => MASCULINE_NAMES.choose(rng),
            Gender::Nonbinary => if rng.gen() {
                MASCULINE_NAMES.choose(rng)
            } else {
                FEMININE_NAMES.choose(rng)
            },
            Gender::Feminine => FEMININE_NAMES.choose(rng),
        }.unwrap();
        let middle = if rng.gen() {
            LAST_NAMES.choose(rng).copied()
        } else {
            None
        };
        let mut last = *LAST_NAMES.choose(rng).unwrap();
        if let Some(middle) = middle {
            while last == middle {
                last = LAST_NAMES.choose(rng).unwrap();
            }
        }
        Name {
            first,
            middle,
            last,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Name {
    pub first: &'static str,
    pub middle: Option<&'static str>,
    pub last: &'static str,
}

impl Name {
    pub fn write_full_name(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(middle) = self.middle {
            write!(f, "{} {middle} {}", self.first, self.last)
        } else {
            write!(f, "{} {}", self.first, self.last)
        }
    }

    pub fn full_name(&self) -> String {
        if let Some(middle) = self.middle {
            format!("{} {middle} {}", self.first, self.last)
        } else {
            format!("{} {}", self.first, self.last)
        }
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.write_full_name(f)
    }
}

#[derive(Debug, Eq)]
pub struct Character {
    pub name: Name,
    pub age: u8,
    pub gender: Gender,
    pub pronouns: SingularPronouns,
    pub fav_music: FavMusicGenre,
    number: u32,
}

impl Distribution<Character> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Character {
        let gender = rng.gen::<Gender>();
        let pronouns = gender.into();
        let name = rng.sample(gender);
        Character {
            name,
            age: rng.gen_range(14..=100),
            pronouns,
            gender,
            fav_music: rng.gen(),
            number: get_unique_number(),
        }
    }
}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Hash for Character {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

impl Display for Character {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let Character {
            name,
            age,
            gender,
            pronouns,
            fav_music,
        .. } = self;
        writeln!(f, "{name}")?;
        writeln!(f, "Age: {age}")?;
        writeln!(f, "Gender: {gender}")?;
        writeln!(f, "Pronouns: {pronouns}")?;
        writeln!(f, "Favorite music: {fav_music}")?;
        Ok(())
    }
}

fn get_unique_number() -> u32 {
    use std::sync::atomic::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    COUNTER.fetch_add(1, Ordering::Relaxed)
}