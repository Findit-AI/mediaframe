#![doc = include_str!("../README.md")]
//!
//! # Feature tiers
//!
//! The crate builds at three tiers, and the tier decides how *open*
//! its vocabularies are.
//!
//! | Tier | Features | Vocabularies |
//! |---|---|---|
//! | no-alloc | (none) | **closed** — an unrecognised slug is rejected |
//! | alloc | `alloc` | open — an unrecognised slug rides `Other(SmolStr)` |
//! | std | `std` (implies `alloc`) | as `alloc`, plus `std::error::Error` |
//!
//! `Other(SmolStr)` needs a heap, so it exists only at the `alloc` /
//! `std` tier. At the no-alloc tier the same enums are closed and their
//! [`FromStr`](core::str::FromStr) returns the vocabulary's own error
//! instead: **an error beats a wrong value**, and collapsing an unknown
//! name onto a named variant would be a wrong value. The *wire shape* is
//! the same at every tier (a slug either way) — only the openness
//! differs.
//!
//! Every gate on an alloc-tier item is spelled
//! `any(feature = "std", feature = "alloc")` rather than bare
//! `feature = "alloc"`, so the item cannot evaporate for a dependant
//! that turns on `std` alone.
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

// Alias `alloc as std` on no_std + alloc builds so code can use
// `std::vec::Vec` etc. uniformly across feature combos. When the
// `std` feature is on, the real `std` crate is already in scope via
// the prelude. The `unused_extern_crates` allow silences a
// rust-2018-idioms false positive — the alias is needed at use-time
// even though rustc can't see that statically.
#[cfg(all(not(feature = "std"), feature = "alloc"))]
#[allow(unused_extern_crates)]
extern crate alloc as std;

#[cfg(feature = "std")]
#[allow(unused_extern_crates)]
extern crate std;

/// The doc text every `*Row::for_tests` carries, written once.
///
/// Declared here, ahead of `mod frame` and `mod source`, because both
/// hold row types and `macro_rules!` is textually scoped. One text in
/// one place — the same rule the row door itself enforces on colour
/// intent.
macro_rules! row_test_door_doc {
  () => {
    " Builds a row directly — **kernel-parity test scaffolding, not API.**\n\
      \n\
      [`Self::new`] is `pub(crate)`. Production rows come from the walkers,\n\
      which read the colour selector off the sink, so on the public surface\n\
      a row cannot be conjured beside the description that chose it. This\n\
      door is the one named exception to that.\n\
      \n\
      It exists because a kernel-parity suite drives a single row kernel\n\
      without materialising a frame, and there is no other way to reach one\n\
      from outside this crate. Not hypothetical: a census on 2026-08-19\n\
      found **493 such constructions across 85 files and 52 row types** in\n\
      `pixon` alone, every one of them test code.\n\
      \n\
      Takes exactly what `new` takes, colour selector included, so the two\n\
      cannot drift. **No stability promise** — it is `#[doc(hidden)]` and\n\
      may change or vanish in any release. If you are not testing a row\n\
      kernel, walk a frame instead."
  };
}

/// Hand-written [`arbitrary::Arbitrary`] impls for the descriptor vocabulary
/// (codecs, container/subtitle/audio formats, capture, language, colour, pixel
/// format, frame geometry/orientation, disposition). All generation goes through
/// the types' public constructors so private fields stay encapsulated and
/// `try_new` validated types come out valid by construction. Mirrors the
/// surface covered by [`serde`](serde_impls) — the same descriptor set the
/// storage / wire layers serialize.
#[cfg(feature = "arbitrary")]
mod arbitrary_impls;
/// Audio-stream descriptor vocabulary — channel layout, sample /
/// container format, bit-rate mode, EBU R128 loudness, fingerprint,
/// embedded metadata tags + cover art. Requires the `alloc` feature
/// (`std` includes it) for the `Other(SmolStr)` escape arms and the
/// `Vec<u8>` payloads.
///
/// **Derive threshold.** Every open enum here carries `Unwrap` /
/// `TryUnwrap` for its `Other(SmolStr)` arm. The pair generates three
/// methods per variant, so an enum in the hundreds pays that in compile
/// time for one reachable payload arm; the two 200-plus-variant codec
/// enums in [`codec`] are the crate's only exemptions. The line is
/// variant count, not principle.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod audio;
#[cfg(feature = "buffa")]
mod buffa;
/// EXIF / capture-metadata vocabulary — capture device, geographic
/// location (with ISO-6709 parse/format). Requires the `alloc`
/// feature (`std` includes it) because the constituent types lean on
/// `SmolStr` / `std::string::String` for their text surface.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod capture;
/// Stream-descriptor codec/format/layout vocabulary for video, audio, and
/// subtitle tracks. Requires the `alloc` feature (`std` includes it) for
/// the `Other(SmolStr)` escape arms.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod codec;
pub mod color;
/// Top-level multimedia container-format vocabulary. Requires the
/// `alloc` feature (`std` includes it) for the `Other(SmolStr)`
/// escape arm.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod container;
/// FFmpeg `AV_DISPOSITION_*` bitflags shared across all track types
/// (video / audio / subtitle).
pub mod disposition;
pub mod frame;
/// Validated BCP-47 language tag wrapping `icu_locale_core` subtags
/// (`Copy`, heap-free representation; `to_bcp47() -> String` and
/// `Display` need the allocator).
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod lang;
// The ASCII case-folding gate shared by every `FromStr` in the crate.
// Private: the errors those parses return live with their vocabularies,
// one per type.
mod parse;
pub mod pixel_format;
/// `fn(&mut quickcheck::Gen) -> T` helpers consumed by the per-type
/// `#[quickcheck(arbitrary = "…")]` attributes on each descriptor's
/// `quickcheck-richderive::Arbitrary` derive. The derive emits the actual
/// `impl quickcheck::Arbitrary for T` blocks; this module owns the bodies.
/// Same surface as [`arbitrary_impls`] (44 descriptor-vocabulary types) but
/// the two are independent — quickcheck does **not** bridge through arbitrary.
#[cfg(feature = "quickcheck")]
#[cfg_attr(docsrs, doc(cfg(feature = "quickcheck")))]
pub mod quickcheck_helpers;
/// Centralised `serde` impls for the descriptor enums (the structs derive
/// serde at their definition sites). Open codec/format enums serialize as
/// their `as_str()` slug; closed FFmpeg-coded enums as their `to_u32()`
/// code — mirroring the storage backends.
#[cfg(feature = "serde")]
mod serde_impls;
pub mod source;
/// Subtitle-stream descriptor vocabulary — file / demuxer format
/// ([`subtitle::Format`]) and track-origin axis
/// ([`subtitle::TrackOrigin`]). Requires the `alloc`
/// feature (`std` includes it) for the [`subtitle::Format`]'s
/// `Other(SmolStr)` escape arm.
#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "std", feature = "alloc"))))]
pub mod subtitle;

pub use source::{PixelSink, SourceFormat};
