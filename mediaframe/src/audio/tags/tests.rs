use super::*;

fn en_us() -> LanguageId {
  LanguageId::new("en-US").expect("a language tag")
}

#[test]
fn default_is_all_absent() {
  let t = Tags::default();
  assert_eq!(t.title(), "");
  assert_eq!(t.artist(), "");
  assert_eq!(t.album_artist(), "");
  assert_eq!(t.album(), "");
  assert_eq!(t.composer(), "");
  assert_eq!(t.genre(), "");
  assert_eq!(t.comment(), "");
  assert_eq!(t.year(), 0);
  assert_eq!(t.track_number(), 0);
  assert_eq!(t.track_total(), 0);
  assert_eq!(t.disc_number(), 0);
  assert_eq!(t.disc_total(), 0);
  assert_eq!(t.language(), None);
}

#[test]
fn new_matches_default() {
  assert_eq!(Tags::new(), Tags::default());
}

#[test]
fn with_builders_roundtrip_every_field() {
  let t = Tags::new()
    .with_title("My Track")
    .with_artist("Artist X")
    .with_album_artist("Various Artists")
    .with_album("Best Album")
    .with_composer("Composer Y")
    .with_genre("Electronic")
    .with_comment("ripped 2026")
    .with_year(2026)
    .with_track_number(3)
    .with_track_total(12)
    .with_disc_number(1)
    .with_disc_total(2)
    .with_language(en_us());
  assert_eq!(t.title(), "My Track");
  assert_eq!(t.artist(), "Artist X");
  assert_eq!(t.album_artist(), "Various Artists");
  assert_eq!(t.album(), "Best Album");
  assert_eq!(t.composer(), "Composer Y");
  assert_eq!(t.genre(), "Electronic");
  assert_eq!(t.comment(), "ripped 2026");
  assert_eq!(t.year(), 2026);
  assert_eq!(t.track_number(), 3);
  assert_eq!(t.track_total(), 12);
  assert_eq!(t.disc_number(), 1);
  assert_eq!(t.disc_total(), 2);
  assert_eq!(t.language(), Some(&en_us()));
}

#[test]
fn setters_mutate_in_place() {
  let mut t = Tags::new();
  t.set_title("Foo").set_artist("Bar").set_year(1999);
  assert_eq!(t.title(), "Foo");
  assert_eq!(t.artist(), "Bar");
  assert_eq!(t.year(), 1999);
}

#[test]
fn numeric_zero_is_the_absent_sentinel() {
  // `0` is "absent" — setting then zeroing is equivalent to never set.
  let mut t = Tags::new().with_year(2026).with_track_number(3);
  assert_eq!(t.year(), 2026);
  t.set_year(0).set_track_number(0);
  assert_eq!(t.year(), 0);
  assert_eq!(t.track_number(), 0);
  assert_eq!(t, Tags::new());
}

#[test]
fn language_vocabulary_covers_set_update_clear() {
  let mut t = Tags::new();
  t.set_language(en_us());
  assert_eq!(t.language(), Some(&en_us()));
  let fr = LanguageId::new("fr-FR").expect("a language tag");
  t.update_language(Some(fr.clone()));
  assert_eq!(t.language(), Some(&fr));
  t.clear_language();
  assert_eq!(t.language(), None);
  let t = Tags::new().with_language(en_us()).maybe_language(None);
  assert_eq!(t.language(), None);
}
