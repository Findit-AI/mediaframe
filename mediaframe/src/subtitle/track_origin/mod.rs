//! [`TrackOrigin`] — provenance axis for a subtitle track:
//! where the bytes come from relative to the media file.

use derive_more::{Display, IsVariant};

/// Where this subtitle track came from, relative to the media file
/// it accompanies.
///
/// Closed unit-only enum (no `Unknown` / `Other` escape): every
/// subtitle track in practice falls into exactly one of these
/// buckets, and the wire id is stable / append-only. `Embedded` is
/// the default — the typical case is a subtitle stream multiplexed
/// inside the container.
///
/// `#[non_exhaustive]` is set anyway so a future expansion (e.g. a
/// distinct "broadcast" origin) is non-breaking.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::track_origin")
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
}

impl TrackOrigin {
  /// Canonical lowercase slug for this origin (`"embedded"` /
  /// `"sidecar"` / `"external"` / `"derived"`). Stable; matches what
  /// [`core::fmt::Display`] produces.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Embedded => "embedded",
      Self::Sidecar => "sidecar",
      Self::External => "external",
      Self::Derived => "derived",
    }
  }

  /// Stable `u32` wire id: `Embedded=0`, `Sidecar=1`, `External=2`,
  /// `Derived=3`. Append-only — never renumber.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> u32 {
    match self {
      Self::Embedded => 0,
      Self::Sidecar => 1,
      Self::External => 2,
      Self::Derived => 3,
    }
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unknown ids fall back to the default
  /// ([`Self::Embedded`]) — this is a closed enum with no lossless
  /// escape arm at all, so the round-trip is exact only for
  /// the enumerated ids `0`/`1`/`2`/`3`.
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
}

/// The error [`TrackOrigin`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Opaque and sealed: the input is deliberately not retained (these types
/// are available at the crate's no-alloc tier, where there is nowhere to
/// put an owned copy, and the input is attacker-controlled on the
/// deserialization path). `#[non_exhaustive]` keeps it constructible only
/// here, so it can grow structure later without breaking callers.
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
  /// Returns [`ParseTrackOriginError`] for any input
  /// outside this closed vocabulary.
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
      _ => return Err(ParseTrackOriginError),
    })
  }
}

#[cfg(test)]
mod tests;
