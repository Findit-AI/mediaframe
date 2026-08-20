//! Audio channel layout vocabulary — the common named layouts plus
//! an `Other(SmolStr)` lossless escape for anything outside the
//! closed set.
//!
//! The named variants cover the `AV_CH_LAYOUT_*` shapes FFmpeg n9.0
//! exposes; layouts not enumerated here (custom orderings, ambisonic
//! variants beyond first-order, etc.) round-trip through
//! [`ChannelLayout::Other`] carrying the FFmpeg-canonical slug
//! verbatim.
//!
//! **Two namings, and they do not line up.** A variant's *ident* follows
//! the FFmpeg constant (`N5Point0Back` ⇒ `AV_CH_LAYOUT_5POINT0_BACK`);
//! its *slug* follows the name FFmpeg's `channel_layout_map[]` gives
//! that constant. For the 5.x family those disagree — FFmpeg hands the
//! unqualified `"5.0"` / `"5.1"` to the **back**-speaker layouts and
//! qualifies the side ones `"5.0(side)"` / `"5.1(side)"` — so four arms
//! read crossed. They are right; the map is the authority, and
//! `channel_layout_slugs_match_ffmpegs_map` pins every one of them.

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_str::SmolStr;

/// Audio channel layout — the common named layouts plus an
/// `Other(SmolStr)` lossless escape.
///
/// Read from FFmpeg `AV_CH_LAYOUT_*` constants (`AVChannelLayout`'s
/// canonical name) / WebCodecs `AudioData.channelLayout`. Named
/// variants are the universally-understood shapes; layouts FFmpeg
/// can describe but this enum doesn't enumerate (e.g.
/// `"hexadecagonal"`, `"22.2"`, ambisonic groupings beyond
/// `Ambisonic1`/`2`/`3`) round-trip through [`Self::Other`] carrying
/// the FFmpeg-canonical slug verbatim — never silently collapsed.
///
/// `#[non_exhaustive]` keeps future additions non-breaking. `Display`
/// renders via [`Self::as_str`].
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::strings::channel_layout")
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant, Unwrap, TryUnwrap)]
#[display("{}", self.as_str())]
#[unwrap(ref, ref_mut)]
#[try_unwrap(ref, ref_mut)]
#[non_exhaustive]
pub enum ChannelLayout {
  /// Single channel: `"mono"` (FFmpeg `AV_CH_LAYOUT_MONO`).
  Mono,
  /// L+R: `"stereo"` (FFmpeg `AV_CH_LAYOUT_STEREO`).
  Stereo,
  /// L+R+LFE: `"2.1"` (FFmpeg `AV_CH_LAYOUT_2POINT1`).
  N2Point1,
  /// L+R+C: `"3.0"` (FFmpeg `AV_CH_LAYOUT_SURROUND`).
  N3Point0,
  /// L+R+BC (back-center surround): `"3.0(back)"` (FFmpeg
  /// `AV_CH_LAYOUT_2_1`).
  N3Point0Back,
  /// L+R+C+LFE: `"3.1"` (FFmpeg `AV_CH_LAYOUT_3POINT1`).
  N3Point1,
  /// L+R+BL+BR: `"quad"` (FFmpeg `AV_CH_LAYOUT_QUAD` =
  /// `STEREO|BACK_LEFT|BACK_RIGHT`). The **side** four-channel layout
  /// is FFmpeg's `AV_CH_LAYOUT_2_2`, named `"quad(side)"`, which this
  /// vocabulary does not enumerate — it rides [`Self::Other`].
  Quad,
  /// L+R+C+SL+SR: `"5.0(side)"` (FFmpeg `AV_CH_LAYOUT_5POINT0` =
  /// `SURROUND|SIDE_LEFT|SIDE_RIGHT`).
  ///
  /// FFmpeg's plain `"5.0"` is the **back** layout, not this one — see
  /// [`Self::N5Point0Back`]. The two spellings look inverted against the
  /// variant idents and are not: the ident follows the FFmpeg constant,
  /// the slug follows FFmpeg's name for it, and FFmpeg gave the
  /// unqualified name to the back-speaker layout.
  N5Point0,
  /// L+R+C+BL+BR: `"5.0"` (FFmpeg `AV_CH_LAYOUT_5POINT0_BACK` =
  /// `SURROUND|BACK_LEFT|BACK_RIGHT`).
  ///
  /// The unqualified `"5.0"` is FFmpeg's name for the back-speaker
  /// layout; the side-speaker one is [`Self::N5Point0`], spelled
  /// `"5.0(side)"`.
  N5Point0Back,
  /// L+R+C+LFE+SL+SR: `"5.1(side)"` (FFmpeg `AV_CH_LAYOUT_5POINT1` =
  /// `5POINT0|LOW_FREQUENCY`).
  ///
  /// FFmpeg's plain `"5.1"` is the **back** layout — see
  /// [`Self::N5Point1Back`].
  N5Point1,
  /// L+R+C+LFE+BL+BR: `"5.1"` (FFmpeg `AV_CH_LAYOUT_5POINT1_BACK` =
  /// `5POINT0_BACK|LOW_FREQUENCY`).
  ///
  /// This is what an FFmpeg-sourced `"5.1"` means — the historically
  /// unqualified spelling belongs to the back-speaker layout, and the
  /// side-speaker one is [`Self::N5Point1`], spelled `"5.1(side)"`.
  N5Point1Back,
  /// 6.0: `"6.0"` (FFmpeg `AV_CH_LAYOUT_6POINT0`).
  N6Point0,
  /// 6.1: `"6.1"` (FFmpeg `AV_CH_LAYOUT_6POINT1`).
  N6Point1,
  /// 7.0: `"7.0"` (FFmpeg `AV_CH_LAYOUT_7POINT0`).
  N7Point0,
  /// 7.1: `"7.1"` (FFmpeg `AV_CH_LAYOUT_7POINT1`).
  N7Point1,
  /// Hexagonal (6 channels in a hexagon, no LFE): `"hexagonal"`
  /// (FFmpeg `AV_CH_LAYOUT_HEXAGONAL`).
  Hexagonal,
  /// Octagonal (8 channels around): `"octagonal"` (FFmpeg
  /// `AV_CH_LAYOUT_OCTAGONAL`).
  Octagonal,
  /// First-order Ambisonic B-format (WXYZ, 4 channels): `"ambisonic1"`.
  Ambisonic1,
  /// Second-order Ambisonic (9 channels): `"ambisonic2"`.
  Ambisonic2,
  /// Third-order Ambisonic (16 channels): `"ambisonic3"`.
  Ambisonic3,
  /// A layout not enumerated above — carries the FFmpeg-canonical
  /// name verbatim (e.g. `"22.2"`, `"hexadecagonal"`, a custom
  /// layout description). Lossless escape.
  Other(SmolStr),
}

impl Default for ChannelLayout {
  /// `Other("")` — the wire-zero / "absent" sentinel. There is no
  /// universally-defensible default channel layout (mono vs stereo
  /// is context-dependent); the empty-string `Other` mirrors the
  /// `buffa`-compatible "absent" state. Callers picking a meaningful
  /// fallback should be explicit (`ChannelLayout::Stereo` is the
  /// common one).
  #[inline]
  fn default() -> Self {
    Self::Other(SmolStr::new_inline(""))
  }
}

impl ChannelLayout {
  /// FFmpeg-canonical layout slug (e.g. `"mono"`, `"stereo"`,
  /// `"5.1"`, `"7.1"`). [`Self::Other`] returns the wrapped string
  /// verbatim.
  pub fn as_str(&self) -> &str {
    match self {
      Self::Mono => "mono",
      Self::Stereo => "stereo",
      Self::N2Point1 => "2.1",
      Self::N3Point0 => "3.0",
      Self::N3Point0Back => "3.0(back)",
      Self::N3Point1 => "3.1",
      Self::Quad => "quad",
      // FFmpeg gives the unqualified name to the BACK layout and
      // qualifies the side one — see `channel_layout_map[]`. The idents
      // follow the constants, the slugs follow FFmpeg's names, so these
      // four read crossed and are exactly right.
      Self::N5Point0 => "5.0(side)",
      Self::N5Point0Back => "5.0",
      Self::N5Point1 => "5.1(side)",
      Self::N5Point1Back => "5.1",
      Self::N6Point0 => "6.0",
      Self::N6Point1 => "6.1",
      Self::N7Point0 => "7.0",
      Self::N7Point1 => "7.1",
      Self::Hexagonal => "hexagonal",
      Self::Octagonal => "octagonal",
      Self::Ambisonic1 => "ambisonic1",
      Self::Ambisonic2 => "ambisonic2",
      Self::Ambisonic3 => "ambisonic3",
      Self::Other(s) => s.as_str(),
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
  ChannelLayout,
  "channel layout",
  [
    Mono, Stereo, N2Point1, N3Point0, N3Point0Back, N3Point1, Quad, N5Point0,
    N5Point0Back, N5Point1, N5Point1Back, N6Point0, N6Point1, N7Point0,
    N7Point1, Hexagonal, Octagonal, Ambisonic1, Ambisonic2, Ambisonic3
  ],
  escape: Other
);

impl FromStr for ChannelLayout {
  type Err = core::convert::Infallible;
  /// Recognise a canonical layout slug; unknown values land in
  /// [`Self::Other`] (infallible, lossless).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s.as_bytes());
    Ok(match folded {
      b"mono" => Self::Mono,
      b"stereo" => Self::Stereo,
      b"2.1" => Self::N2Point1,
      b"3.0" => Self::N3Point0,
      b"3.0(back)" => Self::N3Point0Back,
      b"3.1" => Self::N3Point1,
      b"quad" => Self::Quad,
      b"5.0(side)" => Self::N5Point0,
      b"5.0" => Self::N5Point0Back,
      b"5.1(side)" => Self::N5Point1,
      b"5.1" => Self::N5Point1Back,
      b"6.0" => Self::N6Point0,
      b"6.1" => Self::N6Point1,
      b"7.0" => Self::N7Point0,
      b"7.1" => Self::N7Point1,
      b"hexagonal" => Self::Hexagonal,
      b"octagonal" => Self::Octagonal,
      b"ambisonic1" => Self::Ambisonic1,
      b"ambisonic2" => Self::Ambisonic2,
      b"ambisonic3" => Self::Ambisonic3,
      _ => Self::other(s),
    })
  }
}

#[cfg(test)]
mod tests;
