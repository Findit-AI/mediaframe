use super::{FOLD_CAP, fold};

#[test]
fn fold_lowercases_ascii_and_leaves_the_rest_alone() {
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("BT709", &mut buf), Some(&b"bt709"[..]));
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(
    fold("Chroma-Derived-NC", &mut buf),
    Some(&b"chroma-derived-nc"[..])
  );
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("yuv420p", &mut buf), Some(&b"yuv420p"[..]));
  // Non-ASCII passes through untouched — no locale-dependent mapping.
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("İ", &mut buf), Some("İ".as_bytes()));
}

/// The fold hands back the folded bytes themselves — the key the
/// `b"slug"` tables compare against — so the slice is the buffer prefix
/// of exactly the input's length, and an input that *fills* the buffer
/// is still a hit (only one byte more is the miss).
#[test]
fn fold_returns_the_folded_bytes_up_to_capacity() {
  let brim = core::str::from_utf8(&[b'X'; FOLD_CAP]).unwrap();
  let mut buf = [0u8; FOLD_CAP];
  let folded = fold(brim, &mut buf).expect("an input that exactly fills the buffer folds");
  assert_eq!(folded.len(), FOLD_CAP);
  assert!(folded.iter().all(|b| *b == b'x'));

  // The empty slug is an empty key, not a miss — a lookup, not a length
  // check, is what rejects it.
  let mut buf = [0u8; FOLD_CAP];
  assert_eq!(fold("", &mut buf), Some(&b""[..]));
}

#[test]
fn an_input_longer_than_any_slug_is_a_miss_not_a_panic() {
  let mut buf = [0u8; FOLD_CAP];
  let long = core::str::from_utf8(&[b'x'; FOLD_CAP + 1]).unwrap();
  assert_eq!(fold(long, &mut buf), None);
}
