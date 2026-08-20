//! Audio channel layout vocabulary — the common named layouts plus
//! an `Other(SmolStr)` lossless escape for anything outside the
//! closed set.
//!
//! The named variants cover the `AV_CH_LAYOUT_*` shapes FFmpeg n9.0
//! exposes, less three this vocabulary has never carried (`"5.1.2"`,
//! `"9.1.6"`, `"binaural"`). Those three, custom orderings and ambisonic
//! groupings beyond third order round-trip through
//! [`ChannelLayout::Other`] carrying the FFmpeg-canonical slug verbatim.
//!
//! Channel abbreviations in the variant docs are FFmpeg's own
//! (`ffmpeg -layouts`): `FL`/`FR` front left/right, `FC` front centre,
//! `LFE` low frequency, `BL`/`BR` back left/right, `BC` back centre,
//! `SL`/`SR` side left/right, `FLC`/`FRC` front left/right-of-centre,
//! `WL`/`WR` wide left/right, `TC` top centre, `TFL`/`TFC`/`TFR` top
//! front left/centre/right, `TBL`/`TBC`/`TBR` top back
//! left/centre/right, `TSL`/`TSR` top side left/right,
//! `BFL`/`BFC`/`BFR` bottom front left/centre/right, `LFE2` second low
//! frequency, `DL`/`DR` downmix left/right.
//!
//! **Two namings, and they do not line up.** A variant's *ident* follows
//! the FFmpeg constant (`N5Point1Back` ⇒ `AV_CH_LAYOUT_5POINT1_BACK`);
//! its *slug* is the name FFmpeg's `channel_layout_map[]` gives that
//! constant — what `av_channel_layout_describe` prints, and therefore
//! what an FFmpeg-sourced string actually says. The two disagree three
//! ways, each of which has already cost this crate or its neighbours a
//! bug:
//!
//! 1. **The unqualified name went to whichever layout had it first.**
//!    FFmpeg hands plain `"5.0"`, `"5.1"` and `"7.1(wide)"` to the
//!    **back**-speaker layouts and qualifies the side ones `"5.0(side)"`,
//!    `"5.1(side)"`, `"7.1(wide-side)"`, so six arms read crossed
//!    against their idents. For the 5.1.2 family it runs the other way:
//!    `"5.1.2"` is the side layout and the back one is qualified
//!    `"5.1.2(back)"`. There is no rule to derive here — only the map.
//! 2. **`_BACK` in a constant is not `(back)` in a slug.**
//!    `5POINT1POINT4_BACK`, `7POINT1POINT4_BACK` and
//!    `9POINT1POINT4_BACK` are named `"5.1.4"`, `"7.1.4"` and `"9.1.4"`:
//!    there the suffix distinguishes top-*back* height channels, not
//!    surround placement, and FFmpeg never minted an unsuffixed sibling
//!    to qualify against.
//! 3. **Two idents follow the slug instead of the constant.**
//!    `AV_CH_LAYOUT_2_1` and `AV_CH_LAYOUT_2_2` are historical spellings
//!    that describe no arrangement anyone would recognise — and `2_1`
//!    read as an ident collides with the real `2POINT1`. They are
//!    [`ChannelLayout::N3Point0Back`] and [`ChannelLayout::QuadSide`],
//!    after `"3.0(back)"` and `"quad(side)"`.
//!
//! `channel_layout_slugs_match_ffmpegs_map` transcribes the whole map
//! and pins every arm, because nothing else catches a plausible-looking
//! "correction".

use core::str::FromStr;

use derive_more::{Display, IsVariant, TryUnwrap, Unwrap};
use smol_str::SmolStr;

/// Audio channel layout — the common named layouts plus an
/// `Other(SmolStr)` lossless escape.
///
/// Read from FFmpeg `AV_CH_LAYOUT_*` constants (`AVChannelLayout`'s
/// canonical name) / WebCodecs `AudioData.channelLayout`. Layouts FFmpeg
/// can describe but this enum doesn't enumerate (e.g. `"binaural"`,
/// `"9.1.6"`, ambisonic groupings beyond `Ambisonic1`/`2`/`3`)
/// round-trip through [`Self::Other`] carrying the FFmpeg-canonical slug
/// verbatim — never silently collapsed.
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
  /// `"mono"` — FC (FFmpeg `AV_CH_LAYOUT_MONO`).
  Mono,
  /// `"stereo"` — FL+FR (FFmpeg `AV_CH_LAYOUT_STEREO`).
  Stereo,
  /// `"downmix"` — DL+DR (FFmpeg `AV_CH_LAYOUT_STEREO_DOWNMIX`).
  ///
  /// The Dolby matrix-encoded stereo pair, carried as its own channel
  /// ids so a downmix is distinguishable from an original `Stereo`
  /// recording. FFmpeg names the constant after the source and the map
  /// entry after the result, hence the crossed spelling.
  StereoDownmix,
  /// `"2.1"` — FL+FR+LFE (FFmpeg `AV_CH_LAYOUT_2POINT1`).
  N2Point1,
  /// `"3.0"` — FL+FR+FC (FFmpeg `AV_CH_LAYOUT_SURROUND`).
  N3Point0,
  /// `"3.0(back)"` — FL+FR+BC (FFmpeg `AV_CH_LAYOUT_2_1`).
  ///
  /// Back-centre surround rather than a front centre. The ident follows
  /// the slug here: the constant's `2_1` is a historical spelling, and
  /// as an ident it would collide with the real 2.1 above.
  N3Point0Back,
  /// `"3.1"` — FL+FR+FC+LFE (FFmpeg `AV_CH_LAYOUT_3POINT1`).
  N3Point1,
  /// `"3.1.2"` — FL+FR+FC+LFE+TFL+TFR (FFmpeg
  /// `AV_CH_LAYOUT_3POINT1POINT2`).
  N3Point1Point2,
  /// `"4.0"` — FL+FR+FC+BC (FFmpeg `AV_CH_LAYOUT_4POINT0`).
  N4Point0,
  /// `"4.1"` — FL+FR+FC+LFE+BC (FFmpeg `AV_CH_LAYOUT_4POINT1`).
  N4Point1,
  /// `"quad"` — FL+FR+BL+BR (FFmpeg `AV_CH_LAYOUT_QUAD`).
  ///
  /// The **side** four-channel layout is [`Self::QuadSide`].
  Quad,
  /// `"quad(side)"` — FL+FR+SL+SR (FFmpeg `AV_CH_LAYOUT_2_2`).
  ///
  /// The ident follows the slug for the same reason
  /// [`Self::N3Point0Back`]'s does: the constant's `2_2` names no
  /// arrangement a reader would recognise, and reads as a "2.2" that
  /// does not exist.
  QuadSide,
  /// `"5.0(side)"` — FL+FR+FC+SL+SR (FFmpeg `AV_CH_LAYOUT_5POINT0`).
  ///
  /// FFmpeg's plain `"5.0"` is the **back** layout, not this one — see
  /// [`Self::N5Point0Back`].
  N5Point0,
  /// `"5.0"` — FL+FR+FC+BL+BR (FFmpeg `AV_CH_LAYOUT_5POINT0_BACK`).
  ///
  /// The unqualified `"5.0"` is FFmpeg's name for the back-speaker
  /// layout; the side-speaker one is [`Self::N5Point0`], spelled
  /// `"5.0(side)"`.
  N5Point0Back,
  /// `"5.1(side)"` — FL+FR+FC+LFE+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1`).
  ///
  /// FFmpeg's plain `"5.1"` is the **back** layout — see
  /// [`Self::N5Point1Back`].
  N5Point1,
  /// `"5.1"` — FL+FR+FC+LFE+BL+BR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1_BACK`).
  ///
  /// This is what an FFmpeg-sourced `"5.1"` means — the historically
  /// unqualified spelling belongs to the back-speaker layout, and the
  /// side-speaker one is [`Self::N5Point1`], spelled `"5.1(side)"`.
  N5Point1Back,
  /// `"5.1.2(back)"` — FL+FR+FC+LFE+BL+BR+TFL+TFR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1POINT2_BACK`).
  ///
  /// Here the qualifier runs the opposite way to the 5.1 family: the
  /// unqualified `"5.1.2"` is the *side* layout
  /// (`AV_CH_LAYOUT_5POINT1POINT2`), which this vocabulary does not
  /// enumerate — it rides [`Self::Other`]. FFmpeg's
  /// `AV_CH_LAYOUT_7POINT1_TOP_BACK` is a deprecated alias of this
  /// same layout, not a separate one.
  N5Point1Point2Back,
  /// `"5.1.4"` — FL+FR+FC+LFE+SL+SR+TFL+TFR+TBL+TBR (FFmpeg
  /// `AV_CH_LAYOUT_5POINT1POINT4_BACK`).
  ///
  /// The constant's `_BACK` marks the top-*back* height pair, not the
  /// surrounds — those are side (SL/SR) — so the slug carries no
  /// qualifier.
  N5Point1Point4Back,
  /// `"6.0"` — FL+FR+FC+BC+SL+SR (FFmpeg `AV_CH_LAYOUT_6POINT0`).
  N6Point0,
  /// `"6.0(front)"` — FL+FR+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_6POINT0_FRONT`).
  ///
  /// No front centre: the centre pair is FLC/FRC, off-centre.
  N6Point0Front,
  /// `"6.1"` — FL+FR+FC+LFE+BC+SL+SR (FFmpeg `AV_CH_LAYOUT_6POINT1`).
  N6Point1,
  /// `"6.1(back)"` — FL+FR+FC+LFE+BL+BR+BC (FFmpeg
  /// `AV_CH_LAYOUT_6POINT1_BACK`).
  N6Point1Back,
  /// `"6.1(front)"` — FL+FR+LFE+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_6POINT1_FRONT`).
  ///
  /// No front centre, as with [`Self::N6Point0Front`].
  N6Point1Front,
  /// `"7.0"` — FL+FR+FC+BL+BR+SL+SR (FFmpeg `AV_CH_LAYOUT_7POINT0`).
  N7Point0,
  /// `"7.0(front)"` — FL+FR+FC+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT0_FRONT`).
  N7Point0Front,
  /// `"7.1"` — FL+FR+FC+LFE+BL+BR+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1`).
  N7Point1,
  /// `"7.1(wide-side)"` — FL+FR+FC+LFE+FLC+FRC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1_WIDE`).
  ///
  /// FFmpeg's plain `"7.1(wide)"` is the **back** layout, not this one —
  /// see [`Self::N7Point1WideBack`]. The same crossing as the 5.x pairs,
  /// one family further up.
  N7Point1Wide,
  /// `"7.1(wide)"` — FL+FR+FC+LFE+BL+BR+FLC+FRC (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1_WIDE_BACK`).
  ///
  /// The unqualified `"7.1(wide)"` belongs to the back-speaker layout;
  /// the side-speaker one is [`Self::N7Point1Wide`], spelled
  /// `"7.1(wide-side)"`.
  N7Point1WideBack,
  /// `"7.1.2"` — FL+FR+FC+LFE+BL+BR+SL+SR+TFL+TFR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1POINT2`).
  N7Point1Point2,
  /// `"7.1.4"` — FL+FR+FC+LFE+BL+BR+SL+SR+TFL+TFR+TBL+TBR (FFmpeg
  /// `AV_CH_LAYOUT_7POINT1POINT4_BACK`).
  ///
  /// `_BACK` marks the top-back height pair, not the surrounds — see
  /// [`Self::N5Point1Point4Back`].
  N7Point1Point4Back,
  /// `"7.2.3"` — FL+FR+FC+LFE+BL+BR+SL+SR+TFL+TFR+TBC+LFE2 (FFmpeg
  /// `AV_CH_LAYOUT_7POINT2POINT3`).
  ///
  /// The `.2` is two LFE channels (LFE and LFE2); the `.3` is three
  /// height channels (TFL, TFR, TBC).
  N7Point2Point3,
  /// `"9.1.4"` — FL+FR+FC+LFE+BL+BR+FLC+FRC+SL+SR+TFL+TFR+TBL+TBR
  /// (FFmpeg `AV_CH_LAYOUT_9POINT1POINT4_BACK`).
  ///
  /// `_BACK` marks the top-back height pair, not the surrounds — see
  /// [`Self::N5Point1Point4Back`].
  N9Point1Point4Back,
  /// `"22.2"` — the 24-channel NHK Super Hi-Vision arrangement:
  /// FL+FR+FC+LFE+BL+BR+FLC+FRC+BC+SL+SR+TC+TFL+TFC+TFR+TBL+TBC+TBR+
  /// LFE2+TSL+TSR+BFC+BFL+BFR (FFmpeg `AV_CH_LAYOUT_22POINT2`).
  N22Point2,
  /// `"hexagonal"` — FL+FR+FC+BL+BR+BC (FFmpeg
  /// `AV_CH_LAYOUT_HEXAGONAL`). Six channels in a hexagon, no LFE.
  Hexagonal,
  /// `"octagonal"` — FL+FR+FC+BL+BR+BC+SL+SR (FFmpeg
  /// `AV_CH_LAYOUT_OCTAGONAL`). Eight channels around, no LFE.
  Octagonal,
  /// `"hexadecagonal"` — FL+FR+FC+BL+BR+BC+SL+SR+TFL+TFC+TFR+TBL+TBC+
  /// TBR+WL+WR (FFmpeg `AV_CH_LAYOUT_HEXADECAGONAL`). Sixteen channels,
  /// no LFE.
  Hexadecagonal,
  /// `"cube"` — FL+FR+BL+BR+TFL+TFR+TBL+TBR (FFmpeg
  /// `AV_CH_LAYOUT_CUBE`). Eight channels on the corners of a cube, no
  /// centre and no LFE.
  Cube,
  /// First-order Ambisonic B-format (WXYZ, 4 channels): `"ambisonic1"`.
  Ambisonic1,
  /// Second-order Ambisonic (9 channels): `"ambisonic2"`.
  Ambisonic2,
  /// Third-order Ambisonic (16 channels): `"ambisonic3"`.
  Ambisonic3,
  /// A layout not enumerated above — carries the FFmpeg-canonical
  /// name verbatim (e.g. `"binaural"`, `"9.1.6"`, a custom layout
  /// description). Lossless escape.
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
      Self::StereoDownmix => "downmix",
      Self::N2Point1 => "2.1",
      Self::N3Point0 => "3.0",
      Self::N3Point0Back => "3.0(back)",
      Self::N3Point1 => "3.1",
      Self::N3Point1Point2 => "3.1.2",
      Self::N4Point0 => "4.0",
      Self::N4Point1 => "4.1",
      Self::Quad => "quad",
      Self::QuadSide => "quad(side)",
      // FFmpeg gives the unqualified name to the BACK layout and
      // qualifies the side one — see `channel_layout_map[]`. The idents
      // follow the constants, the slugs follow FFmpeg's names, so these
      // six read crossed and are exactly right.
      Self::N5Point0 => "5.0(side)",
      Self::N5Point0Back => "5.0",
      Self::N5Point1 => "5.1(side)",
      Self::N5Point1Back => "5.1",
      // ...and here the qualifier runs the other way: the unqualified
      // "5.1.2" is the SIDE layout, which this vocabulary does not name.
      Self::N5Point1Point2Back => "5.1.2(back)",
      // `_BACK` in these three constants marks the top-back height pair,
      // not the surrounds, and FFmpeg minted no unsuffixed sibling — so
      // no `(back)` reaches the slug.
      Self::N5Point1Point4Back => "5.1.4",
      Self::N6Point0 => "6.0",
      Self::N6Point0Front => "6.0(front)",
      Self::N6Point1 => "6.1",
      Self::N6Point1Back => "6.1(back)",
      Self::N6Point1Front => "6.1(front)",
      Self::N7Point0 => "7.0",
      Self::N7Point0Front => "7.0(front)",
      Self::N7Point1 => "7.1",
      Self::N7Point1Wide => "7.1(wide-side)",
      Self::N7Point1WideBack => "7.1(wide)",
      Self::N7Point1Point2 => "7.1.2",
      Self::N7Point1Point4Back => "7.1.4",
      Self::N7Point2Point3 => "7.2.3",
      Self::N9Point1Point4Back => "9.1.4",
      Self::N22Point2 => "22.2",
      Self::Hexagonal => "hexagonal",
      Self::Octagonal => "octagonal",
      Self::Hexadecagonal => "hexadecagonal",
      Self::Cube => "cube",
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
    Mono,
    Stereo,
    StereoDownmix,
    N2Point1,
    N3Point0,
    N3Point0Back,
    N3Point1,
    N3Point1Point2,
    N4Point0,
    N4Point1,
    Quad,
    QuadSide,
    N5Point0,
    N5Point0Back,
    N5Point1,
    N5Point1Back,
    N5Point1Point2Back,
    N5Point1Point4Back,
    N6Point0,
    N6Point0Front,
    N6Point1,
    N6Point1Back,
    N6Point1Front,
    N7Point0,
    N7Point0Front,
    N7Point1,
    N7Point1Wide,
    N7Point1WideBack,
    N7Point1Point2,
    N7Point1Point4Back,
    N7Point2Point3,
    N9Point1Point4Back,
    N22Point2,
    Hexagonal,
    Octagonal,
    Hexadecagonal,
    Cube,
    Ambisonic1,
    Ambisonic2,
    Ambisonic3
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
      b"downmix" => Self::StereoDownmix,
      b"2.1" => Self::N2Point1,
      b"3.0" => Self::N3Point0,
      b"3.0(back)" => Self::N3Point0Back,
      b"3.1" => Self::N3Point1,
      b"3.1.2" => Self::N3Point1Point2,
      b"4.0" => Self::N4Point0,
      b"4.1" => Self::N4Point1,
      b"quad" => Self::Quad,
      b"quad(side)" => Self::QuadSide,
      b"5.0(side)" => Self::N5Point0,
      b"5.0" => Self::N5Point0Back,
      b"5.1(side)" => Self::N5Point1,
      b"5.1" => Self::N5Point1Back,
      b"5.1.2(back)" => Self::N5Point1Point2Back,
      b"5.1.4" => Self::N5Point1Point4Back,
      b"6.0" => Self::N6Point0,
      b"6.0(front)" => Self::N6Point0Front,
      b"6.1" => Self::N6Point1,
      b"6.1(back)" => Self::N6Point1Back,
      b"6.1(front)" => Self::N6Point1Front,
      b"7.0" => Self::N7Point0,
      b"7.0(front)" => Self::N7Point0Front,
      b"7.1" => Self::N7Point1,
      b"7.1(wide-side)" => Self::N7Point1Wide,
      b"7.1(wide)" => Self::N7Point1WideBack,
      b"7.1.2" => Self::N7Point1Point2,
      b"7.1.4" => Self::N7Point1Point4Back,
      b"7.2.3" => Self::N7Point2Point3,
      b"9.1.4" => Self::N9Point1Point4Back,
      b"22.2" => Self::N22Point2,
      b"hexagonal" => Self::Hexagonal,
      b"octagonal" => Self::Octagonal,
      b"hexadecagonal" => Self::Hexadecagonal,
      b"cube" => Self::Cube,
      b"ambisonic1" => Self::Ambisonic1,
      b"ambisonic2" => Self::Ambisonic2,
      b"ambisonic3" => Self::Ambisonic3,
      _ => Self::other(s),
    })
  }
}

#[cfg(test)]
mod tests;
