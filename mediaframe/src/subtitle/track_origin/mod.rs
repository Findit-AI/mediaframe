//! [`TrackOrigin`] — provenance axis for a subtitle track:
//! where the bytes come from relative to the media file.

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_str::SmolStr;

/// Where this subtitle track came from, relative to the media file
/// it accompanies.
///
/// Open vocabulary: the named variants cover the buckets a media
/// pipeline sorts tracks into, and [`Self::Other`] carries a
/// classification this build does not name. mediaframe is a shared
/// library, not one pipeline's private enum — the set of provenances
/// worth distinguishing belongs to whoever is doing the classifying,
/// so a downstream that tracks a distinction this crate has not heard
/// of keeps its **name** rather than losing it to a nearby variant.
///
/// `Embedded` is the default — the typical case is a subtitle stream
/// multiplexed inside the container. The named variants keep stable,
/// append-only `u32` ids; [`Self::Other`] has none (see
/// [`Self::to_u32`]), so the slug from [`Self::as_str`] is the
/// spelling that always survives.
///
/// `#[non_exhaustive]` is retained: promoting a slug that rides
/// `Other` today into a named variant tomorrow stays a minor change.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::track_origin")
)]
pub enum TrackOrigin {
  /// Stream multiplexed into the container alongside the video /
  /// audio tracks (e.g. an `.mkv` with embedded `.srt`-equivalent
  /// subtitle streams). The default origin.
  #[default]
  Embedded,
  /// Separate subtitle file living next to the media file on disk
  /// — typically an external `.srt` / `.vtt` / `.ass` paired with
  /// the video by filename stem.
  Sidecar,
  /// Externally sourced — downloaded from an online subtitle
  /// database, or otherwise obtained ready-made from outside the
  /// original media container. The track existed as subtitle text
  /// before this pipeline saw the media; contrast [`Self::Derived`],
  /// which this pipeline produced from the media itself.
  External,
  /// Produced by a processing pass over the media rather than
  /// obtained as subtitle text — an automatic-speech-recognition
  /// transcript, a machine translation of another track, OCR of
  /// image-based subtitles, or any comparable derivation.
  ///
  /// The axis this variant adds is *provenance-by-derivation*: the
  /// bytes have no existence independent of a pipeline stage, so
  /// their quality, licensing and re-generability differ from a
  /// track that arrived as text. [`Self::External`] covers the
  /// arrived-as-text case even when the arrival was a download.
  Derived,
  /// A provenance this vocabulary does not name — carried verbatim,
  /// ASCII-folded to lowercase by the parse gate. The crate-wide
  /// extension idiom: a downstream classifier naming an origin
  /// mediaframe has never heard of keeps that **name**, and it
  /// round-trips through `as_str` / `FromStr` / `serde` intact.
  Other(SmolStr),
}

impl TrackOrigin {
  /// Canonical lowercase slug for this origin (`"embedded"` /
  /// `"sidecar"` / `"external"` / `"derived"`). Stable; matches what
  /// [`core::fmt::Display`] produces. [`Self::Other`] returns the
  /// wrapped slug verbatim.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Embedded => "embedded",
      Self::Sidecar => "sidecar",
      Self::External => "external",
      Self::Derived => "derived",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id for the named origins: `Embedded=0`,
  /// `Sidecar=1`, `External=2`, `Derived=3`. Append-only — never
  /// renumber.
  ///
  /// [`None`] for [`Self::Other`]: it names an origin this build has
  /// no id for, and inventing one would lose the name. The slug from
  /// [`Self::as_str`] is the spelling that always survives — which is
  /// why the wire codecs (`serde`, `buffa`) carry the slug and not
  /// this number.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Embedded => 0,
      Self::Sidecar => 1,
      Self::External => 2,
      Self::Derived => 3,
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unknown ids fall back to the default
  /// ([`Self::Embedded`]): the numeric space is this crate's own and
  /// an id it never assigned carries no name to preserve, so
  /// [`Self::Other`] is not the right home for one. The round-trip is
  /// exact for the enumerated ids `0`/`1`/`2`/`3`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Self {
    match v {
      0 => Self::Embedded,
      1 => Self::Sidecar,
      2 => Self::External,
      3 => Self::Derived,
      _ => Self::Embedded,
    }
  }

  /// Strict counterpart to [`Self::from_u32`]: returns `None` for any code
  /// outside the enumerated set, instead of silently mapping it to the
  /// default. Used by the strict deserialize path so adversarial / corrupt
  /// wire values fail loudly rather than masquerading as `Embedded`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_from_u32(v: u32) -> Option<Self> {
    match v {
      0 => Some(Self::Embedded),
      1 => Some(Self::Sidecar),
      2 => Some(Self::External),
      3 => Some(Self::Derived),
      _ => None,
    }
  }

  /// The open escape for a slug this vocabulary does not name, ASCII-folded
  /// to the crate's lowercase canon.
  ///
  /// The **one** construction path for [`Self::Other`]: folding here is what
  /// keeps the whole value space lowercase-canonical, so the derived `Eq` /
  /// `Hash` compare names rather than spellings. Constructing the variant
  /// directly bypasses the fold and is not the supported spelling.
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::Other(crate::parse::fold_owned(slug.as_ref()))
  }
}

roster!(
  TrackOrigin,
  "track origin",
  [Embedded, Sidecar, External, Derived],
  escape: Other
);

/// The error [`TrackOrigin`]'s [`FromStr`](core::str::FromStr) declares.
///
/// **Never returned as of 0.5.0.** [`TrackOrigin`] gained its
/// [`Other`](TrackOrigin::Other) escape in that release, and this module
/// exists only at the `alloc` tier, so every slug now parses. The type is
/// kept — exported and named by `FromStr::Err` — because narrowing the
/// associated type is itself a breaking change, and keeping it leaves the
/// door open to a future no-alloc tier for this module where the
/// vocabulary would close again and the refusal become real.
///
/// Opaque and sealed: the input is deliberately not retained (the input is
/// attacker-controlled on the deserialization path). `#[non_exhaustive]`
/// keeps it constructible only here, so it can grow structure later
/// without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a track-origin name")]
#[non_exhaustive]
pub struct ParseTrackOriginError;

impl core::str::FromStr for TrackOrigin {
  type Err = ParseTrackOriginError;

  /// Parses the canonical slug [`Self::as_str`] renders — the exact
  /// inverse of [`Display`](core::fmt::Display).
  ///
  /// # Errors
  ///
  /// Never — this parse is **total**: a slug this type does not name
  /// rides [`Self::Other`], ASCII-folded to lowercase by
  /// [`Self::other`]. The [`ParseTrackOriginError`] in the signature
  /// is retained for the reason given on that type.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s.as_bytes());
    Ok(match folded {
      b"embedded" => Self::Embedded,
      b"sidecar" => Self::Sidecar,
      b"external" => Self::External,
      b"derived" => Self::Derived,
      _ => Self::other(s),
    })
  }
}

#[cfg(test)]
mod tests;
