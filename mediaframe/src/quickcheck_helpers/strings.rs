//! Cluster A — open string enums w/ `Other(SmolStr)` and total `FromStr`.
//!
//! One `pub(crate) fn name(g: &mut Gen) -> T` per type, referenced from each
//! type's container-level `#[quickcheck(arbitrary = "crate::quickcheck_helpers::strings::name")]`.
//!
//! Pattern: 50/50 picks a curated slug or an arbitrary string — **both
//! routed through `FromStr`**, so every generated value is canonical.
//!
//! Owned types:
//!   - codec::{VideoCodec, AudioCodec, SubtitleCodec}
//!   - container::Format
//!   - subtitle::Format
//!   - audio::ChannelLayout, audio::SampleFormat, audio::ContainerFormat

qc_open_string_enum!(
  video_codec,
  crate::codec::VideoCodec,
  ["h264", "hevc", "av1", "vp9", "mpeg4", "prores"]
);

qc_open_string_enum!(
  audio_codec,
  crate::codec::AudioCodec,
  ["aac", "mp3", "opus", "flac", "ac3", "alac"]
);

qc_open_string_enum!(
  subtitle_codec,
  crate::codec::SubtitleCodec,
  ["srt", "ass", "ssa", "webvtt", "mov_text", "dvb_subtitle"]
);

qc_open_string_enum!(
  container_format,
  crate::container::Format,
  ["mp4", "mkv", "webm", "mov", "avi", "mpegts"]
);

qc_open_string_enum!(
  subtitle_format,
  crate::subtitle::Format,
  ["srt", "webvtt", "ass", "ssa", "mov_text", "ttml"]
);

// See the twin note in `arbitrary_impls::strings`: `TrackOrigin` parses
// totally at this (alloc-gated) tier despite naming a refusal type.
qc_open_string_enum!(
  track_origin,
  crate::subtitle::TrackOrigin,
  ["embedded", "sidecar", "external", "derived"]
);

// Both spellings of the 5.x pair are seeded on purpose. FFmpeg gives the
// unqualified `"5.0"` / `"5.1"` to the **back**-speaker layouts and
// qualifies the side ones, so seeding only the short pair reaches
// `N5Point0Back` / `N5Point1Back` and leaves `N5Point0` / `N5Point1`
// unreachable — exactly the half whose slug moved in 0.4.0.
qc_open_string_enum!(
  channel_layout,
  crate::audio::ChannelLayout,
  [
    "mono",
    "stereo",
    "5.1",
    "5.1(side)",
    "7.1",
    "quad",
    "5.0",
    "5.0(side)"
  ]
);

qc_open_string_enum!(
  sample_format,
  crate::audio::SampleFormat,
  [
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p"
  ]
);

qc_open_string_enum!(
  audio_container_format,
  crate::audio::ContainerFormat,
  ["mp3", "aac", "flac", "wav", "m4a", "opus"]
);
