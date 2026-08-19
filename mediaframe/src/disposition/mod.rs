//! [`TrackDisposition`] — FFmpeg `AV_DISPOSITION_*` bitflags shared
//! across video, audio, and subtitle tracks.
//!
//! Direct mirror of FFmpeg's `libavformat/avformat.h` disposition
//! constants as of n9.0. Bit values are **stable** — append-only, never
//! renumbered — matching FFmpeg's contract; an unrecognised bit on the
//! wire round-trips losslessly through [`Self::from_u32`] /
//! [`Self::to_u32`] via the underlying [`bitflags`] storage, which
//! preserves every bit (`from_bits_retain` semantics) regardless of
//! whether it has a named constant in this version of the crate.
//!
//! This file replaces the placeholder `disposition: u32` shape used in
//! the mediaschema track aggregates (see
//! `schema/bitflags.md` r4 / `mediaschema::domain::bitflags`'s note
//! that `TrackDisposition` is "moved to `::mediaframe`").

use bitflags::bitflags;

bitflags! {
    /// FFmpeg `AV_DISPOSITION_*` flags — track-level metadata hints
    /// shared across video / audio / subtitle streams (e.g.
    /// `DEFAULT`, `FORCED`, `HEARING_IMPAIRED`).
    ///
    /// Bit values mirror `libavformat/avformat.h` as of FFmpeg n9.0
    /// and are **append-only** (never renumber). Unknown bits
    /// passing through [`TrackDisposition::from_u32`] /
    /// [`TrackDisposition::to_u32`] are preserved verbatim
    /// (`from_bits_retain` semantics under the hood), so the
    /// round-trip is lossless for any wire value — including bits
    /// added in a future FFmpeg release before this file is updated.
    ///
    /// **Default convention**: `Default::default()` returns the
    /// empty flag set (no disposition hints).
    ///
    /// # Text form
    ///
    /// This type deliberately has **no** `Display` / `FromStr`, and its
    /// serde shape is the `u32` bits. A vocabulary of *names* takes a
    /// text form; a bit *set* takes a number. A flag-set grammar
    /// (`"default|forced"`) would also have two shapes rather than one,
    /// because `from_bits_retain` keeps bits this build has no name for
    /// and those can only be printed as a bare literal. Human-readable
    /// names live in `Debug` and in whatever consumer surface wants
    /// them.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[cfg_attr(
      feature = "quickcheck",
      derive(::quickcheck_richderive::Arbitrary),
      quickcheck(arbitrary = "crate::quickcheck_helpers::coded::track_disposition")
    )]
    pub struct TrackDisposition: u32 {
        /// `AV_DISPOSITION_DEFAULT` — the default track of its kind.
        const DEFAULT          = 0x0000_0001;
        /// `AV_DISPOSITION_DUB` — dubbed audio.
        const DUB              = 0x0000_0002;
        /// `AV_DISPOSITION_ORIGINAL` — the original-language track.
        const ORIGINAL         = 0x0000_0004;
        /// `AV_DISPOSITION_COMMENT` — commentary audio.
        const COMMENT          = 0x0000_0008;
        /// `AV_DISPOSITION_LYRICS` — lyrics subtitle track.
        const LYRICS           = 0x0000_0010;
        /// `AV_DISPOSITION_KARAOKE` — karaoke subtitle track.
        const KARAOKE          = 0x0000_0020;
        /// `AV_DISPOSITION_FORCED` — forced subtitle track (shown
        /// regardless of user subtitle settings).
        const FORCED           = 0x0000_0040;
        /// `AV_DISPOSITION_HEARING_IMPAIRED` — track for the
        /// hearing-impaired (closed captions, SDH).
        const HEARING_IMPAIRED = 0x0000_0080;
        /// `AV_DISPOSITION_VISUAL_IMPAIRED` — descriptive audio
        /// track for the visually-impaired.
        const VISUAL_IMPAIRED  = 0x0000_0100;
        /// `AV_DISPOSITION_CLEAN_EFFECTS` — clean-effects audio
        /// (music + effects, no dialogue).
        const CLEAN_EFFECTS    = 0x0000_0200;
        /// `AV_DISPOSITION_ATTACHED_PIC` — the stream is an attached
        /// picture (e.g. an album-art `APIC` frame).
        const ATTACHED_PIC     = 0x0000_0400;
        /// `AV_DISPOSITION_TIMED_THUMBNAILS` — the stream carries
        /// thumbnails for sparse-sample rendering.
        const TIMED_THUMBNAILS = 0x0000_0800;
        /// `AV_DISPOSITION_NON_DIEGETIC` — the audio is non-diegetic
        /// (originates outside the on-screen action — narration,
        /// musical score).
        const NON_DIEGETIC     = 0x0000_1000;
        /// `AV_DISPOSITION_CAPTIONS` — the subtitle track is
        /// captions (transcribes dialogue + non-dialogue sounds for
        /// the deaf / hard-of-hearing). Distinct from
        /// [`Self::HEARING_IMPAIRED`].
        const CAPTIONS         = 0x0001_0000;
        /// `AV_DISPOSITION_DESCRIPTIONS` — the track carries
        /// textual descriptions of on-screen action.
        const DESCRIPTIONS     = 0x0002_0000;
        /// `AV_DISPOSITION_METADATA` — the stream carries metadata
        /// rather than primary content.
        const METADATA         = 0x0004_0000;
        /// `AV_DISPOSITION_DEPENDENT` — the audio stream is mixed
        /// with / dependent on another audio stream.
        const DEPENDENT        = 0x0008_0000;
        /// `AV_DISPOSITION_STILL_IMAGE` — the video stream contains
        /// a still image.
        const STILL_IMAGE      = 0x0010_0000;
    }
}

impl TrackDisposition {
  /// Canonical no-arg constructor — the empty flag set (no
  /// disposition hints). [`Default::default`] is `Self::new()`.
  #[inline]
  pub const fn new() -> Self {
    Self::empty()
  }
}

impl Default for TrackDisposition {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl TrackDisposition {
  /// Stable `u32` wire id — the raw bits, equivalent to
  /// [`Self::bits`] but named for the conventional
  /// `to_u32`/`from_u32` pair every mediaframe enum exposes.
  #[inline]
  pub const fn to_u32(self) -> u32 {
    self.bits()
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. Unknown bits are **preserved** (round-trip
  /// is lossless even for bits FFmpeg adds in a future release).
  /// Equivalent to [`bitflags::Flags::from_bits_retain`].
  #[inline]
  pub const fn from_u32(v: u32) -> Self {
    Self::from_bits_retain(v)
  }
}

#[cfg(test)]
mod tests;
