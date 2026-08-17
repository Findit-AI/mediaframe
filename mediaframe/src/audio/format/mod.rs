//! Audio sample-format vocabulary (`SampleFormat`, FFmpeg
//! `AVSampleFormat`) and audio-only container-format vocabulary
//! (`ContainerFormat`, audio file extensions).

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_str::SmolStr;

/// Audio sample format — FFmpeg `AVSampleFormat`.
///
/// One named variant per FFmpeg n8.1 sample format (the standard 12
/// — `u8`/`s16`/`s32`/`s64` × packed/planar plus `flt`/`dbl` ×
/// packed/planar), with the planar variants suffixed `p` per FFmpeg
/// convention.
///
/// `to_u32` / `from_u32` use the FFmpeg `AV_SAMPLE_FMT_*` enum
/// indices (`U8 = 0`, `S16 = 1`, …, `S64P = 11`); unrecognised
/// codes round-trip via [`Self::Unknown`]. Slugs that don't match
/// any named variant round-trip via [`Self::Other`].
///
/// `#[non_exhaustive]` keeps future additions non-breaking.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::sample_format")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum SampleFormat {
  /// `AV_SAMPLE_FMT_U8` (code `0`) — unsigned 8-bit, packed.
  U8,
  /// `AV_SAMPLE_FMT_S16` (code `1`) — signed 16-bit, packed.
  S16,
  /// `AV_SAMPLE_FMT_S32` (code `2`) — signed 32-bit, packed.
  S32,
  /// `AV_SAMPLE_FMT_FLT` (code `3`) — 32-bit float, packed.
  Flt,
  /// `AV_SAMPLE_FMT_DBL` (code `4`) — 64-bit float, packed.
  Dbl,
  /// `AV_SAMPLE_FMT_U8P` (code `5`) — unsigned 8-bit, planar.
  U8p,
  /// `AV_SAMPLE_FMT_S16P` (code `6`) — signed 16-bit, planar.
  S16p,
  /// `AV_SAMPLE_FMT_S32P` (code `7`) — signed 32-bit, planar.
  S32p,
  /// `AV_SAMPLE_FMT_FLTP` (code `8`) — 32-bit float, planar.
  Fltp,
  /// `AV_SAMPLE_FMT_DBLP` (code `9`) — 64-bit float, planar.
  Dblp,
  /// `AV_SAMPLE_FMT_S64` (code `10`) — signed 64-bit, packed.
  S64,
  /// `AV_SAMPLE_FMT_S64P` (code `11`) — signed 64-bit, planar.
  S64p,
  /// A format slug not enumerated above — carries the slug verbatim
  /// (the [`Self::from_str`] lossless escape).
  Other(SmolStr),
}

impl Default for SampleFormat {
  /// `Other("")` — the wire-zero / "absent" sentinel, matching
  /// [`ContainerFormat`]. FFmpeg's `AV_SAMPLE_FMT_NONE` is `-1`, outside
  /// the `u32` code space, so it has no numeric spelling here; the empty
  /// slug is the one that round-trips.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Other(SmolStr::new_inline(""))
  }
}

impl SampleFormat {
  /// FFmpeg-canonical slug (`"u8"`, `"s16"`, `"flt"`, `"u8p"`, …).
  pub fn as_str(&self) -> &str {
    match self {
      Self::U8 => "u8",
      Self::S16 => "s16",
      Self::S32 => "s32",
      Self::Flt => "flt",
      Self::Dbl => "dbl",
      Self::U8p => "u8p",
      Self::S16p => "s16p",
      Self::S32p => "s32p",
      Self::Fltp => "fltp",
      Self::Dblp => "dblp",
      Self::S64 => "s64",
      Self::S64p => "s64p",
      Self::Other(s) => s.as_str(),
    }
  }

  /// The FFmpeg `AV_SAMPLE_FMT_*` enum index for the named variants —
  /// a boundary helper for FFmpeg interop, not a wire form.
  ///
  /// [`Self::Other`] returns [`None`]: it names a format FFmpeg has no
  /// code for, and inventing one would lose the name. The slug from
  /// [`Self::as_str`] is the spelling that always survives.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::U8 => 0,
      Self::S16 => 1,
      Self::S32 => 2,
      Self::Flt => 3,
      Self::Dbl => 4,
      Self::U8p => 5,
      Self::S16p => 6,
      Self::S32p => 7,
      Self::Fltp => 8,
      Self::Dblp => 9,
      Self::S64 => 10,
      Self::S64p => 11,
      Self::Other(_) => return None,
    })
  }

  /// Decodes an FFmpeg `AV_SAMPLE_FMT_*` code, or [`None`] if this build
  /// names no format for it. The numeric space is FFmpeg's, so an
  /// unrecognised code carries no name to preserve.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::U8,
      1 => Self::S16,
      2 => Self::S32,
      3 => Self::Flt,
      4 => Self::Dbl,
      5 => Self::U8p,
      6 => Self::S16p,
      7 => Self::S32p,
      8 => Self::Fltp,
      9 => Self::Dblp,
      10 => Self::S64,
      11 => Self::S64p,
      _ => return None,
    })
  }

  /// The open escape for a slug this vocabulary does not name, ASCII-folded
  /// to the crate's lowercase canon.
  ///
  /// The **one** construction path for [`Self::Other`]: folding here is what
  /// keeps the whole value space lowercase-canonical, so the derived `Eq` /
  /// `Hash` compare names rather than spellings.
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::Other(crate::parse::fold_owned(slug.as_ref()))
  }

  /// `true` for the planar layout variants (`*p`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_planar(&self) -> bool {
    matches!(
      self,
      Self::U8p | Self::S16p | Self::S32p | Self::Fltp | Self::Dblp | Self::S64p
    )
  }
}

impl FromStr for SampleFormat {
  type Err = core::convert::Infallible;
  /// Recognise a canonical FFmpeg sample-format slug; unknown
  /// values land in [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s);
    Ok(match folded {
      "u8" => Self::U8,
      "s16" => Self::S16,
      "s32" => Self::S32,
      "flt" => Self::Flt,
      "dbl" => Self::Dbl,
      "u8p" => Self::U8p,
      "s16p" => Self::S16p,
      "s32p" => Self::S32p,
      "fltp" => Self::Fltp,
      "dblp" => Self::Dblp,
      "s64" => Self::S64,
      "s64p" => Self::S64p,
      other => Self::other(other),
    })
  }
}

// ---------------------------------------------------------------------------

/// Audio-only file / container format vocabulary.
///
/// Top-level multimedia containers (`mp4`/`mkv`/`mov`/`webm`/…)
/// live on [`crate::container::Format`]; this enum
/// enumerates the **audio-only** containers (one audio stream, no
/// video). Closed-ish vocabulary — not FFmpeg-coded, so there is no
/// `to_u32`/`from_u32`; the `Other(SmolStr)` arm preserves unknown
/// slugs losslessly.
///
/// `as_str` returns the file-extension-style slug (`"mp3"`, `"aac"`,
/// `"flac"`, …).
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::audio_container_format")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum ContainerFormat {
  /// MPEG-1/2 Audio Layer III (`.mp3`). The auto-derived predicate
  /// name would be `is_mp_3` (digit-snake-case); the hand-written
  /// [`Self::is_mp3`] uses the cleaner name.
  #[is_variant(ignore)]
  Mp3,
  /// Raw AAC ADTS / ADIF stream (`.aac`).
  Aac,
  /// Free Lossless Audio Codec (`.flac`).
  Flac,
  /// Ogg Vorbis / generic Ogg container (`.ogg`).
  Ogg,
  /// Opus in Ogg or raw (`.opus`).
  Opus,
  /// RIFF WAVE (`.wav`).
  Wav,
  /// Audio Interchange File Format (`.aiff` / `.aif`).
  Aiff,
  /// Apple Lossless (ALAC) — usually carried inside `.m4a`,
  /// occasionally `.caf`; this variant is the bare-codec spelling.
  Alac,
  /// Windows Media Audio (`.wma`).
  Wma,
  /// Monkey's Audio (`.ape`).
  Ape,
  /// WavPack (`.wv`).
  Wv,
  /// Matroska Audio (`.mka`).
  Mka,
  /// MPEG-4 audio-only (`.m4a`) — AAC / ALAC in an MP4 box layout.
  /// The auto-derived predicate name would be `is_m_4_a`
  /// (digit-snake-case); the hand-written [`Self::is_m4a`] uses the
  /// cleaner name.
  #[is_variant(ignore)]
  M4a,
  /// Apple Core Audio Format (`.caf`).
  Caf,
  /// A container not enumerated above — carries the
  /// extension-style slug verbatim. Lossless escape.
  Other(SmolStr),
}

impl Default for ContainerFormat {
  /// `Other("")` — the wire-zero / "absent" sentinel. Audio
  /// containers vary by source; there is no universally-defensible
  /// default. Callers picking a meaningful fallback should be
  /// explicit.
  #[inline]
  fn default() -> Self {
    Self::Other(SmolStr::new_inline(""))
  }
}

impl ContainerFormat {
  /// True iff this is [`Self::Mp3`]. Hand-written to override the
  /// auto-derived `is_mp_3` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_mp3(&self) -> bool {
    matches!(self, Self::Mp3)
  }

  /// True iff this is [`Self::M4a`]. Hand-written to override the
  /// auto-derived `is_m_4_a` (digit-snake-case is ugly).
  #[inline(always)]
  pub const fn is_m4a(&self) -> bool {
    matches!(self, Self::M4a)
  }

  /// File-extension-style slug (`"mp3"`, `"aac"`, `"flac"`, …).
  pub fn as_str(&self) -> &str {
    match self {
      Self::Mp3 => "mp3",
      Self::Aac => "aac",
      Self::Flac => "flac",
      Self::Ogg => "ogg",
      Self::Opus => "opus",
      Self::Wav => "wav",
      Self::Aiff => "aiff",
      Self::Alac => "alac",
      Self::Wma => "wma",
      Self::Ape => "ape",
      Self::Wv => "wv",
      Self::Mka => "mka",
      Self::M4a => "m4a",
      Self::Caf => "caf",
      Self::Other(s) => s.as_str(),
    }
  }

  /// Primary file-on-disk extension (without the leading dot —
  /// `"mp3"`, `"flac"`, `"m4a"`, …). For most audio containers the
  /// extension matches the FFmpeg slug from [`Self::as_str`]; the
  /// exception is `Alac`, which has no standalone extension (the
  /// codec rides inside `.m4a`), so this method returns `"m4a"`.
  ///
  /// Returns `""` for [`Self::Other`] — the open variant carries an
  /// FFmpeg slug, not an extension, so the mapping is unknown.
  /// Returns `&'static str` (not `&str`) so the value is compile-time
  /// stable and the method is `const`.
  #[inline(always)]
  pub const fn as_extension(&self) -> &'static str {
    match self {
      Self::Mp3 => "mp3",
      Self::Aac => "aac",
      Self::Flac => "flac",
      Self::Ogg => "ogg",
      Self::Opus => "opus",
      Self::Wav => "wav",
      Self::Aiff => "aiff",
      Self::Alac => "m4a",
      Self::Wma => "wma",
      Self::Ape => "ape",
      Self::Wv => "wv",
      Self::Mka => "mka",
      Self::M4a => "m4a",
      Self::Caf => "caf",
      Self::Other(_) => "",
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

impl FromStr for ContainerFormat {
  type Err = core::convert::Infallible;
  /// Recognise a canonical extension-style slug; unknown values
  /// land in [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s);
    Ok(match folded {
      "mp3" => Self::Mp3,
      "aac" => Self::Aac,
      "flac" => Self::Flac,
      "ogg" => Self::Ogg,
      "opus" => Self::Opus,
      "wav" => Self::Wav,
      "aiff" => Self::Aiff,
      "alac" => Self::Alac,
      "wma" => Self::Wma,
      "ape" => Self::Ape,
      "wv" => Self::Wv,
      "mka" => Self::Mka,
      "m4a" => Self::M4a,
      "caf" => Self::Caf,
      other => Self::other(other),
    })
  }
}

#[cfg(test)]
mod tests;
