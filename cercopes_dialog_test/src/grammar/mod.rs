use derive_more::Display;
pub use SingularPronouns::*;

/// Gendered pronouns
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display)]
pub enum SingularPronouns {
    #[display(fmt = "I (1st p)")]
    IMeMine,
    #[display(fmt = "we (2nd p)")]
    WeUsOurs,
    #[display(fmt = "he/him")]
    HeHimHis,
    #[display(fmt = "they/them")]
    TheyThemTheirs,
    #[display(fmt = "she/her")]
    SheHerHers,
    #[display(fmt = "it/it")]
    ItItIts,
    #[display(fmt = "plural")]
    Plural,
    #[display(fmt = "you (2nd p)")]
    SecondPerson,
    #[display(fmt = "who (interrogative)")]
    InterrogativePersonal,
    #[display(fmt = "what (interrogative)")]
    InterrogativeNonpersonal,
}

impl SingularPronouns {
    /// The nominative case for the pronoun:
    ///
    /// "he" or "they" or "she"
    pub const fn nominative(self) -> &'static str {
        match self {
            IMeMine => "I",
            WeUsOurs => "we",
            HeHimHis => "he",
            TheyThemTheirs | Plural => "they",
            SheHerHers => "she",
            ItItIts => "it",
            SecondPerson => "you",
            InterrogativePersonal => "who",
            InterrogativeNonpersonal => "what",
        }
    }

    /// The accusative case for the pronoun:
    ///
    /// "him" or "them" or "her"
    pub const fn accusative(self) -> &'static str {
        match self {
            IMeMine => "me",
            WeUsOurs => "we",
            HeHimHis => "him",
            TheyThemTheirs | Plural => "them",
            SheHerHers => "her",
            ItItIts => "it",
            SecondPerson => "you",
            InterrogativePersonal => "whom",
            InterrogativeNonpersonal => "what",
        }
    }

    /// Reflexive for a pronoun:
    ///
    /// "himself" or "herself" or "themself"
    pub const fn reflexive(self) -> &'static str {
        match self {
            IMeMine => "mine",
            WeUsOurs => "we",
            HeHimHis => "his",
            TheyThemTheirs | InterrogativePersonal => "themself",
            SheHerHers => "her",
            ItItIts | InterrogativeNonpersonal => "itself",
            Plural => "themselves",
            SecondPerson => "yourself",
        }
    }

    /// Dependent genitive for a pronoun:
    ///
    /// "his" or "their" or "her"
    pub const fn dep_genitive(self) -> &'static str {
        match self {
            IMeMine => "my",
            WeUsOurs => "our",
            HeHimHis => "his",
            TheyThemTheirs | Plural => "their",
            SheHerHers => "her",
            ItItIts | InterrogativeNonpersonal => "its",
            SecondPerson => "your",
            InterrogativePersonal => "whose",
        }
    }

    /// Independent genetive for a pronoun:
    ///
    /// "his" or "theirs" or "hers"
    pub const fn indep_genitive(self) -> &'static str {
        match self {
            IMeMine => "mine",
            WeUsOurs => "ours",
            HeHimHis => "his",
            TheyThemTheirs | Plural => "theirs",
            SheHerHers => "hers",
            ItItIts | InterrogativeNonpersonal => "its",
            SecondPerson => "yours",
            InterrogativePersonal => "whose",
        }
    }
}