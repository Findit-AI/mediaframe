// Cluster A — open string enums w/ `Other(SmolStr)` and total `FromStr`.
//
// Every type covered here has:
//   - an `Other(SmolStr)` lossless-escape arm, and
//   - an `impl FromStr` whose `Err = core::convert::Infallible`,
// so the shared `arb_open_string_enum!` macro applies directly. The 50/50
// branch in the macro flips between a curated slug (round-tripped through
// `FromStr` to exercise the named arms) and `Other(SmolStr::from(<arbitrary
// String>))` (exercises the lossless escape — including empty strings,
// pre-known slugs, and arbitrary bytes — for fuzz coverage).
//
// Slug picks: ~6 canonical FFmpeg / file-extension slugs per type, drawn
// from each type's own `as_str()` match. Common-case picks (not edge
// cases) — the goal is "this is a real value a real file would carry",
// since the `Other` branch already covers everything else.

super::arb_open_string_enum!(
  crate::codec::VideoCodec,
  ["h264", "hevc", "av1", "vp9", "mpeg4", "prores"]
);

super::arb_open_string_enum!(
  crate::codec::AudioCodec,
  ["aac", "mp3", "opus", "flac", "ac3", "alac"]
);

super::arb_open_string_enum!(
  crate::codec::SubtitleCodec,
  ["srt", "ass", "ssa", "webvtt", "mov_text", "dvb_subtitle"]
);

super::arb_open_string_enum!(
  crate::container::Format,
  ["mp4", "mkv", "webm", "mov", "avi", "mpegts"]
);

super::arb_open_string_enum!(
  crate::subtitle::Format,
  ["srt", "webvtt", "ass", "ssa", "mov_text", "ttml"]
);

// `TrackOrigin`'s `FromStr::Err` is `ParseTrackOriginError`, not
// `Infallible` — but this module is alloc-gated, and at that tier the
// parse is total (every miss rides `Other`), so the macro's `unwrap` is
// unreachable. See the note on `ParseTrackOriginError`.
super::arb_open_string_enum!(
  crate::subtitle::TrackOrigin,
  ["embedded", "sidecar", "external", "derived"]
);

// Both spellings of the 5.x pair are seeded on purpose — see the twin
// list in `quickcheck_helpers::strings`. The unqualified `"5.0"` /
// `"5.1"` are FFmpeg's names for the **back** layouts, so the short pair
// alone leaves `N5Point0` / `N5Point1` unreachable.
super::arb_open_string_enum!(
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

super::arb_open_string_enum!(
  crate::audio::ContainerFormat,
  ["mp3", "aac", "flac", "wav", "m4a", "opus"]
);

super::arb_open_string_enum!(
  crate::audio::SampleFormat,
  [
    "u8", "s16", "s32", "flt", "dbl", "u8p", "s16p", "s32p", "fltp", "dblp", "s64", "s64p"
  ]
);
