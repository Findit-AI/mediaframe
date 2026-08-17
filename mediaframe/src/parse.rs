//! The error returned by the [`FromStr`](core::str::FromStr) impls of
//! mediaframe's **closed** vocabulary and geometry types.
//!
//! The *open* enums — the ones carrying an `Other(SmolStr)` escape arm
//! ([`crate::codec::VideoCodec`], [`crate::container::Format`], …) — parse
//! infallibly and use `Infallible` as their error; they never reach this
//! type. Everything else has a finite set of legal spellings, so an
//! unrecognised one has to be rejected.
//!
//! # Why one shared type rather than one error per vocabulary
//!
//! Every one of these parses fails for the same three reasons, none of
//! which the caller can recover from differently, so a per-type taxonomy
//! would be ~18 structurally identical types with no added caller
//! decision. The rejected input is deliberately **not** retained: these
//! types are available at the crate's no-alloc tier, where there is
//! nowhere to put an owned copy, and the input is attacker-controlled on
//! the deserialization path.

/// Error returned when a string is not a valid spelling of a mediaframe
/// vocabulary or geometry value.
///
/// Carries the name of the type that rejected the input — enough to tell
/// two failures apart in a log line — but not the input itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid {ty}: {kind}")]
pub struct ParseError {
  ty: &'static str,
  kind: Kind,
}

impl ParseError {
  /// The name of the type that rejected the input (`"PixelFormat"`, …).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn type_name(&self) -> &'static str {
    self.ty
  }

  /// The input is not one of this vocabulary's canonical slugs.
  pub(crate) const fn unrecognised(ty: &'static str) -> Self {
    Self {
      ty,
      kind: Kind::Unrecognised,
    }
  }

  /// The input does not have this type's textual shape at all (missing
  /// separator, non-numeric component, trailing text).
  pub(crate) const fn malformed(ty: &'static str) -> Self {
    Self {
      ty,
      kind: Kind::Malformed,
    }
  }

  /// The input parsed structurally but violates the type's invariant.
  pub(crate) const fn out_of_range(ty: &'static str) -> Self {
    Self {
      ty,
      kind: Kind::OutOfRange,
    }
  }
}

/// ASCII-fold a slug to the crate's lowercase canon.
///
/// The one gate every `Other(SmolStr)` escape is built through, so the whole
/// value space stays lowercase-canonical and the derived `Eq` / `Hash` on
/// those enums compare *names*, not spellings. Deliberately ASCII-only:
/// these are FFmpeg/H.273 identifiers, and Unicode case folding is
/// locale-sensitive in ways a wire vocabulary must not be.
///
/// Allocates only when the input is not already folded.
#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) fn fold_owned(s: &str) -> smol_str::SmolStr {
  if s.bytes().any(|b| b.is_ascii_uppercase()) {
    smol_str::SmolStr::new(s.to_ascii_lowercase())
  } else {
    smol_str::SmolStr::new(s)
  }
}

/// Kept private: the three reasons are a diagnostic detail, not a
/// classification callers branch on. Promoting it later is additive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
  Unrecognised,
  Malformed,
  OutOfRange,
}

impl core::fmt::Display for Kind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      Self::Unrecognised => "unrecognised value",
      Self::Malformed => "malformed",
      Self::OutOfRange => "value out of range",
    })
  }
}

#[cfg(test)]
mod tests {
  use super::ParseError;

  #[test]
  fn type_name_survives_and_kinds_stay_distinct() {
    let unrecognised = ParseError::unrecognised("PixelFormat");
    let malformed = ParseError::malformed("Rational");
    let out_of_range = ParseError::out_of_range("Rational");

    assert_eq!(unrecognised.type_name(), "PixelFormat");
    assert_eq!(malformed.type_name(), "Rational");
    assert_ne!(
      malformed, out_of_range,
      "a malformed input and a rejected invariant must not compare equal"
    );
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[test]
  fn display_names_the_rejecting_type_and_never_the_input() {
    use std::string::ToString;

    assert_eq!(
      ParseError::unrecognised("PixelFormat").to_string(),
      "invalid PixelFormat: unrecognised value"
    );
    assert_eq!(
      ParseError::malformed("Dimensions").to_string(),
      "invalid Dimensions: malformed"
    );
    assert_eq!(
      ParseError::out_of_range("Rational").to_string(),
      "invalid Rational: value out of range"
    );
  }
}
