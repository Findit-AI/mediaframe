//! The [`Language`] primitive: the validated newtype, the error its door refuses with, and the
//! shared rows the family macro attaches to it.

use super::{
  registry,
  subtag::{self, Ascii, MAX},
};

#[cfg(test)]
mod tests;

/// A primary language subtag, in the shortest spelling BCP 47 has for it: `de`, `zh`, `yue`, `und`.
///
/// WIDE IN, STRICT OUT. What a container writes is dirty in four separate ways and each has a fold,
/// none of which is a rule invented here — every one is a column of a vendored registry:
///
/// | sent | held | the column that did it |
/// |---|---|---|
/// | `DE`, `De` | `de` | none — ASCII case is a spelling, not a value |
/// | `ger` (mkv, ISO 639-2/B) | `de` | [`registry::alpha3`] |
/// | `deu` (mp4, ISO 639-2/T) | `de` | the same, in ONE hop rather than through `ger` |
/// | `iw` | `he` | [`registry::language_preferred`] |
///
/// The two folds are applied in that order and each at most once, so `GER` reaches `de` and stops.
/// See [`new`](Self::new).
///
/// # `und` is a VALUE, and it is not the absence of one
///
/// The registry registers `und` — *Undetermined* — like any other subtag, and this type holds it
/// like any other value. A track whose language nobody recorded has NO language, which is an
/// `Option::None`; a track a muxer explicitly tagged `und` has one, and it says the
/// muxer looked and could not tell. Collapsing the two would lose the difference between *nobody
/// asked* and *somebody asked and did not know*, and the second is the one that says a detector has
/// work to do.
///
/// [`UND`](Self::UND) is how the value is named — an associated constant, which the seat being an
/// inline byte buffer makes trivially possible. It allocates nothing and it never could: this type
/// has no pointer in it.
///
/// # A subtag the registry does not carry is ADMITTED
///
/// Structure and registration are two questions and this type only refuses on the first. `xyz` is
/// three ASCII letters, which is a language subtag's shape, so it is held — and
/// [`is_registered`](Self::is_registered) is what says the registry has never heard of it.
///
/// Refusing it would be the worse bargain by some distance. The registry gains subtags, a vendored
/// copy is a snapshot, and a container in the wild carries whatever its muxer believed: refusing
/// would make a file unindexable because of a table this crate last fetched in August, where
/// admitting it stores the tag, answers `false` to the one question that is actually about the
/// registry, and lets everything else proceed.
///
/// The PRIVATE-USE range is the same posture with a name: `qaa` through `qtz` are reserved by the
/// registry as a block and individually registered by nobody, so
/// [`is_private_use`](Self::is_private_use) is what tells a deliberate private tag apart from a
/// typo. Both answer `is_registered() == false`, and only the first is intentional.
///
/// # Equality is the CANONICAL subtag, so the folds are what make it useful
///
/// `Language::new("ger")? == Language::new("de")?`, because both hold `de`. That is the whole
/// point of folding at the door rather than at comparison time: an mkv's German and an mp4's German
/// are one value, one map key and one hash bucket — and nothing downstream has to know that
/// ISO 639-2 has two alphabets.
///
/// It also means a value that has been stored is canonical, which is what lets a row written from
/// an mkv and a query built from `de` meet without either side knowing the other's spelling.
///
/// # Ordering is ALPHABETICAL, and that is all it is
///
/// [`Ord`] is derived through the text, so `de < en < fr`. It orders the SPELLINGS and says nothing
/// about the languages — `zh` sorts last of those four and is spoken by the most people — which is
/// the same bargain any single-word vocabulary states for its sort order, and it earns its place
/// for the same reason: total and stable, and therefore a usable key wherever values have to be
/// walked in a fixed order.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::language_subtag")
)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language(Ascii<MAX>);

impl Language {
  /// Read a language subtag, WIDE — any case, and either ISO 639-2 alphabet.
  ///
  /// The one door in. Every other route to a value of this type — [`FromStr`](core::str::FromStr),
  /// the serde read, the wire decode — goes through here, so there is one grammar and one fold
  /// rather than one per entry point.
  ///
  /// # The grammar, and it is checked BEFORE the registry
  ///
  /// Two to eight ASCII letters, which is BCP 47's `language = 2*3ALPHA / 4ALPHA / 5*8ALPHA` read
  /// as one range. Anything else is refused, and the refusal names which rule broke rather than
  /// reporting that the input was malformed — see [`ParseLanguageError`].
  ///
  /// The ALPHABET is tested before the LENGTH, which is not the order the rules are written in and
  /// is the order that gives the better sentence: `日本語` is 9 bytes, so a length-first check would
  /// refuse it for being too long when what is wrong with it is that a subtag is letters.
  ///
  /// # The folds, in order, each at most once
  ///
  /// 1. **case** — lower, which is the case the registry spells a language in.
  /// 2. **ISO 639-2 to the shortest spelling** — [`registry::alpha3`]. `ger` and `deu` both answer
  ///    `de`, directly rather than through each other, so this is one hop and not two.
  /// 3. **deprecated to preferred** — [`registry::language_preferred`]. `iw` answers `he`. One hop
  ///    is enough by construction: the generator refuses a table in which a preferred value itself
  ///    prefers something else.
  ///
  /// The second and third are applied in sequence rather than as alternatives, so a code that
  /// needed both would take both. None does today — every target of the alpha-3 fold is a current
  /// two-letter code — and the sequence is what keeps that a fact about the data rather than a
  /// premise of the code.
  ///
  /// # Errors
  ///
  /// [`ParseLanguageError`], naming the rule that refused. Being unregistered is not one of them.
  pub fn new(text: &str) -> Result<Self, ParseLanguageError> {
    if text.is_empty() {
      return Err(ParseLanguageError::Empty);
    }

    if let Some(outside) = subtag::non_alphabetic(text) {
      return Err(ParseLanguageError::NotAlphabetic(outside));
    }

    match text.len() {
      0 | 1 => return Err(ParseLanguageError::TooShort),
      2..=MAX => {}
      _ => return Err(ParseLanguageError::TooLong),
    }

    let lower = Ascii::<MAX>::lower(text);
    let shortest = registry::alpha3(lower.as_str()).unwrap_or_else(|| lower.as_str());
    let preferred = registry::language_preferred(shortest).unwrap_or(shortest);

    // The registry hands back a `&'static str` from the generated table, so the fold's result is
    // re-seated rather than returned: `lower` is the arriving spelling and `preferred` may be a
    // different subtag entirely (`ger` reaches `de`). Where no fold fired the two are the same
    // bytes and this is a copy of a copy, which at eight bytes is not worth a branch to avoid.
    Ok(Self(Ascii::verbatim(preferred)))
  }

  /// The undetermined language, `und` — the value a muxer writes when it looked and could not tell.
  pub const UND: Self = Self(Ascii::literal("und"));

  /// Is this the undetermined language?
  ///
  /// The question a `match` arm would have asked if a value of this type could be a pattern. See
  /// the type docs, where what stands in the way is stated.
  #[must_use]
  pub fn is_undetermined(&self) -> bool {
    *self == Self::UND
  }

  /// The canonical subtag, as text.
  #[inline]
  #[must_use]
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }

  /// The registry's own name for this language — `German`, `Yue Chinese` — or [`None`] where it
  /// registers no such subtag.
  ///
  /// The FIRST description where the registry gives several: `ro` is *Romanian*, which the registry
  /// leads with, and not *Moldavian* or *Moldovan*, which it also lists. A name is a label for a
  /// person to read; the identity is the subtag.
  #[inline]
  #[must_use]
  pub fn name(&self) -> Option<&'static str> {
    registry::language_name(self.as_str())
  }

  /// Does the vendored registry carry this subtag?
  ///
  /// `false` is not an error and never was — see the type docs. It is `false` for a private-use
  /// subtag too, which [`is_private_use`](Self::is_private_use) is what distinguishes.
  #[inline]
  #[must_use]
  pub fn is_registered(&self) -> bool {
    self.name().is_some()
  }

  /// Has the registry deprecated this subtag?
  ///
  /// A canonical value can still be deprecated, which is not a contradiction: 120 deprecated
  /// language subtags name no replacement, so the fold has nowhere to send them and they stay
  /// themselves. `iw` is never one of these — it prefers `he`, so a `Language` never holds it.
  #[inline]
  #[must_use]
  pub fn is_deprecated(&self) -> bool {
    registry::language_is_deprecated(self.as_str())
  }

  /// Does this subtag fall in the range the registry reserves for private use, `qaa` to `qtz`?
  ///
  /// What tells a deliberate private tag apart from a typo, both of which are unregistered.
  #[inline]
  #[must_use]
  pub fn is_private_use(&self) -> bool {
    registry::language_is_private_use(self.as_str())
  }

  /// The script this language implies, where the registry says it implies one.
  ///
  /// `Suppress-Script`, and the reason it is published on the LANGUAGE rather than kept private to
  /// the composition that uses it: it is a fact about the language — English is written in Latin
  /// script — and a caller who wants to know why `en-Latn` composed as `en` reads it here.
  ///
  /// [`None`] for `zh`, which is what keeps `zh-Hans` and `zh-Hant` two different identities.
  #[inline]
  #[must_use]
  pub fn suppressed_script(&self) -> Option<&'static str> {
    registry::language_suppress_script(self.as_str())
  }
}

/// A string does not name a language subtag.
///
/// One variant per way the GRAMMAR can fail, and no variant for the registry — an unregistered
/// subtag is admitted, which the type docs argue. The text arrives from a container or from a
/// caller, and which rule broke is the only part of the answer either can act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ParseLanguageError {
  /// Nothing was sent. Distinct from [`TooShort`](Self::TooShort) because it is the case a
  /// container actually produces: a stream with no language tag at all, which is an ABSENT language
  /// rather than a malformed one, and whose home is an `Option::None`.
  #[error("a language subtag is two to eight letters, and nothing was sent")]
  Empty,
  /// One letter. There is no one-letter language subtag — a single letter opens an EXTENSION or the
  /// private-use sequence `x-`, neither of which is a language.
  #[error("a language subtag is at least two letters")]
  TooShort,
  /// More than eight letters, which is the widest subtag the grammar admits.
  #[error("a language subtag is at most eight letters")]
  TooLong,
  /// A character that is not an ASCII letter, carrying the first one found — the `_` of `zh_CN`, a
  /// digit, or anything outside ASCII.
  #[error("a language subtag is letters, so `{0}` is not one of its characters")]
  NotAlphabetic(char),
}

super::subtag_common!(Language, ParseLanguageError);
