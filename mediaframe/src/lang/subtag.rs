//! THE SEAT ALL THREE SUBTAG TYPES ARE: a fixed ASCII buffer, its length, and the folds that fill
//! it — plus the two structural questions every door asks before it reaches a registry.
//!
//! None of this is language knowledge, which is why it is written here rather than generated: a
//! subtag's shape is BCP 47's grammar — so many letters, or so many digits — and its case is a
//! spelling convention. What each type admits, and which case it folds onto, is the type's own
//! subject and is stated there.
//!
//! # A SUBTAG IS BOUNDED, so it is stored INLINE and it is [`Copy`]
//!
//! BCP 47 bounds every one of the three: a language is `2*3ALPHA / 4ALPHA / 5*8ALPHA`, a script is
//! `4ALPHA` exactly, and a region is `2ALPHA / 3DIGIT`. Eight bytes is the widest thing any of them
//! can be, and none of them can be empty. So there is no heap here and no pointer: a value IS its
//! bytes, which makes all three `Copy`, makes a clone a register move, and makes equality a
//! fixed-width comparison rather than a dereference.
//!
//! That is the whole difference from a text seat like `smol_bytes::Utf8Bytes`, which exists for
//! text whose length is *not* bounded — and the one seat in this household that still needs one is
//! [`LanguageId`](super::LanguageId)'s tail, which holds every variant, extension and private-use
//! subtag a tag carries and has no width the grammar bounds.
//!
//! # The PADDING is what keeps the derived order the text's order
//!
//! [`Ascii`] derives [`Ord`], so it compares the whole buffer and then the length. Unused bytes are
//! zero and every byte a subtag can hold is an ASCII letter or digit (`0x30` at the lowest), so a
//! shorter subtag's first unused byte sorts BELOW any byte a longer one could have there — which is
//! exactly what `str`'s own comparison does when one operand runs out. `en` sorts before `eng` on
//! both readings, and two subtags that differ before either ends are decided by the same first
//! differing byte.
//!
//! The length can therefore never be the tiebreak, since two equal padded buffers hold the same
//! bytes and no subtag contains a `NUL`. It is compared anyway, because deriving is what keeps the
//! rule one line rather than a hand-written comparison to get wrong.

/// The widest subtag BCP 47's grammar admits, and therefore the width [`Language`] is stored at.
///
/// Eight, from `language = 2*3ALPHA / 4ALPHA / 5*8ALPHA`. Nothing longer can be a subtag, so a text
/// longer than this is refused on its LENGTH before any fold is attempted — which is what makes the
/// buffer a fixed one.
///
/// [`Language`]: super::Language
pub(super) const MAX: usize = 8;

/// One canonical subtag: `N` bytes of ASCII with a length, and nothing else.
///
/// The storage of all three subtag types, and — since a value is its bytes — the whole of what
/// makes them [`Copy`]. See the module docs for why the derived [`Ord`] is the text's order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Ascii<const N: usize> {
  /// The canonical bytes, zero-padded past [`len`](Self::len).
  buffer: [u8; N],
  /// How many of them are the subtag. Never zero: every door refuses an empty text first.
  len: u8,
}

impl<const N: usize> Ascii<N> {
  /// The text lower-cased — the case the registry spells a LANGUAGE subtag in.
  pub(super) fn lower(text: &str) -> Self {
    Self::mapped(text, |_, byte| byte.to_ascii_lowercase())
  }

  /// The text Titlecased — one capital, then lower, which is how ISO 15924 and the registry both
  /// spell a SCRIPT subtag.
  pub(super) fn title(text: &str) -> Self {
    Self::mapped(text, |at, byte| {
      if at == 0 {
        byte.to_ascii_uppercase()
      } else {
        byte.to_ascii_lowercase()
      }
    })
  }

  /// The text upper-cased — the case the registry spells a REGION's country code in.
  pub(super) fn upper(text: &str) -> Self {
    Self::mapped(text, |_, byte| byte.to_ascii_uppercase())
  }

  /// The text as it arrived, folded by nothing.
  ///
  /// The REGION's digit arm, and the one place a door canonicalises by doing nothing: a digit has
  /// no case, and the leading zero of `001` is part of the code rather than formatting.
  pub(super) fn verbatim(text: &str) -> Self {
    Self::mapped(text, |_, byte| byte)
  }

  /// A subtag known at compile time — what lets [`Language::UND`](super::Language::UND) and the
  /// script and region sentinels be associated CONSTANTS rather than functions.
  ///
  /// It folds nothing, because the callers spell their subtags in the canonical case already; a
  /// constant that needed folding would be a constant somebody had spelled wrong.
  ///
  /// # Panics
  ///
  /// At COMPILE time, if `text` is longer than `N` — a `const` item cannot be built out of a panic,
  /// so the wrong width is a build error at the constant rather than a value that exists.
  pub(super) const fn literal(text: &str) -> Self {
    let bytes = text.as_bytes();
    assert!(
      bytes.len() <= N,
      "a subtag constant is wider than the seat that holds it"
    );

    let mut buffer = [0u8; N];
    let mut at = 0;
    while at < bytes.len() {
      buffer[at] = bytes[at];
      at += 1;
    }

    Self {
      buffer,
      len: bytes.len() as u8,
    }
  }

  /// The folds' one body, `at` being the byte's index so Titlecase can tell the first from the rest.
  ///
  /// # Panics
  ///
  /// If `text` is longer than `N`. Every caller checks the width first — a subtag that long is
  /// outside the grammar and is refused with a sentence saying so — so reaching this is a bug in
  /// this house rather than bad input, and a panic is the right answer to it.
  fn mapped(text: &str, fold: impl Fn(usize, u8) -> u8) -> Self {
    assert!(
      text.len() <= N,
      "a subtag wider than its own seat reached the fold"
    );

    let mut buffer = [0u8; N];
    for (at, byte) in text.as_bytes().iter().enumerate() {
      buffer[at] = fold(at, *byte);
    }

    Self {
      buffer,
      len: text.len() as u8,
    }
  }

  /// The canonical text.
  ///
  /// UTF-8 validity here is PROVEN BY PROVENANCE rather than measured, which is what lets the
  /// conversion be unchecked — the same trade [`parse::fold`](crate::parse) states from the other
  /// side when it declines to derive a `str` at all: a second O(n) pass to prove a property the
  /// bytes already carry buys nothing.
  ///
  /// **The claim is CHECKED in every debug build**, so the `SAFETY` note below is a machine-tested
  /// assertion rather than prose: `simdutf8`'s validator runs under `debug_assert!`, which is
  /// exactly the configuration this crate's `miri` and sanitizer lanes run their test suites in,
  /// and the household's own suite walks all 8275 registered languages through this row.
  pub(super) fn as_str(&self) -> &str {
    let held = &self.buffer[..self.len as usize];

    debug_assert!(
      simdutf8::compat::from_utf8(held).is_ok(),
      "a canonical subtag's seat must hold valid UTF-8"
    );

    // SAFETY: every byte here was written by `mapped` or `literal`, and both copy from a `&str` —
    // already valid UTF-8. `mapped`'s callers refuse anything that is not ASCII alphanumeric before
    // folding (`non_alphabetic` / `non_alphanumeric`, and the two-grammar test in `Region::new`),
    // and an ASCII byte is a one-byte UTF-8 sequence that `to_ascii_lowercase` /
    // `to_ascii_uppercase` map to another ASCII byte; `literal`'s callers spell canonical ASCII
    // subtags. So the prefix holds exactly `self.len` one-byte sequences. `self.len` is that
    // `&str`'s own length, which both constructors assert is at most `N`, so the slice is in bounds
    // and ends on a character boundary.
    unsafe { core::str::from_utf8_unchecked(held) }
  }
}

/// The first character of `text` that is not an ASCII letter, or [`None`] where every one is.
///
/// It answers with the CHARACTER rather than a `bool` so a refusal can name what it found: `zh_CN`
/// is refused for its underscore and `dé` for its `é`, and a client told only that its input was
/// malformed has to guess which.
pub(super) fn non_alphabetic(text: &str) -> Option<char> {
  text
    .chars()
    .find(|character| !character.is_ascii_alphabetic())
}

/// The first character of `text` that is neither an ASCII letter nor an ASCII digit, or [`None`]
/// where every one is.
///
/// A region has TWO grammars, so its door cannot ask a single-class question: what it needs to tell
/// a client is whether the input left the alphanumeric alphabet altogether — `zh_CN`'s underscore —
/// or stayed inside it and belonged to neither arm, which is `D1`.
pub(super) fn non_alphanumeric(text: &str) -> Option<char> {
  text
    .chars()
    .find(|character| !character.is_ascii_alphanumeric())
}
