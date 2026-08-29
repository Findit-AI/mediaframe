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

/// One vocabulary's extension entries, as gathered by
/// [`extension_entries_of`]: `(extension, the owning variant's Debug)`,
/// one pair per `(variant, alias)` combination.
#[cfg(any(feature = "std", feature = "alloc"))]
type ExtensionEntries = std::vec::Vec<(&'static str, std::string::String)>;

/// Gathers every `(extension, variant)` pair a roster's own
/// [`Format::extensions`](crate::container::Format::extensions)-shaped
/// method emits, for every named variant. Generic over the accessor
/// rather than the type, so the three call sites in
/// [`container_and_audio_and_image_extension_entries`] are one line each
/// with no hand-transcribed extension lists to drift from the real
/// `extensions()` tables.
#[cfg(any(feature = "std", feature = "alloc"))]
fn extension_entries_of<T: Debug>(
  roster: &'static [T],
  extensions: impl Fn(&'static T) -> &'static [&'static str],
) -> ExtensionEntries {
  roster
    .iter()
    .flat_map(|v| {
      extensions(v)
        .iter()
        .map(move |ext| (*ext, std::format!("{v:?}")))
    })
    .collect()
}

/// The three extension-bearing vocabularies' entries, gathered
/// programmatically from their own `ROSTER` + `extensions()` — no hand
/// list of extensions here to drift from the real tables in
/// `container::Format`, `audio::ContainerFormat`, and `image::Format`
/// themselves.
#[cfg(any(feature = "std", feature = "alloc"))]
fn container_and_audio_and_image_extension_entries()
-> std::vec::Vec<(&'static str, ExtensionEntries)> {
  std::vec![
    (
      "container::Format",
      extension_entries_of(crate::container::Format::ROSTER, |v| v.extensions()),
    ),
    (
      "audio::ContainerFormat",
      extension_entries_of(crate::audio::ContainerFormat::ROSTER, |v| v.extensions()),
    ),
    (
      "image::Format",
      extension_entries_of(crate::image::Format::ROSTER, |v| v.extensions()),
    ),
  ]
}

/// Asserts no two *different* vocabularies in `rosters` claim the same
/// extension — `rosters` is `(vocabulary name, its extension entries)`.
///
/// findit's directory-walk filter unions every extension-bearing
/// vocabulary's `ROSTER` into one `BTreeSet<&str>` (its own module doc
/// calls this "ONE question being asked twice rather than two lists to be
/// searched in turn") to decide which files are offered to the demuxer.
/// That union is only sound if the same extension is never claimed by two
/// *different* formats that a consumer might need to tell apart — a
/// duplicate **within** one format's own alias list is fine
/// (`ContainerFormat::Alac` and `ContainerFormat::M4a` both legitimately
/// extend `.m4a`: one string, either reading is correct), but a duplicate
/// **across** vocabularies would make "which format is this extension"
/// ambiguous the moment a caller (present or future) needs that answer
/// rather than a plain yes/no. Only the *owning vocabulary* is compared
/// for exactly that reason — two variants of the SAME vocabulary sharing
/// an extension is outside this check's concern, by design.
///
/// Factored out of [`container_audio_image_extensions_are_disjoint`] so
/// [`disjointness_check_catches_a_synthetic_cross_vocabulary_collision`]
/// can drive the identical logic against a deliberately-colliding
/// synthetic dataset — a checker that has only ever run against clean
/// real data and was never proven to fire on a real collision is not yet
/// proven to fire at all.
#[cfg(any(feature = "std", feature = "alloc"))]
fn assert_extensions_disjoint(rosters: std::vec::Vec<(&'static str, ExtensionEntries)>) {
  use std::collections::BTreeMap;

  // extension -> (owning vocabulary, owning variant's Debug) for every
  // extension seen so far, so a collision names both sides.
  let mut owner: BTreeMap<&'static str, (&'static str, std::string::String)> = BTreeMap::new();

  for (vocab, entries) in rosters {
    for (ext, variant) in entries {
      if ext.is_empty() {
        // `as_extension() == ""` is the documented "no known extension"
        // sentinel (never emitted by `extensions()`, which returns `&[]`
        // instead), not a real spelling to police.
        continue;
      }
      match owner.get(ext) {
        Some((prior_vocab, prior_variant)) => assert_eq!(
          *prior_vocab, vocab,
          "extension `{ext}` is claimed by both {prior_vocab}::{prior_variant} and \
           {vocab}::{variant} — the walk's BTreeSet union can no longer tell these apart"
        ),
        None => {
          owner.insert(ext, (vocab, variant));
        }
      }
    }
  }
}

/// Asserts `rosters`' gathered entry count for `vocab` is *exactly* the
/// sum of `expected_extensions_len` over that vocabulary's own `ROSTER` —
/// not `>=`, an exact match.
///
/// This is the general-purpose successor to spot-checking specific
/// aliases (`container_audio_image_extensions_are_disjoint`'s R2 version
/// checked four representative ones: `.m2ts`, `.aif`, `.oga`, `.ori`).
/// Codex R3's finding on that version: every one of those four happened
/// to be each variant's *second* `extensions()` entry, so a gatherer
/// regressed to `.iter().take(2)` per variant would still pass all four
/// checks while `.mts`/`.m2t`/`.aifc`/`.spx`/`.jpe`/`.hif` (third-or-later
/// entries at the time — `.hif` has since been excluded from every
/// roster outright, R8) silently vanished. A **count** cannot be fooled by *which*
/// entries a truncation drops — any drop, duplication, or off-by-one at
/// any position in any variant's list changes the sum, so this one
/// assertion subsumes what an unbounded number of spot-checks would
/// otherwise need to enumerate one alias at a time.
#[cfg(any(feature = "std", feature = "alloc"))]
fn assert_exact_gathered_cardinality(
  rosters: &[(&'static str, ExtensionEntries)],
  vocab: &'static str,
  expected: usize,
) {
  let actual = rosters
    .iter()
    .find(|(v, _)| *v == vocab)
    .map_or(0, |(_, entries)| entries.len());
  assert_eq!(
    actual, expected,
    "{vocab}: gatherer produced {actual} extension entries, but summing \
     extensions().len() over ROSTER directly gives {expected} — some alias \
     was silently dropped (or duplicated) by the gathering step; a `.take(N)`-style \
     truncation is exactly the shape of bug this catches, regardless of which \
     alias or variant it happens to hit"
  );
}

/// The runtime proof that the three extension-bearing vocabularies —
/// [`crate::container::Format`], [`crate::audio::ContainerFormat`], and
/// [`crate::image::Format`] — hold [`assert_extensions_disjoint`]'s
/// property today, the same way [`check`] proves within-roster
/// uniqueness. It is written to *report* an overlap rather than assume
/// there is none, per this crate's own honesty-over-silence convention: a
/// future roster addition that collides fails here with the offending
/// extension and both owning formats named, not with a silent
/// three-instead-of-four count somewhere downstream.
///
/// Every entry from every roster's `extensions()` is checked — not just
/// each type's `as_extension()` primary spelling — so a documented alias
/// (`container::Format::MpegTs`'s `.m2ts`, `audio::ContainerFormat::Aiff`'s
/// `.aifc`, …) colliding with another vocabulary would be caught here
/// too, not just a collision on the three primaries.
///
/// **Before trusting that**, this test asserts each vocabulary's gathered
/// entry count *exactly* matches summing `extensions().len()` directly
/// over its own `ROSTER` — see [`assert_exact_gathered_cardinality`]'s own
/// doc for why an exact count, not a handful of spot-checked aliases, is
/// what actually rules out a truncating gatherer. [`extension_entries_of`]
/// is the one place all the real `ROSTER` + `extensions()` data funnels
/// through before reaching [`assert_extensions_disjoint`]; a starved
/// gatherer would make the disjointness check below pass for the wrong
/// reason — *fewer* entries can only make a collision *less* likely to be
/// found, so "clean" from a starved gatherer looks identical to "clean"
/// from a complete one. This is the check that tells them apart, run
/// before the disjointness assertion so a starved gatherer fails here
/// first and names exactly how far off the count is.
///
/// Gated on `any(std, alloc)` — unlike [`check`], this one is not
/// tier-agnostic: all three vocabularies it names, plus the
/// `BTreeMap`/`String` bookkeeping, only exist at that tier. `roster_tests`
/// itself has no such gate (`check` must run at the no-alloc tier too), so
/// the gate lives on this function rather than the module.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn container_audio_image_extensions_are_disjoint() {
  let rosters = container_and_audio_and_image_extension_entries();

  assert_exact_gathered_cardinality(
    &rosters,
    "container::Format",
    crate::container::Format::ROSTER
      .iter()
      .map(|v| v.extensions().len())
      .sum(),
  );
  assert_exact_gathered_cardinality(
    &rosters,
    "audio::ContainerFormat",
    crate::audio::ContainerFormat::ROSTER
      .iter()
      .map(|v| v.extensions().len())
      .sum(),
  );
  assert_exact_gathered_cardinality(
    &rosters,
    "image::Format",
    crate::image::Format::ROSTER
      .iter()
      .map(|v| v.extensions().len())
      .sum(),
  );

  assert_extensions_disjoint(rosters);
}

/// Two synthetic vocabularies that exist for exactly one reason: to give
/// [`disjointness_check_catches_a_synthetic_cross_vocabulary_collision`] a
/// collision to drive through the *real* [`extension_entries_of`]
/// gathering path — synthetic `ROSTER` + accessor, not real container /
/// audio / image data.
///
/// **Each roster carries the colliding variant third**, behind two
/// decoys with distinct, non-colliding extensions
/// (`Decoy1`/`Decoy2` → `zzsynthetic-{a,b}-decoy{1,2}`, `Collider` →
/// `zzsynthetic-collision`). Codex R3's finding on the R2 version (a
/// single-variant roster per vocabulary): the collision was every
/// synthetic roster's *only* entry, so a gatherer regressed to
/// `.iter().take(1)` or `.take(2)` on the *roster* itself (as opposed to
/// per-variant `extensions()`, which [`assert_exact_gathered_cardinality`]
/// covers) would still carry the collision through untouched and this
/// test would keep passing — "one entry, positioned first" cannot
/// distinguish a gatherer that iterates the whole roster from one that
/// silently stops after N. Two unique, non-colliding entries ahead of the
/// collider close that gap: only a gatherer that genuinely walks the full
/// roster ever reaches the collision at all.
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug)]
enum SyntheticVocabA {
  Decoy1,
  Decoy2,
  Collider,
}
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug)]
enum SyntheticVocabB {
  Decoy1,
  Decoy2,
  Collider,
}
#[cfg(any(feature = "std", feature = "alloc"))]
const SYNTHETIC_VOCAB_A_ROSTER: &[SyntheticVocabA] = &[
  SyntheticVocabA::Decoy1,
  SyntheticVocabA::Decoy2,
  SyntheticVocabA::Collider,
];
#[cfg(any(feature = "std", feature = "alloc"))]
const SYNTHETIC_VOCAB_B_ROSTER: &[SyntheticVocabB] = &[
  SyntheticVocabB::Decoy1,
  SyntheticVocabB::Decoy2,
  SyntheticVocabB::Collider,
];
#[cfg(any(feature = "std", feature = "alloc"))]
fn synthetic_vocab_a_extensions(v: &SyntheticVocabA) -> &'static [&'static str] {
  match v {
    SyntheticVocabA::Decoy1 => &["zzsynthetic-a-decoy1"],
    SyntheticVocabA::Decoy2 => &["zzsynthetic-a-decoy2"],
    SyntheticVocabA::Collider => &["zzsynthetic-collision"],
  }
}
#[cfg(any(feature = "std", feature = "alloc"))]
fn synthetic_vocab_b_extensions(v: &SyntheticVocabB) -> &'static [&'static str] {
  match v {
    SyntheticVocabB::Decoy1 => &["zzsynthetic-b-decoy1"],
    SyntheticVocabB::Decoy2 => &["zzsynthetic-b-decoy2"],
    SyntheticVocabB::Collider => &["zzsynthetic-collision"],
  }
}

/// Proves [`assert_extensions_disjoint`] actually has teeth — and, unlike
/// the R1 version of this test, proves it **through the same gathering
/// path the real check uses**, not by calling
/// [`assert_extensions_disjoint`] directly with a hand-built collision.
///
/// Two rounds of Codex findings shaped this test's current construction:
/// - **R2**: the R1 version constructed pre-flattened `ExtensionEntries`
///   and skipped [`extension_entries_of`] entirely, so a regression in
///   the gatherer itself — the exact failure class R1 was fixing — would
///   have left both this test and the real disjointness test green.
///   Fixed by routing the synthetic collision through
///   [`extension_entries_of`] (via [`SyntheticVocabA`] / [`SyntheticVocabB`]
///   standing in for real roster types).
/// - **R3**: the R2 version's synthetic rosters had exactly one entry
///   each — the collider, first and only — so a gatherer that silently
///   stopped after the first roster item (or first two) would still
///   carry the collision through untouched, proving nothing about
///   whether the gatherer walks a *whole* roster. Fixed by giving each
///   synthetic roster two non-colliding decoys ahead of the collider —
///   see [`SyntheticVocabA`]'s own doc.
///
/// Fed a synthetic two-vocabulary dataset that deliberately collides on
/// one extension, it must panic and name both the extension and the two
/// owning `vocab::variant` pairs — "no collision found" and "incapable of
/// finding a collision" must not read identically from a green checkmark.
#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
#[should_panic(
  expected = "extension `zzsynthetic-collision` is claimed by both synthetic::VocabA::Collider and synthetic::VocabB::Collider"
)]
fn disjointness_check_catches_a_synthetic_cross_vocabulary_collision() {
  let rosters = std::vec![
    (
      "synthetic::VocabA",
      extension_entries_of(SYNTHETIC_VOCAB_A_ROSTER, synthetic_vocab_a_extensions),
    ),
    (
      "synthetic::VocabB",
      extension_entries_of(SYNTHETIC_VOCAB_B_ROSTER, synthetic_vocab_b_extensions),
    ),
  ];
  assert_extensions_disjoint(rosters);
}
