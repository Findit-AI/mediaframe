//! The [`ScriptSubtag`] primitive: the validated newtype, the error its door refuses with, and the
//! shared rows the family macro attaches to it.

use super::{
  registry,
  subtag::{self, Ascii},
};

#[cfg(test)]
mod tests;

/// An ISO 15924 script subtag, in the registry's own Titlecase: `Latn`, `Hans`, `Hant`, `Zxxx`.
///
/// The same family shape as [`Language`](super::Language) with a narrower grammar and ONE fewer
/// fold. Four ASCII letters exactly, Titlecased, and that is the whole canonicalisation — the
/// registry publishes no `Preferred-Value` on a single script record, so there is no deprecation to
/// follow and nothing here to chain.
///
/// | sent | held |
/// |---|---|
/// | `Latn`, `latn`, `LATN`, `lAtN` | `Latn` |
/// | `Hans` | `Hans` |
/// | `Hant` | `Hant` — and NOT `Hans` |
///
/// # SIMPLIFIED AND TRADITIONAL ARE TWO SCRIPTS HERE, and that is a ruling rather than an oversight
///
/// `Hans` and `Hant` are different values, they compare unequal, they store as different bytes and
/// nothing here relates them. A caller looking for a fold that makes `zh-Hant` answer a `zh-Hans`
/// question will not find one, and should not: **this is the metadata layer, and its job is to
/// carry what the file DECLARED.**
///
/// The two are genuinely different writing systems for one language, so which one a track is in is
/// a fact about the track — and a type that folded them would make the two indistinguishable in
/// every direction, including the one somebody wants. What a SEARCH over them should do is a
/// question for the retrieval layer, which has a query, a corpus and a user's intent to weigh; a
/// primitive has none of the three and would be guessing on all of them.
///
/// The composition rule is what keeps the pair reachable at all: `zh` publishes no
/// `Suppress-Script`, so `zh-Hans` composes as itself rather than collapsing to `zh` the way
/// `en-Latn` collapses to `en`. See [`LanguageId`](super::LanguageId).
///
/// # `Zxxx` and `Zzzz` are VALUES, for `und`'s reason
///
/// The registry registers both. `Zxxx` is *Code for unwritten documents* — a track with speech and
/// no writing — and `Zzzz` is *Code for uncoded script*, which is a script somebody looked at and
/// could not place. Neither is the absence of a script: that is an `Option::None`, and it says
/// nobody declared one.
///
/// [`ZXXX`](Self::ZXXX) and [`ZZZZ`](Self::ZZZZ) name them, as associated constants — the shape
/// [`Language::UND`](super::Language::UND) takes, and for the same reason.
///
/// # An unregistered subtag is ADMITTED, and the private-use range is named
///
/// [`Language`](super::Language)'s posture exactly: four letters is a script subtag's shape, so
/// `Abcd` is held and [`is_registered`](Self::is_registered) is `false`. `Qaaa` through `Qabx` are
/// the registry's reserved block, and [`is_private_use`](Self::is_private_use) is what tells a
/// deliberate private script apart from a typo — both being unregistered, and only one intentional.
///
/// # No deprecation reading, because the registry publishes none
///
/// [`Language`](super::Language) and [`Region`](super::Region) each carry `is_deprecated`, and this
/// type does not. That asymmetry is the registry's rather than this house's: not one of the 224
/// script records carries a `Deprecated` date. A predicate here would be a method that has answered
/// `false` since ISO 15924 began and would go on doing so until it silently stopped being complete.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::script_subtag")
)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScriptSubtag(Ascii<WIDTH>);

impl ScriptSubtag {
  /// Read a script subtag, WIDE — any case.
  ///
  /// The one door in, as [`Language::new`](super::Language::new) is for its type.
  ///
  /// # The grammar
  ///
  /// EXACTLY four ASCII letters, which is ISO 15924's whole shape — `script = 4ALPHA` in BCP 47's
  /// own grammar, with no shorter and no longer form. The ALPHABET is tested before the LENGTH, for
  /// [`Language::new`](super::Language::new)'s reason: a non-ASCII input is refused for its
  /// characters rather than for the width of its encoding.
  ///
  /// # The fold, and there is one
  ///
  /// Titlecase — one capital, then lower — which is how ISO 15924 and the registry both spell a
  /// script. Nothing else: a script subtag has no alias table and no preferred value to follow.
  ///
  /// # Errors
  ///
  /// [`ParseScriptSubtagError`], naming the rule that refused. Being unregistered is not one of
  /// them.
  pub fn new(text: &str) -> Result<Self, ParseScriptSubtagError> {
    if text.is_empty() {
      return Err(ParseScriptSubtagError::Empty);
    }

    if let Some(outside) = subtag::non_alphabetic(text) {
      return Err(ParseScriptSubtagError::NotAlphabetic(outside));
    }

    if text.len() != WIDTH {
      return Err(ParseScriptSubtagError::WrongWidth);
    }

    Ok(Self(Ascii::title(text)))
  }

  /// The script subtag for a document with no writing in it — `Zxxx`, *Code for unwritten
  /// documents*.
  ///
  /// An associated constant — see [`Language::UND`](super::Language::UND), where the shape is
  /// argued.
  pub const ZXXX: Self = Self(Ascii::literal("Zxxx"));

  /// The script subtag for writing nobody could place — `Zzzz`, *Code for uncoded script*.
  pub const ZZZZ: Self = Self(Ascii::literal("Zzzz"));

  /// Is this the unwritten-document script, `Zxxx`?
  #[must_use]
  pub fn is_unwritten(&self) -> bool {
    *self == Self::ZXXX
  }

  /// Is this the uncoded script, `Zzzz`?
  #[must_use]
  pub fn is_uncoded(&self) -> bool {
    *self == Self::ZZZZ
  }

  /// The canonical subtag, as text.
  #[inline]
  #[must_use]
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }

  /// The registry's own name for this script — `Latin`, `Han (Simplified variant)` — or [`None`]
  /// where it registers no such subtag.
  #[inline]
  #[must_use]
  pub fn name(&self) -> Option<&'static str> {
    registry::script_name(self.as_str())
  }

  /// Does the vendored registry carry this subtag?
  #[inline]
  #[must_use]
  pub fn is_registered(&self) -> bool {
    self.name().is_some()
  }

  /// Does this subtag fall in the range the registry reserves for private use, `Qaaa` to `Qabx`?
  #[inline]
  #[must_use]
  pub fn is_private_use(&self) -> bool {
    registry::script_is_private_use(self.as_str())
  }
}

/// Every script subtag is exactly this wide. ISO 15924's own shape, and BCP 47's `script = 4ALPHA`.
const WIDTH: usize = 4;

/// A string does not name a script subtag.
///
/// Three variants where [`ParseLanguageError`](super::ParseLanguageError) has four, and the missing
/// one is the grammar's doing: a script has ONE width, so there is no *too short* to tell apart
/// from *too long*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ParseScriptSubtagError {
  /// Nothing was sent — an ABSENT script rather than a malformed one, whose home is an
  /// `Option::None`. Distinct from [`WrongWidth`](Self::WrongWidth) because it is the case a
  /// container actually produces.
  #[error("a script subtag is four letters, and nothing was sent")]
  Empty,
  /// Not exactly four letters. `Lat` and `Latin` are refused alike: ISO 15924 has one width and
  /// neither a shorter nor a longer form of a code.
  #[error("a script subtag is exactly four letters")]
  WrongWidth,
  /// A character that is not an ASCII letter, carrying the first one found.
  #[error("a script subtag is letters, so `{0}` is not one of its characters")]
  NotAlphabetic(char),
}

super::subtag_common!(ScriptSubtag, ParseScriptSubtagError);
