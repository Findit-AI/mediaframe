use super::{FOLD_CAP, fold};

#[test]
fn fold_lowercases_ascii_and_leaves_the_rest_alone() {
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("BT709", &mut buf), Some("bt709"));
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(
    fold("Chroma-Derived-NC", &mut buf),
    Some("chroma-derived-nc")
  );
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("yuv420p", &mut buf), Some("yuv420p"));
  // Non-ASCII passes through untouched — no locale-dependent mapping.
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("İ", &mut buf), Some("İ"));
}

#[test]
fn an_input_longer_than_any_slug_is_a_miss_not_a_panic() {
  let mut buf = [0u8; FOLD_CAP];
  let long = core::str::from_utf8(&[b'x'; FOLD_CAP + 1]).unwrap();
  assert_eq!(fold(long, &mut buf), None);
}
