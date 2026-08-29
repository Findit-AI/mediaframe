//! WHAT LANGUAGE IT IS IN: the identity a track's words carry, and the vendored registries that
//! identity is read against.
//!
//! The invariant here is a STANDARD — BCP 47, and the three ISO registries under it — and a
//! standard does not arrive one type at a time: a language, a script and a region are three subtags
//! of ONE grammar, and an identity is the three composed. What arrived is ONE invariant with four
//! names, not four inventions; making any of the three subtags a `&str` inside the composite would
//! put three folds somewhere a caller cannot see them.
//!
//! # Where it came from, and what it replaced
//!
//! The family arrived from `ingraph::primitives::lang` under that crate's #428 — whole: the four
//! types, the generated registry table, the generator behind it and every one of its tests, with no
//! behaviour changed. What did NOT come is the SCORING composite (`DetectedLanguage`, an identity
//! with a confidence beside it): identity is a vocabulary question and belongs here, scoring is a
//! retrieval question and stays where the retrieval framework is.
//!
//! It REPLACED a wrapper over `icu_locale_core`'s three subtag types, which validated a whole tag
//! and then discarded everything past the region — so `de-CH-1901` and `de-CH` were one value and a
//! muxer's `ger` was not German at all. Nothing here discards a subtag, and the tail is the seat
//! that says so.
//!
//! # ZERO language knowledge is written here
//!
//! Not a subtag, not a name, not a fold. Two authority files are vendored under `xtask/vendor/` and
//! `cargo xtask gen-lang` turns them into `registry::table`, which is checked in;
//! `cargo xtask check` regenerates it and refuses if the two disagree, beside the pixel-format,
//! colour and codec tables the same command already checks.
//!
//! The rule earns its keep on the case the scene actually presents. A container's language tag is
//! dirty in ways no small hand-written table anticipates — an mkv writes ISO 639-2/B `ger`, an mp4
//! writes 639-2/T `deu`, a muxer writes `iw` for Hebrew forty years after the registry renamed it
//! `he`, and plenty write `und` or nothing at all. Each of those is a row in a published table, and
//! the way to get all of them right is to read the table.
//!
//! # Two files, because one of them cannot answer the question
//!
//! The IANA language-subtag-registry is the BCP 47 authority and feeds everything but ONE column.
//! It cannot fold `ger` or `deu`, because it contains neither word: BCP 47 takes a language's
//! two-letter code where one exists and never registers the three-letter one beside it, so the
//! whole ISO 639-2 alpha-3 space for a major language is absent from it — and that is exactly the
//! space a container writes. The ISO 639-2 registrar's own table is vendored beside it and feeds
//! that column alone. See [`registry`], and the `xtask` crate, where the argument is made in full.
//!
//! # The four types, and which of them a track actually holds
//!
//! ```text
//!   Language  ScriptSubtag  Region      one validated subtag each
//!   LanguageId                    the three composed, plus a lossless tail — four seats
//! ```
//!
//! A track's DECLARED language is a [`LanguageId`](crate::lang::LanguageId), or an `Option<LanguageId>` where the file said
//! nothing.
//!
//! The three subtag types are rarely held on their own and are declared as types anyway, which is
//! the same decision `mediatime`'s halves get: each has a grammar, a canonical form and a registry
//! column behind it.
//!
//! # `ScriptSubtag`, and not `Script`
//!
//! The type asks *which script did this tag DECLARE* — a subtag read out of a registry, never off a
//! character. A `Script` in a media vocabulary is the other question, *which writing system is this
//! codepoint in*, asked of text rather than of metadata; the name is left free for it. Every FIELD
//! spelled `script` is untouched: a seat's name is the question it asks.
//!
//! # ONE fold per question, applied at the DOOR
//!
//! Every canonicalisation this house does happens once, when a value is constructed, and nowhere
//! else. That is what makes equality useful: two spellings of one identity become one value before
//! anything compares them.
//!
//! | question | folded by | reading a column of |
//! |---|---|---|
//! | is this the same language? | `alpha3`, then `Preferred-Value` | ISO 639-2, then BCP 47 |
//! | is this the same script? | ASCII case alone | — |
//! | is this the same region? | ASCII case, then `Preferred-Value` | BCP 47 |
//! | is this the same identity? | the three, then `Suppress-Script` | all of them |
//!
//! # The three subtag types are ONE shape, and the shape is where the family words live
//!
//! [`Language`](crate::lang::Language), [`ScriptSubtag`](crate::lang::ScriptSubtag) and [`Region`](crate::lang::Region) each hold a validated `Utf8Bytes` in its own
//! canonical case, and every one of them owes the same eight impls: two renderings, the parse in
//! its three standard spellings, the text out borrowed, and the text out owned in each of the two
//! carriers a caller crosses on. Not one of those eight differs between the three by anything but
//! the type's name, so they are emitted by one `subtag_common!` macro rather than written out three
//! times.
//!
//! What is NOT in the macro is everything a reader comes here to check: the structural grammar each
//! type admits, the case it folds onto, which registry columns it folds THROUGH, and the sentences
//! its door refuses with. Those are the type's subject and each one spells them out.
//!
//! # BOUNDED SEATS ARE INLINE AND `Copy`; the one unbounded seat is not
//!
//! BCP 47 bounds all three subtags — a language at eight ASCII letters, a script at four, a region
//! at two letters or three digits — so each is stored as a fixed byte buffer with a length and
//! nothing else. That makes [`Language`](crate::lang::Language), [`ScriptSubtag`](crate::lang::ScriptSubtag) and [`Region`](crate::lang::Region) `Copy`: a clone is a
//! register move, equality is a fixed-width comparison, and none of the three can allocate. It is
//! also why [`LanguageId`](crate::lang::LanguageId)'s three bounded accessors hand back VALUES rather than borrows.
//!
//! [`LanguageId`](crate::lang::LanguageId)'s tail is the exception and the reason the exception exists: variants, extensions
//! and the private-use sequence have no width the grammar bounds, so that one seat is a
//! `smol_bytes::Utf8Bytes` — the text seat the retrieval layer downstream addresses a row by, which
//! is what lets an identity be keyed there without a copy at the boundary. It is the household's
//! only heap-backed field, and therefore the only reason a `LanguageId` is `Clone` and not `Copy`.

mod id;
mod language;
mod region;
pub mod registry;
mod script;
mod subtag;

pub use id::{LanguageId, ParseLanguageIdError};
pub use language::{Language, ParseLanguageError};
pub use region::{ParseRegionError, Region};
pub use script::{ParseScriptSubtagError, ScriptSubtag};

/// Generate the rows that are the same row for all three subtag types, and the reason this house
/// has a macro at all.
///
/// | row | what it says |
/// |---|---|
/// | [`Display`](core::fmt::Display), [`Debug`](core::fmt::Debug) | the canonical subtag, and nothing around it |
/// | [`FromStr`](core::str::FromStr), [`TryFrom`] × 2 | the parse, forwarded to the type's own door — off a `&str` and off the text seat |
/// | [`AsRef<str>`] | the text out, borrowed |
/// | [`From`] × 2 | the text out, owned — as the seat the type holds, and as a `String` |
///
/// The `String` conversion is what gives `String: TryFrom<Language>` — and the same for the other
/// two — through std's blanket, with `Error = Infallible`: a canonical subtag is valid text by
/// construction.
macro_rules! subtag_common {
  ($subtag:ident, $error:ident) => {
    /// The canonical subtag, and nothing else — no quotes, no wrapper, no type name.
    impl ::core::fmt::Display for $subtag {
      #[inline]
      fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str(self.as_str())
      }
    }

    /// Written out rather than derived, so an assertion message reads `Language("de")` rather than
    /// the text seat's own three-field shape.
    impl ::core::fmt::Debug for $subtag {
      fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, concat!(stringify!($subtag), "({:?})"), self.as_str())
      }
    }

    impl ::core::str::FromStr for $subtag {
      type Err = $error;

      /// The type's own door — see its `new`, where the grammar and the folds are.
      #[inline]
      fn from_str(text: &str) -> ::core::result::Result<Self, Self::Err> {
        Self::new(text)
      }
    }

    /// The standard fallible conversion, which is the door above under the name a generic caller
    /// reaches for.
    impl ::core::convert::TryFrom<&str> for $subtag {
      type Error = $error;

      #[inline]
      fn try_from(text: &str) -> ::core::result::Result<Self, Self::Error> {
        Self::new(text)
      }
    }

    /// The same door, off the text seat this household carries — the inverse of the `From` row
    /// below, and fallible where that one is not.
    impl ::core::convert::TryFrom<::smol_bytes::Utf8Bytes> for $subtag {
      type Error = $error;

      #[inline]
      fn try_from(text: ::smol_bytes::Utf8Bytes) -> ::core::result::Result<Self, Self::Error> {
        Self::new(text.as_str())
      }
    }

    impl ::core::convert::AsRef<str> for $subtag {
      #[inline]
      fn as_ref(&self) -> &str {
        self.as_str()
      }
    }

    /// The text out, as the seat the UNBOUNDED half of this household carries.
    ///
    /// Built from the inline bytes rather than handed over: a subtag IS its bytes, so there is no
    /// inner seat to give away — which is the whole of what makes these three [`Copy`].
    impl ::core::convert::From<$subtag> for ::smol_bytes::Utf8Bytes {
      #[inline]
      fn from(subtag: $subtag) -> Self {
        Self::from(subtag.as_str())
      }
    }

    /// The text out, OWNED as ordinary text — which is also what gives the subtag a
    /// `String: TryFrom<_>` through std's blanket, and gives it with `Error = Infallible`: a
    /// canonical subtag is valid text by construction.
    impl ::core::convert::From<$subtag> for ::std::string::String {
      #[inline]
      fn from(subtag: $subtag) -> Self {
        Self::from(subtag.as_str())
      }
    }
  };
}

pub(crate) use subtag_common;

// Optional `serde` impls for the whole household, grouped in one gated `const` block: every one of
// the four types (de)serializes as its CANONICAL TEXT — `"de"`, `"Hans"`, `"MM"`, `"zh-Hant-TW"` —
// which is the same text `Display` writes and `FromStr` reads back.
//
// Written here rather than in `serde_impls`, which is where this household's predecessor kept its
// own bespoke BCP-47 impl, and for the same reason: `serde_impls` centralises the roster enums'
// slug crossing, and these four are not roster enums.
//
// The READ goes through the type's own door, which is what makes it CANONICALISE rather than merely
// validate: a document written before a registry bump, or by a writer that is not this crate,
// holding `"GER"` reads back as the `de` every other route produces. A text that names no subtag is
// a deserialize error rather than a value whose type says it cannot exist.
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
const _: () = {
  use serde::{Deserialize, Deserializer, Serialize, Serializer};

  macro_rules! serde_via_text {
    ($ty:ident, $expecting:literal) => {
      impl Serialize for $ty {
        fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
          ser.collect_str(self)
        }
      }

      impl<'de> Deserialize<'de> for $ty {
        fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
          struct V;

          impl serde::de::Visitor<'_> for V {
            type Value = $ty;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
              f.write_str($expecting)
            }

            fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<$ty, E> {
              $ty::new(text).map_err(serde::de::Error::custom)
            }
          }

          de.deserialize_str(V)
        }
      }
    };
  }

  serde_via_text!(Language, "a BCP 47 primary language subtag");
  serde_via_text!(ScriptSubtag, "an ISO 15924 script subtag");
  serde_via_text!(Region, "an ISO 3166-1 or UN M.49 region subtag");
  serde_via_text!(LanguageId, "a BCP 47 language tag");
};

#[cfg(test)]
mod tests;
