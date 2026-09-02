//! The ASCII case-folding gate every name lookup passes through.
//!
//! Slugs in this crate are **canonically lowercase**: `as_str`, `Display`
//! and serde emit nothing else, and every `FromStr` folds its input before
//! looking it up, so `"BT709"`, `"Bt709"` and `"bt709"` are one value — the
//! same **named** variant. `Self::other` runs that exact lookup first too,
//! rather than a second table, so a canonical or documented-alias spelling
//! resolves to the named variant there as well, never a second value for a
//! meaning this crate already names.
//!
//! Only a genuine stranger — a spelling the lookup misses — reaches the
//! `Other(SmolStr)` escape, and it carries the caller's spelling
//! **verbatim**. Folding a name nobody claims would destroy information
//! (vendor fourccs and codec tags are routinely case-sensitive) for no
//! compensating benefit, now that the lookup above already catches every
//! case-variant of a name this crate *does* recognise.
//!
//! The lookup itself is on the **byte side**: [`fold`] yields the folded
//! bytes and every table arm is a `b"slug"` literal. A `FromStr` added
//! here writes its table that way.
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

/// ASCII-fold `s` into `buf`, returning the lowercase bytes, or [`None`]
/// when `s` is longer than any slug can be.
///
/// The lookup tables compare on the byte side — every `FromStr` in the
/// crate matches `b"slug"` literals — so the fold hands back the buffer
/// itself and never re-derives a `str` from it. That conversion would be
/// a second O(n) pass over bytes this function just wrote, to prove a
/// property no caller consumes.
///
/// Allocation-free, so the lookup gate is the same at every capability
/// tier.
pub(crate) fn fold<'b>(s: &str, buf: &'b mut [u8; FOLD_CAP]) -> Option<&'b [u8]> {
  let bytes = s.as_bytes();
  let n = bytes.len();
  if n > FOLD_CAP {
    return None;
  }
  buf[..n].copy_from_slice(bytes);
  buf[..n].make_ascii_lowercase();
  Some(&buf[..n])
}

#[cfg(test)]
mod tests;
