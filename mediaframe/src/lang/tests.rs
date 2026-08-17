use super::*;
use core::str::FromStr;

// Compile-time check that `Language` is `Copy`. (If the icu_locale_core
// upgrade ever breaks this, the build fails here rather than at a
// distant call site that relied on it.)
const fn _is_copy<T: Copy>() {}
const _: () = _is_copy::<Language>();

#[test]
fn default_is_und() {
  let l = Language::default();
  assert_eq!(l.language(), "und");
  assert!(l.script().is_none());
  assert!(l.region().is_none());
  assert!(l.is_undetermined());
}

#[test]
fn from_bcp47_lang_only() {
  let l = Language::from_bcp47("en").unwrap();
  assert_eq!(l.language(), "en");
  assert!(l.script().is_none());
  assert!(l.region().is_none());
  assert!(!l.is_undetermined());
  assert_eq!(l.to_bcp47(), "en");
}

#[test]
fn from_bcp47_lang_region() {
  let l = Language::from_bcp47("en-US").unwrap();
  assert_eq!(l.language(), "en");
  assert_eq!(l.region(), Some("US"));
  assert!(l.script().is_none());
  assert_eq!(l.to_bcp47(), "en-US");
}

#[test]
fn from_bcp47_lang_script_region() {
  let l = Language::from_bcp47("zh-Hant-TW").unwrap();
  assert_eq!(l.language(), "zh");
  assert_eq!(l.script(), Some("Hant"));
  assert_eq!(l.region(), Some("TW"));
  assert_eq!(l.to_bcp47(), "zh-Hant-TW");
}

#[test]
fn from_bcp47_und() {
  let l = Language::from_bcp47("und").unwrap();
  assert!(l.is_undetermined());
  assert_eq!(l.to_bcp47(), "und");
}

#[test]
fn from_bcp47_rejects_bogus() {
  let err = Language::from_bcp47("xx-yy-zz-bogus").unwrap_err();
  assert!(matches!(err, LanguageError::MalformedBcp47(_)));
}

#[test]
fn try_new_components() {
  let l = Language::try_new("en", None, Some("US")).unwrap();
  assert_eq!(l.language(), "en");
  assert_eq!(l.region(), Some("US"));

  let l = Language::try_new("zh", Some("Hant"), Some("TW")).unwrap();
  assert_eq!(l.script(), Some("Hant"));
  assert_eq!(l.region(), Some("TW"));
}

#[test]
fn try_new_rejects_each_subtag() {
  assert!(matches!(
    Language::try_new("!!", None, None),
    Err(LanguageError::InvalidLanguage(_))
  ));
  assert!(matches!(
    Language::try_new("en", Some("###"), None),
    Err(LanguageError::InvalidScript(_))
  ));
  assert!(matches!(
    Language::try_new("en", None, Some("###")),
    Err(LanguageError::InvalidRegion(_))
  ));
}

#[test]
fn from_str_smoke() {
  let l: Language = "en-US".parse().unwrap();
  assert_eq!(l.language(), "en");
  assert_eq!(l.region(), Some("US"));
}

#[test]
fn display_round_trip() {
  let l = Language::from_bcp47("zh-Hant-TW").unwrap();
  let rendered = std::format!("{}", l);
  assert_eq!(rendered, "zh-Hant-TW");
  let parsed = Language::from_str(&rendered).unwrap();
  assert_eq!(parsed, l);
}
