use super::*;

#[test]
fn construct_and_access() {
  let s = ChannelSpec::new(2, 4);
  assert_eq!(s.index(), 2);
  assert_eq!(s.raw_id(), 4);
  assert_eq!(s.label(), "");
}

#[test]
fn default_is_all_zero_and_unlabelled() {
  let s = ChannelSpec::default();
  assert_eq!(s, ChannelSpec::new(0, 0));
  assert_eq!(s.index(), 0);
  assert_eq!(s.raw_id(), 0);
  assert_eq!(s.label(), "");
}

#[test]
fn builders_chain() {
  let s = ChannelSpec::default()
    .with_index(1)
    .with_raw_id(3)
    .with_label("FL");
  assert_eq!(s.index(), 1);
  assert_eq!(s.raw_id(), 3);
  assert_eq!(s.label(), "FL");
}

#[test]
fn setters_chain() {
  let mut s = ChannelSpec::default();
  s.set_index(7).set_raw_id(11).set_label("BC");
  assert_eq!(s.index(), 7);
  assert_eq!(s.raw_id(), 11);
  assert_eq!(s.label(), "BC");
}

/// The label is free text, not a vocabulary: it is stored exactly as
/// handed over, case included. Nothing here folds — the folding gate is
/// for slugs, and a channel label is a backend's own rendering.
#[test]
fn the_label_is_free_text_not_a_slug() {
  let s = ChannelSpec::new(0, 1).with_label("LFE2");
  assert_eq!(s.label(), "LFE2");
  assert_ne!(s, ChannelSpec::new(0, 1).with_label("lfe2"));
}
