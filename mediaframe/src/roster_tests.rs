//! The shared runtime half of the `ROSTER` contract.
//!
//! Completeness is proved at compile time: the `roster!` macro builds the
//! constant and an exhaustive `match` from one list, so a variant missing
//! from the roster is `E0004` with the compiler naming it. Two properties
//! survive that proof and need running code:
//!
//! - a **duplicate** entry. A list that names one variant twice and still
//!   names every other variant once leaves the witness exhaustive, so the
//!   compiler has nothing to say about it.
//! - a **slug collision**. Two variants rendering the same string keep the
//!   roster well-formed while making `FromStr` unable to return both, so
//!   the parse round-trip stops being the inverse of the render.
//!
//! [`check`] takes the type's `as_str` as a function rather than going
//! through [`Display`](core::fmt::Display), so the checks run at the
//! no-alloc tier too — `to_string` would need an allocator the lean build
//! does not have.

use core::{fmt::Debug, str::FromStr};

/// Asserts the two runtime properties of a `ROSTER`, for one type.
///
/// `what` names the type in the failure message; `as_str` is the type's
/// canonical renderer.
pub(crate) fn check<T, F>(roster: &[T], what: &str, as_str: F)
where
  T: FromStr + PartialEq + Debug,
  <T as FromStr>::Err: Debug,
  F: Fn(&T) -> &str,
{
  assert!(!roster.is_empty(), "{what}: ROSTER is empty");

  for (i, entry) in roster.iter().enumerate() {
    let slug = as_str(entry);

    // No duplicate entries, and no two entries sharing a slug. The second
    // is the stronger statement and catches the first for every type whose
    // renderer is injective, but both are asserted: a type could in
    // principle repeat a variant whose slug is also repeated, and the
    // separate messages say which fault it is.
    for (j, prior) in roster.iter().enumerate().take(i) {
      assert!(
        prior != entry,
        "{what}: ROSTER names entry {j} ({slug}) again at {i}"
      );
      assert!(
        as_str(prior) != slug,
        "{what}: ROSTER entries {j} and {i} both render `{slug}`, \
         so `FromStr` cannot be the inverse of `as_str` for both"
      );
    }

    // The slug round-trips: rendering then parsing is the identity on
    // every named variant.
    let parsed = slug
      .parse::<T>()
      .unwrap_or_else(|e| panic!("{what}: ROSTER slug `{slug}` does not parse: {e:?}"));
    assert!(
      &parsed == entry,
      "{what}: `{slug}` parses to {parsed:?}, not the roster entry {entry:?}"
    );
  }
}
