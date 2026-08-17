//! The ASCII case-folding gate every name lookup and every open escape
//! passes through.
//!
//! Slugs in this crate are **canonically lowercase**: `as_str`, `Display`
//! and serde emit nothing else, and every `FromStr` folds its input before
//! looking it up, so `"BT709"`, `"Bt709"` and `"bt709"` are one value. The
//! escape arms fold too ([`fold_owned`]), which is what keeps the whole
//! value space canonical and the derived `Eq` / `Hash` comparing *names*
//! rather than spellings.
//!
//! Folding is deliberately **ASCII-only**. These are FFmpeg / H.273 / file
//! extension identifiers; Unicode case folding is locale-sensitive
//! (Turkish dotless i maps `I` to `ı`, not `i`) in ways a wire vocabulary
//! must not be, and would make the canonical form depend on who is
//! reading.
//!
//! The errors these parses return live with the vocabularies themselves,
//! one per type — a `Rational` that is malformed and a `Matrix` that names
//! nothing are different failures, and the type is what says which.

/// Capacity of the stack buffer [`fold`] folds into.
///
/// The longest canonical slug in the crate is well under this; the buffer
/// exists because the coded vocabularies are available at the crate's
/// no-alloc tier, where there is no heap to fold into. An input that does
/// not fit cannot name a variant either, so the caller treats the
/// overflow as an ordinary miss.
pub(crate) const FOLD_CAP: usize = 64;

/// ASCII-fold `s` into `buf`, returning the lowercase view, or [`None`]
/// when `s` is longer than any slug can be.
///
/// Allocation-free, so the lookup gate is the same at every capability
/// tier.
pub(crate) fn fold<'b>(s: &str, buf: &'b mut [u8; FOLD_CAP]) -> Option<&'b str> {
  let bytes = s.as_bytes();
  let n = bytes.len();
  if n > FOLD_CAP {
    return None;
  }
  buf[..n].copy_from_slice(bytes);
  buf[..n].make_ascii_lowercase();
  // ASCII-lowercasing maps ASCII bytes to ASCII bytes and leaves every
  // other byte untouched, so UTF-8 validity is preserved; `ok()` keeps
  // this total without reaching for `unsafe`.
  core::str::from_utf8(&buf[..n]).ok()
}

/// ASCII-fold a slug that is about to be stored in an `Other(SmolStr)`
/// escape.
///
/// The one gate every escape is built through. Allocates only when the
/// input is not already folded.
#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) fn fold_owned(s: &str) -> smol_str::SmolStr {
  if s.bytes().any(|b| b.is_ascii_uppercase()) {
    smol_str::SmolStr::new(s.to_ascii_lowercase())
  } else {
    smol_str::SmolStr::new(s)
  }
}

#[cfg(test)]
mod tests;
