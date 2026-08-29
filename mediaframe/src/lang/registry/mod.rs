//! The vendored registries, as questions rather than arrays: is this subtag registered, what is it
//! called, what does it fold onto, what script does it imply.
//!
//! `table` is generated — 8275 languages, 224 scripts, 303 regions and four fold tables, every
//! word of them read out of a file vendored under `xtask/vendor/`. This module is the half that is
//! written, and it holds NO language knowledge at all: what is here is binary search, a range test
//! and the names of the columns.
//!
//! That split is the point. `cargo xtask check` regenerates `table` and refuses if the checked-in
//! file differs — the same command that checks this crate's pixel-format, colour and codec tables
//! against their own vendored files — so the vocabulary is a function of the vendored files and of
//! nothing else, and reviewing a registry bump is reading a diff of `("bh", "Bihari languages")`
//! rows rather than auditing hand-typed lists for a subtag someone got wrong.
//!
//! # It is PUBLIC, and what that is for
//!
//! [`Language`](super::Language) republishes the answers it needs — `name`, `is_registered`,
//! `is_deprecated` — so this module is not the route a caller normally takes. What it is for is the
//! question those methods cannot answer: **why** a value came out the way it did. A tag that
//! arrived as `ger-Latn-DE` and canonicalised to `de-DE` has taken two folds and a suppression, and
//! [`alpha3`], [`language_preferred`] and [`language_suppress_script`] are the three rows that
//! performed them. A fold with no way to read its own table is a canonicalisation a caller can only
//! disagree with.
//!
//! # Every lookup here takes the REGISTRY's own case, and none of them folds
//!
//! `language_name("DE")` is [`None`], and that is a floor rather than a gap. The registry spells a
//! language lower, a script Titlecase and a region UPPER, and each of the three types folds a wide
//! spelling onto that case ONCE, in its own constructor — so a value that exists is already in the
//! case its table is keyed by, and every later lookup is a direct hit. A second fold here would be
//! that work done twice, on a value that cannot need it.
//!
//! # The four folds, and which file each comes out of
//!
//! ```text
//!   ger, deu     ──alpha3──►          de     ISO 639-2's table — BCP 47 has neither word
//!   iw           ──language_preferred──► he    the registry's `Preferred-Value`
//!   BU           ──region_preferred──►  MM     the same column, on a region
//!   i-klingon    ──grandfathered──►     tlh    a whole TAG, which is `LanguageId`'s business
//! ```
//!
//! The first three fold a SUBTAG and are one hop each, which the generator refuses a registry
//! without. The fourth folds the whole tag — so the other three, and the `Suppress-Script` deletion
//! beside them, can hand it a tag it then folds again. That is why it is applied to a fixed point
//! rather than once, and why [`MAX_GRANDFATHERED_HOPS`] is a generated number.

pub(crate) mod table;

#[cfg(test)]
mod tests;

pub use table::FILE_DATE;

/// How many primary language subtags the vendored registry holds.
///
/// The reserved private-use RANGE is not one of them: it is a single record spelling `qaa..qtz`,
/// which names 512 subtags and registers none of them individually. [`LANGUAGE_PRIVATE_USE`] is
/// that record.
pub const LANGUAGE_COUNT: usize = table::LANGUAGES.len();

/// How many script subtags the vendored registry holds, the private-use range excluded.
pub const SCRIPT_COUNT: usize = table::SCRIPTS.len();

/// How many region subtags the vendored registry holds, the two private-use ranges excluded.
///
/// Two grammars are counted together, because the registry holds them in one list: ISO 3166-1
/// alpha-2 codes and UN M.49 three-digit area codes are both region subtags.
pub const REGION_COUNT: usize = table::REGIONS.len();

/// How many whole tags the registry grandfathered — those that name a replacement and those that do
/// not.
pub const GRANDFATHERED_COUNT: usize = table::GRANDFATHERED.len() + table::GRANDFATHERED_KEPT.len();

/// The bounds of the language range reserved for private use, inclusive at both ends.
///
/// Published because it is a fact about the REGISTRY that a reader may want to check a subtag
/// against directly; [`language_is_private_use`] is the same fact as a question.
pub const LANGUAGE_PRIVATE_USE: (&str, &str) = table::LANGUAGE_PRIVATE_USE;

/// The bounds of the script range reserved for private use, inclusive at both ends.
pub const SCRIPT_PRIVATE_USE: (&str, &str) = table::SCRIPT_PRIVATE_USE;

/// The bounds of the region ranges reserved for private use, inclusive at both ends.
///
/// TWO ranges, where a language and a script have one each — `QM`..`QZ` and `XA`..`XZ`.
pub const REGION_PRIVATE_USE: &[(&str, &str)] = table::REGION_PRIVATE_USE;

/// The region subtags reserved for private use INDIVIDUALLY, outside those ranges.
///
/// `AA` and `ZZ`. A region is the only subtag kind with any, so it is the only one whose
/// private-use question is not a range test alone — see [`region_is_private_use`]. The generator
/// refuses a registry in which a language or a script grows one, which is what keeps the asymmetry
/// checked rather than assumed.
pub const REGION_PRIVATE_USE_SUBTAGS: &[&str] = table::REGION_PRIVATE_USE_SUBTAGS;

// -----------------------------------------------------------------------------------------------
// Language
// -----------------------------------------------------------------------------------------------

/// The registry's first `Description` for a language subtag, or [`None`] where it registers none.
///
/// Answering [`Some`] IS registration — [`Language::is_registered`](super::Language::is_registered)
/// is this lookup asked for its emptiness — so there is one table behind both questions and no way
/// for a name and a membership to disagree.
///
/// The FIRST description where a subtag carries several: `zh` is *Chinese*, and `ro` is *Romanian*
/// where the registry also lists *Moldavian* and *Moldovan*. The first is the one the registry
/// leads with, and picking it here is what keeps `name` a function rather than a list a caller has
/// to choose from.
#[inline]
#[must_use]
pub fn language_name(subtag: &str) -> Option<&'static str> {
  paired(table::LANGUAGES, subtag)
}

/// The subtag the registry says to use INSTEAD of this one, or [`None`] where it names none.
///
/// The `Preferred-Value` column: `iw` prefers `he`, `in` prefers `id`, `mo` prefers `ro`. It is
/// ONE hop by construction — the generator refuses a table where a preferred value itself prefers
/// something else — so a fold applies this once and is done.
///
/// A deprecated subtag is not always here. 120 of the 232 deprecated languages name no replacement
/// at all, and those stay themselves: [`language_is_deprecated`] is the question that finds them.
#[inline]
#[must_use]
pub fn language_preferred(subtag: &str) -> Option<&'static str> {
  paired(table::LANGUAGE_PREFERRED, subtag)
}

/// The script this language implies, or [`None`] where it implies none.
///
/// The `Suppress-Script` column, and the whole of what makes `en-Latn` compose as `en` while
/// `zh-Hans` composes as itself: `en` implies `Latn` and `zh` implies nothing, Chinese being written
/// in more than one script.
#[inline]
#[must_use]
pub fn language_suppress_script(subtag: &str) -> Option<&'static str> {
  paired(table::LANGUAGE_SUPPRESS_SCRIPT, subtag)
}

/// Has the registry deprecated this language subtag?
///
/// Independent of [`language_preferred`]: a subtag can be deprecated with a replacement, deprecated
/// without one, or neither. What it is never is replaced without being deprecated.
#[inline]
#[must_use]
pub fn language_is_deprecated(subtag: &str) -> bool {
  listed(table::LANGUAGE_DEPRECATED, subtag)
}

/// Does this subtag fall in the range the registry reserves for private use?
///
/// `qaa` through `qtz` — 512 subtags the registry names as a block and registers not one of. So a
/// private-use subtag is structurally fine, carries no name, and is not registered, which is three
/// separate answers rather than one.
#[inline]
#[must_use]
pub fn language_is_private_use(subtag: &str) -> bool {
  within(table::LANGUAGE_PRIVATE_USE, subtag)
}

/// The shortest BCP 47 spelling of an ISO 639-2 code, or [`None`] where the code IS the shortest
/// spelling or is not a 639-2 code at all.
///
/// The one column the SECOND vendored file feeds, and the one BCP 47 cannot: `ger` and `deu` are
/// German's bibliographic and terminological alpha-3 codes, an mkv writes the first and an mp4 the
/// second, and the registry contains neither word because `de` exists. Both answer `de` here, in
/// one hop.
///
/// `haw` answers [`None`], and that is the same rule rather than a miss: Hawaiian has no two-letter
/// code, so `haw` is already the shortest spelling and the registry carries it.
#[inline]
#[must_use]
pub fn alpha3(code: &str) -> Option<&'static str> {
  paired(table::ALPHA3, code)
}

// -----------------------------------------------------------------------------------------------
// Script
// -----------------------------------------------------------------------------------------------

/// The registry's first `Description` for a script subtag, or [`None`] where it registers none.
#[inline]
#[must_use]
pub fn script_name(subtag: &str) -> Option<&'static str> {
  paired(table::SCRIPTS, subtag)
}

/// Does this subtag fall in the script range reserved for private use — `Qaaa` through `Qabx`?
#[inline]
#[must_use]
pub fn script_is_private_use(subtag: &str) -> bool {
  within(table::SCRIPT_PRIVATE_USE, subtag)
}

// -----------------------------------------------------------------------------------------------
// Region
// -----------------------------------------------------------------------------------------------

/// The registry's first `Description` for a region subtag, or [`None`] where it registers none.
#[inline]
#[must_use]
pub fn region_name(subtag: &str) -> Option<&'static str> {
  paired(table::REGIONS, subtag)
}

/// The region the registry says to use INSTEAD of this one, or [`None`] where it names none.
///
/// Six rows, and every one of them a state that was succeeded by exactly one other: `BU` prefers
/// `MM`, `ZR` prefers `CD`, `TP` prefers `TL`. One hop, for [`language_preferred`]'s reason and
/// under the same generated guard.
#[inline]
#[must_use]
pub fn region_preferred(subtag: &str) -> Option<&'static str> {
  paired(table::REGION_PREFERRED, subtag)
}

/// Has the registry deprecated this region subtag?
///
/// Eleven have been, and five of those name no successor — `AN`, `CS`, `NT`, `SU` and `YU`, each a
/// state that dissolved into SEVERAL, where there is no single region to fold onto. Those keep
/// their own spelling and answer `true` here, which is the only honest pair of answers available.
#[inline]
#[must_use]
pub fn region_is_deprecated(subtag: &str) -> bool {
  listed(table::REGION_DEPRECATED, subtag)
}

/// Is this subtag one the registry reserves for private use — by range, or by a record of its own?
///
/// FOUR spellings and only two of them ranges: `QM`..`QZ`, `XA`..`XZ`, and then `AA` and `ZZ`, each
/// an individually registered record carrying the description *Private use*. A range test alone
/// answers `false` to the two a container actually writes.
///
/// It is therefore the ONE predicate in this module that can be `true` at the same time as
/// [`region_name`] answering [`Some`]. That is the registry's shape, not a reading invented here.
#[inline]
#[must_use]
pub fn region_is_private_use(subtag: &str) -> bool {
  listed(table::REGION_PRIVATE_USE_SUBTAGS, subtag)
    || table::REGION_PRIVATE_USE
      .iter()
      .any(|range| within(*range, subtag))
}

// -----------------------------------------------------------------------------------------------
// Grandfathered whole tags
// -----------------------------------------------------------------------------------------------

/// The tag the registry says to use INSTEAD of this grandfathered one, or [`None`] where it names
/// none — including for every tag that is not grandfathered at all.
///
/// A whole TAG rather than a subtag, which is what makes this [`LanguageId`](super::LanguageId)'s
/// table and not [`Language`](super::Language)'s: `i-klingon` prefers `tlh` and `zh-guoyu` prefers
/// `cmn`, and neither `i-klingon` nor `zh-guoyu` is a subtag of anything.
///
/// LOWER-CASED on both sides: the caller folds the tag's case before asking, because case is not
/// part of a tag's identity and `I-KLINGON` names what `i-klingon` names.
#[inline]
#[must_use]
pub fn grandfathered_preferred(tag: &str) -> Option<&'static str> {
  paired(table::GRANDFATHERED, tag)
}

/// How many times the whole-tag fold can fire before a canonicalisation is a FIXED POINT.
///
/// **The one number here the generator PROVED rather than read off a column**, and the reason it
/// had to: [`grandfathered_preferred`] is applied to the whole tag, and the other folds REWRITE the
/// whole tag. `en-Latn-GB-oed` is not grandfathered as written — the suppression drops `Latn` and
/// leaves `en-GB-oed`, which is — so a canonicalisation that applied each fold once would render
/// text that reads back as a different identity, and the rendering is what serde and the wire codec
/// store. [`LanguageId::new`](super::LanguageId::new) therefore iterates to a fixed point, and this
/// is the bound it iterates to.
///
/// The bound is a property of the REGISTRY, so it is settled where the registry is read: the
/// generator walks every grandfathered tag through the composition rules, follows the chain each
/// fold's own output opens, and refuses a registry in which one cycles. A bump that lengthened a
/// chain fails `cargo xtask check`; it cannot reach production as a fold that quietly stops short.
pub const MAX_GRANDFATHERED_HOPS: usize = table::MAX_GRANDFATHERED_HOPS;

/// Is this whole tag one the registry grandfathered, whether or not it names a replacement?
///
/// Twenty-six are. Twenty-one name a replacement and are folded onto it; the five that do not are
/// [`grandfathered_preferred`]'s [`None`] and this predicate's `true` at the same time, which is the
/// distinction a single lookup could not carry.
#[inline]
#[must_use]
pub fn is_grandfathered(tag: &str) -> bool {
  paired(table::GRANDFATHERED, tag).is_some() || listed(table::GRANDFATHERED_KEPT, tag)
}

// -----------------------------------------------------------------------------------------------
// The three shapes every lookup above is one of
// -----------------------------------------------------------------------------------------------

/// The second half of the row whose first half is `key`, by binary search.
///
/// Every generated pair table is sorted on its first element — the generator builds them in a
/// [`BTreeMap`](std::collections::BTreeMap), so the order is a property of how they are emitted
/// rather than a claim about them — which is what makes this a search rather than a scan over eight
/// thousand rows.
fn paired(table: &'static [(&'static str, &'static str)], key: &str) -> Option<&'static str> {
  table
    .binary_search_by(|(candidate, _)| (*candidate).cmp(key))
    .ok()
    .map(|at| table[at].1)
}

/// Does this sorted table hold `key`?
fn listed(table: &'static [&'static str], key: &str) -> bool {
  table.binary_search(&key).is_ok()
}

/// Does `key` fall inside an inclusive range of same-width ASCII subtags?
///
/// The WIDTH test is what makes the comparison a range test rather than a lexicographic accident:
/// `qaa`..`qtz` is a block of three-letter subtags, and `qq` sorts between its bounds as text while
/// naming nothing inside it. Every subtag in a registry range has the width of the range's bounds,
/// so requiring it costs nothing and closes the case.
///
/// Byte comparison is the right relation here because both bounds and every subtag that can reach
/// this are ASCII: a structural check has already refused anything else, so no two subtags of equal
/// width compare by anything but their code points.
fn within((low, high): (&'static str, &'static str), key: &str) -> bool {
  key.len() == low.len() && low <= key && key <= high
}
