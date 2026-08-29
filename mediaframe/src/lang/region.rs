//! The [`Region`] primitive: the validated newtype, the error its door refuses with, and the
//! shared rows the family macro attaches to it.

use super::{
  registry,
  subtag::{self, Ascii},
};

#[cfg(test)]
mod tests;

/// A region subtag, in the registry's own spelling: `DE`, `TW`, `419`, `ZZ`.
///
/// The one type in this family with TWO grammars, because BCP 47 gives a region two: ISO 3166-1's
/// two-letter country codes, and UN M.49's three-digit area codes for the things that are not
/// countries — `419` is *Latin America and the Caribbean*, `001` is *World*. They share one
/// registry list, one column and one type, because they answer one question.
///
/// | sent | held | why |
/// |---|---|---|
/// | `DE`, `de`, `dE` | `DE` | ASCII case, upper being the registry's spelling |
/// | `419`, `001` | `419`, `001` | digits have no case, and the LEADING ZERO is part of the code |
/// | `BU` | `MM` | [`registry::region_preferred`] — Burma became Myanmar |
/// | `SU` | `SU` | deprecated, and the registry names no successor: the USSR became several |
///
/// # `ZZ` and `AA` are REGISTERED and PRIVATE at the same time, which no other subtag manages
///
/// The registry gives a region FOUR private-use spellings and only two of them are ranges: `QM`
/// through `QZ`, `XA` through `XZ`, and then `AA` and `ZZ` as records of their own, each carrying
/// the description *Private use*. A language and a script have one range each and nothing beside
/// it.
///
/// So on a region — and only on a region — [`is_registered`](Self::is_registered) and
/// [`is_private_use`](Self::is_private_use) can BOTH be true. That is the registry's shape rather
/// than a reading invented here, and it is why the private-use question is answered by a table the
/// generator builds from the registry's own `Description` column rather than by a range test alone.
///
/// [`ZZ`](Self::ZZ) names the one a container actually writes. In BCP 47 it is *Private use*; in
/// CLDR's reading of the same code it is *unknown or invalid region*, which is the sense a muxer
/// means by it. This type publishes the registry's word and not CLDR's, because the registry is
/// what is vendored.
///
/// # Deprecation, and the two answers the registry gives
///
/// Eleven region subtags are deprecated and six of them name a successor, which the door follows in
/// one hop. The other five — `AN`, `CS`, `NT`, `SU`, `YU` — name none, because each was a state
/// that dissolved into SEVERAL and there is no single region to fold onto. Those keep their own
/// spelling and answer `true` to [`is_deprecated`](Self::is_deprecated), which is the only honest
/// pair of answers available: inventing a successor would be picking one of Yugoslavia's six.
///
/// # The rest is [`Language`](super::Language)'s posture, unchanged
///
/// An unregistered subtag is admitted and named as such; a structural violation is refused;
/// equality is the canonical spelling, so `bu` and `MM` are one value and one stored row; ordering
/// is alphabetical and total. See that type, where each is argued.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::region")
)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Region(Ascii<AREA>);

impl Region {
  /// Read a region subtag, WIDE — either grammar, and any case.
  ///
  /// # The grammar, which is a DISJUNCTION
  ///
  /// Two ASCII letters, or three ASCII digits. BCP 47 spells it `region = 2ALPHA / 3DIGIT`, and the
  /// two are not interchangeable: `01` is not `001` and `DEU` is not a region at all.
  ///
  /// The refusal reads the CHARACTER CLASS first and the width second, which is what gives each
  /// arm its own sentence. A text of letters that is not two long is told the letter rule; one of
  /// digits that is not three long is told the digit rule; one that mixes the two is told that it
  /// is neither. Reading the width first would answer `D1` and `日本` with the same sentence, and
  /// neither client would learn anything from it.
  ///
  /// # The folds
  ///
  /// 1. **case**, on the letter arm only — upper, which is how the registry spells a country code.
  ///    A digit has no case, and the leading zero of `001` is part of the code rather than
  ///    formatting: it is NOT stripped, and a client sending `1` is refused rather than rounded.
  /// 2. **deprecated to preferred** — [`registry::region_preferred`]. `BU` answers `MM`. One hop is
  ///    enough by construction: the generator refuses a table in which a preferred value itself
  ///    prefers something else.
  ///
  /// # Errors
  ///
  /// [`ParseRegionError`], naming the rule that refused. Being unregistered is not one of them.
  pub fn new(text: &str) -> Result<Self, ParseRegionError> {
    if text.is_empty() {
      return Err(ParseRegionError::Empty);
    }

    let letters = text
      .chars()
      .all(|character| character.is_ascii_alphabetic());
    let digits = text.chars().all(|character| character.is_ascii_digit());

    let canonical = match (letters, digits) {
      (true, _) if text.len() == COUNTRY => Ascii::upper(text),
      (true, _) => return Err(ParseRegionError::WrongLetterWidth),
      // A digit has no case, so the fold is the identity — and the leading zero of `001` survives
      // it for the same reason it survives everything else here: it is part of the code.
      (_, true) if text.len() == AREA => Ascii::verbatim(text),
      (_, true) => return Err(ParseRegionError::WrongDigitWidth),
      _ => match subtag::non_alphanumeric(text) {
        Some(outside) => return Err(ParseRegionError::NotAlphanumeric(outside)),
        None => return Err(ParseRegionError::Mixed),
      },
    };

    match registry::region_preferred(canonical.as_str()) {
      Some(preferred) => Ok(Self(Ascii::verbatim(preferred))),
      None => Ok(Self(canonical)),
    }
  }

  /// The region subtag a muxer writes when it has none to give — `ZZ`.
  ///
  /// An associated constant — see [`Language::UND`](super::Language::UND), where the shape is
  /// argued.
  pub const ZZ: Self = Self(Ascii::literal("ZZ"));

  /// Is this `ZZ`?
  ///
  /// Named for the CODE rather than for a meaning, and deliberately: the registry calls it *Private
  /// use* and CLDR calls it *unknown or invalid region*, and a container writes it meaning the
  /// second. Naming the method `is_unknown` would put CLDR's reading on a type whose whole
  /// vocabulary is the registry's.
  #[must_use]
  pub fn is_zz(&self) -> bool {
    *self == Self::ZZ
  }

  /// The canonical subtag, as text.
  #[inline]
  #[must_use]
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }

  /// Is this an M.49 AREA code rather than an ISO 3166 country code?
  ///
  /// The one reading that tells the two grammars apart, and it is worth publishing because they
  /// mean different things: `DE` is a country and `150` is *Europe*, so a caller aggregating by
  /// country has to know which it is holding. Structural rather than a lookup — three digits is the
  /// whole of what makes a subtag an area code.
  #[must_use]
  pub fn is_area(&self) -> bool {
    self.as_str().len() == AREA
  }

  /// The registry's own name for this region — `Germany`, `Latin America and the Caribbean` — or
  /// [`None`] where it registers no such subtag.
  #[inline]
  #[must_use]
  pub fn name(&self) -> Option<&'static str> {
    registry::region_name(self.as_str())
  }

  /// Does the vendored registry carry this subtag?
  ///
  /// Unlike the other two types in this family, this can be `true` at the same time as
  /// [`is_private_use`](Self::is_private_use) — see the type docs, where the registry's four
  /// private-use spellings are set out.
  #[inline]
  #[must_use]
  pub fn is_registered(&self) -> bool {
    self.name().is_some()
  }

  /// Has the registry deprecated this subtag?
  ///
  /// A canonical value can still be deprecated: five of the eleven name no successor, so the fold
  /// has nowhere to send them. `BU` is never one of these — it prefers `MM`, so a `Region` never
  /// holds it.
  #[inline]
  #[must_use]
  pub fn is_deprecated(&self) -> bool {
    registry::region_is_deprecated(self.as_str())
  }

  /// Is this subtag one the registry reserves for private use?
  ///
  /// FOUR spellings and only two of them ranges — `QM`..`QZ`, `XA`..`XZ`, and the individually
  /// registered `AA` and `ZZ`. A range test alone would answer `false` to the two a container
  /// actually writes.
  #[inline]
  #[must_use]
  pub fn is_private_use(&self) -> bool {
    registry::region_is_private_use(self.as_str())
  }
}

/// An ISO 3166-1 country code is exactly this wide.
const COUNTRY: usize = 2;

/// A UN M.49 area code is exactly this wide.
const AREA: usize = 3;

/// A string does not name a region subtag.
///
/// Five variants, which is more than either sibling has, and the extra two are the DISJUNCTION's
/// doing: two grammars means two width rules to be told apart, and a text that is alphanumeric
/// without belonging to either.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ParseRegionError {
  /// Nothing was sent — an ABSENT region rather than a malformed one, whose home is an
  /// `Option::None`.
  #[error("a region subtag is two letters or three digits, and nothing was sent")]
  Empty,
  /// Letters, but not exactly two. `DEU` is refused: a three-letter code is a LANGUAGE's shape, and
  /// admitting it here would take a tag apart wrongly rather than say so.
  #[error("a region subtag written in letters is exactly two")]
  WrongLetterWidth,
  /// Digits, but not exactly three. `1` is refused rather than read as `001` — the leading zeros
  /// are part of the M.49 code, not formatting.
  #[error("a region subtag written in digits is exactly three")]
  WrongDigitWidth,
  /// A character that is neither an ASCII letter nor an ASCII digit, carrying the first one found.
  #[error("a region subtag is letters or digits, so `{0}` is not one of its characters")]
  NotAlphanumeric(char),
  /// Letters and digits together, which is neither grammar — `D1`, `4A`.
  #[error("a region subtag is two letters or three digits, and never a mixture")]
  Mixed,
}

super::subtag_common!(Region, ParseRegionError);
