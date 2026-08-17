use super::*;

#[test]
fn new_is_empty_and_matches_default() {
  assert_eq!(TrackDisposition::new(), TrackDisposition::empty());
  assert_eq!(TrackDisposition::default(), TrackDisposition::new());
  assert_eq!(TrackDisposition::new().bits(), 0);
}

#[test]
fn bit_values_match_ffmpeg_avformat_h() {
  // Spot-check the canonical FFmpeg constants against the
  // values declared in this file.
  assert_eq!(TrackDisposition::DEFAULT.bits(), 0x0000_0001);
  assert_eq!(TrackDisposition::DUB.bits(), 0x0000_0002);
  assert_eq!(TrackDisposition::ORIGINAL.bits(), 0x0000_0004);
  assert_eq!(TrackDisposition::COMMENT.bits(), 0x0000_0008);
  assert_eq!(TrackDisposition::LYRICS.bits(), 0x0000_0010);
  assert_eq!(TrackDisposition::KARAOKE.bits(), 0x0000_0020);
  assert_eq!(TrackDisposition::FORCED.bits(), 0x0000_0040);
  assert_eq!(TrackDisposition::HEARING_IMPAIRED.bits(), 0x0000_0080);
  assert_eq!(TrackDisposition::VISUAL_IMPAIRED.bits(), 0x0000_0100);
  assert_eq!(TrackDisposition::CLEAN_EFFECTS.bits(), 0x0000_0200);
  assert_eq!(TrackDisposition::ATTACHED_PIC.bits(), 0x0000_0400);
  assert_eq!(TrackDisposition::TIMED_THUMBNAILS.bits(), 0x0000_0800);
  assert_eq!(TrackDisposition::NON_DIEGETIC.bits(), 0x0000_1000);
  assert_eq!(TrackDisposition::CAPTIONS.bits(), 0x0001_0000);
  assert_eq!(TrackDisposition::DESCRIPTIONS.bits(), 0x0002_0000);
  assert_eq!(TrackDisposition::METADATA.bits(), 0x0004_0000);
  assert_eq!(TrackDisposition::DEPENDENT.bits(), 0x0008_0000);
  assert_eq!(TrackDisposition::STILL_IMAGE.bits(), 0x0010_0000);
}

#[test]
fn round_trip_via_u32_for_known_combinations() {
  let cases = [
    TrackDisposition::empty(),
    TrackDisposition::DEFAULT,
    TrackDisposition::FORCED | TrackDisposition::HEARING_IMPAIRED,
    TrackDisposition::DEFAULT
      | TrackDisposition::DUB
      | TrackDisposition::COMMENT
      | TrackDisposition::CAPTIONS,
    TrackDisposition::all(),
  ];
  for c in cases {
    assert_eq!(TrackDisposition::from_u32(c.to_u32()), c);
  }
}

#[test]
fn unknown_bits_round_trip_losslessly() {
  // A bit FFmpeg might add in the future (e.g. 0x0400_0000) must
  // survive `to_u32` / `from_u32` even though no named constant
  // is declared for it — `from_bits_retain` semantics.
  let bits_with_future = TrackDisposition::DEFAULT.bits() | 0x0400_0000;
  let rt = TrackDisposition::from_u32(bits_with_future);
  assert_eq!(rt.to_u32(), bits_with_future);
  assert!(rt.contains(TrackDisposition::DEFAULT));
}

#[test]
fn from_bits_truncate_drops_unknown_bits() {
  // Distinct from `from_u32`: `from_bits_truncate` is the
  // bitflags-crate's masking constructor and DOES drop unknown
  // bits — included as a smoke-test of the underlying API.
  let bits_with_future = TrackDisposition::DEFAULT.bits() | 0x0400_0000;
  let truncated = TrackDisposition::from_bits_truncate(bits_with_future);
  assert_eq!(truncated, TrackDisposition::DEFAULT);
  assert_eq!(truncated.bits(), 0x0000_0001);
}

#[test]
fn contains_insert_remove_smoke() {
  let mut d = TrackDisposition::empty();
  assert!(!d.contains(TrackDisposition::DEFAULT));
  d.insert(TrackDisposition::DEFAULT | TrackDisposition::FORCED);
  assert!(d.contains(TrackDisposition::DEFAULT));
  assert!(d.contains(TrackDisposition::FORCED));
  d.remove(TrackDisposition::DEFAULT);
  assert!(!d.contains(TrackDisposition::DEFAULT));
  assert!(d.contains(TrackDisposition::FORCED));
}
