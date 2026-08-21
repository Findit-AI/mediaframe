use super::*;
use ::std::vec;

#[test]
fn default_is_empty() {
  let d = ChannelLayoutDescription::default();
  assert!(d.is_empty());
  assert_eq!(d.channels(), 0);
  assert_eq!(d.order(), ChannelOrder::Unspecified);
  assert_eq!(d.known_kind(), &ChannelLayout::default());
  assert!(d.native_mask().is_none());
  assert!(d.custom_channels().is_empty());
  assert_eq!(d.text(), "");
}

#[test]
fn new_with_channels_only() {
  let d = ChannelLayoutDescription::new(6);
  assert!(!d.is_empty()); // channels > 0
  assert_eq!(d.channels(), 6);
  assert_eq!(
    ChannelLayoutDescription::new(0),
    ChannelLayoutDescription::default()
  );
}

#[test]
fn builders_chain() {
  let d = ChannelLayoutDescription::new(6)
    .with_order(ChannelOrder::Native)
    .with_known_kind(ChannelLayout::Ch5_1Back)
    .with_native_mask(Some(0x3F))
    .with_text("5.1");
  assert_eq!(d.channels(), 6);
  assert_eq!(d.order(), ChannelOrder::Native);
  assert_eq!(d.known_kind(), &ChannelLayout::Ch5_1Back);
  assert_eq!(d.native_mask(), Some(0x3F));
  assert_eq!(d.text(), "5.1");
}

#[test]
fn setters_chain() {
  let mut d = ChannelLayoutDescription::default();
  d.set_channels(8)
    .set_order(ChannelOrder::Native)
    .set_native_mask(Some(0x63F));
  d.set_known_kind(ChannelLayout::Ch7_1).set_text("7.1");
  assert_eq!(d.channels(), 8);
  assert_eq!(d.order(), ChannelOrder::Native);
  assert_eq!(d.known_kind(), &ChannelLayout::Ch7_1);
  assert_eq!(d.native_mask(), Some(0x63F));
  assert_eq!(d.text(), "7.1");
}

#[test]
fn custom_channels_round_trip() {
  let custom = vec![
    ChannelSpec::new(0, 1).with_label("FL"),
    ChannelSpec::new(1, 2).with_label("FR"),
  ];
  let d = ChannelLayoutDescription::new(2)
    .with_order(ChannelOrder::Custom)
    .with_custom_channels(custom);
  assert_eq!(d.custom_channels().len(), 2);
  assert_eq!(d.custom_channels()[0].label(), "FL");
  assert_eq!(d.custom_channels()[1].label(), "FR");
}

/// The name seat holds `ChannelLayout`, whose "absent" value is the
/// `Other("")` sentinel rather than a named `Unknown` variant — that is
/// what `is_empty` tests, and what a layout no vocabulary can name
/// carries.
#[test]
fn an_unnamed_layout_carries_the_absent_sentinel() {
  let d = ChannelLayoutDescription::new(4)
    .with_order(ChannelOrder::Custom)
    .with_custom_channels(vec![ChannelSpec::new(0, 1)])
    .with_text("4 channels (FL+FR+BL+BR)");
  assert_eq!(d.known_kind(), &ChannelLayout::default());
  assert!(d.known_kind().is_other());
  // Naming it moves the field off the sentinel and nothing else.
  let named = d.clone().with_known_kind(ChannelLayout::Quad);
  assert_eq!(named.known_kind(), &ChannelLayout::Quad);
  assert_eq!(named.text(), d.text());
}

/// `text` is the backend's rendering, `known_kind` the parsed name.
/// They are independent seats: a description may carry either, both or
/// neither, and setting one never touches the other.
#[test]
fn the_name_and_the_rendering_are_separate_seats() {
  let only_name = ChannelLayoutDescription::new(6).with_known_kind(ChannelLayout::Ch5_1Back);
  assert_eq!(only_name.text(), "");
  assert_eq!(only_name.known_kind(), &ChannelLayout::Ch5_1Back);

  let only_text = ChannelLayoutDescription::new(6).with_text("5.1(side)");
  assert_eq!(only_text.known_kind(), &ChannelLayout::default());
  assert_eq!(only_text.text(), "5.1(side)");

  // And the rendering is verbatim — it is not folded, parsed or
  // canonicalised on the way in.
  let odd = ChannelLayoutDescription::new(3).with_text("3 channels (FL+FR+LFE)");
  assert_eq!(odd.text(), "3 channels (FL+FR+LFE)");
}

/// No invariant binds the fields: the incoherent combinations are
/// constructible, which is exactly why the fuzz generators are told to
/// reach them.
#[test]
fn incoherent_combinations_are_constructible() {
  let custom_without_channels = ChannelLayoutDescription::new(2).with_order(ChannelOrder::Custom);
  assert!(custom_without_channels.custom_channels().is_empty());

  let native_without_mask = ChannelLayoutDescription::new(6).with_order(ChannelOrder::Native);
  assert!(native_without_mask.native_mask().is_none());
}

/// A zero-channel description is not automatically empty — `is_empty`
/// is the conjunction over all six seats, not a channel-count test.
#[test]
fn is_empty_is_the_conjunction_over_every_seat() {
  assert!(ChannelLayoutDescription::new(0).is_empty());
  assert!(
    !ChannelLayoutDescription::new(0)
      .with_order(ChannelOrder::Native)
      .is_empty()
  );
  assert!(
    !ChannelLayoutDescription::new(0)
      .with_known_kind(ChannelLayout::Mono)
      .is_empty()
  );
  assert!(
    !ChannelLayoutDescription::new(0)
      .with_native_mask(Some(0))
      .is_empty()
  );
  assert!(
    !ChannelLayoutDescription::new(0)
      .with_custom_channels(vec![ChannelSpec::default()])
      .is_empty()
  );
  assert!(!ChannelLayoutDescription::new(0).with_text("x").is_empty());
}
