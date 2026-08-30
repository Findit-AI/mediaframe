//! The [`LanguageId`] composite: the four seats, and the composition rules that fill them.

use core::{fmt, str::FromStr};
use std::{string::ToString, vec::Vec};

use smol_bytes::Utf8Bytes;

use super::{
  Language, ParseLanguageError, ParseRegionError, ParseScriptSubtagError, Region, ScriptSubtag,
  registry,
};

#[cfg(test)]
mod tests;

/// A whole language identity — what UTS-35 calls a *unicode language id* and what a track's
/// metadata actually carries: `de`, `zh-Hans`, `en-US`, `sr-Cyrl-RS`, `en-US-x-lorem`.
///
/// FOUR seats, and the fourth is what makes the type lossless:
///
/// ```text
///   language   Language        always present — a tag with no language is not a tag
///   script     Option<ScriptSubtag>  absent where none was declared, or where the language
///                                implies it
///   region     Option<Region>  absent where none was declared
///   rest       Option<…>       everything past the region, VERBATIM
/// ```
///
/// # The composition rules, and every one of them is the registry's
///
/// | sent | held | the rule |
/// |---|---|---|
/// | `zh_Hans_CN` | `zh-Hans-CN` | `_` is read as `-`, which is what a filename-safe tag uses |
/// | `en-Latn` | `en` | `en` implies `Latn`, so declaring it says nothing — `Suppress-Script` |
/// | `zh-Hans` | `zh-Hans` | `zh` implies NO script, so the declared one is the whole content |
/// | `i-klingon` | `tlh` | a grandfathered tag, folded by its `Preferred-Value` |
/// | `GER-latn-de` | `de-DE` | each seat's own door, then the suppression |
/// | `en-Latn-Cyrl` | `en-Latn-Cyrl` | the suppression is skipped where it would not reparse |
///
/// **The `Suppress-Script` fold is the one that earns its keep**, and the pair above is why: it
/// removes a subtag that carries no information (`en-Latn` and `en` are the same identity) and
/// leaves alone one that carries all of it (`zh-Hans` and `zh-Hant` are two identities). A fold
/// spelled by hand would have had to decide which languages imply which scripts; this one reads a
/// column of a vendored file, so the answer is right for all 134 of them and stays right when the
/// registry moves.
///
/// **And it fires only where it is REVERSIBLE**, which is the last row and the one rule of this
/// table that is about the table itself: a fold here rewrites the TEXT, the text is the form serde
/// and the wire codec store, so a fold may only rewrite text that reads back as the same value. A
/// tail is held verbatim and its envelope is loose, so it can open with a subtag SHAPED like a
/// script — and dropping the script would leave that slot vacant for a reparse to fill from the
/// tail. `en-Latn-Cyrl` keeps its `Latn` for that reason and no other.
///
/// # `rest` is LOSSLESS, and that is the whole difference from what this type replaced
///
/// Everything past the region — variants (`en-US-posix`), extensions (`de-u-co-phonebk`), private
/// use (`en-x-lorem`) — is held VERBATIM, rendered back verbatim, and counts towards equality. It
/// is not parsed, not folded, and not case-normalised.
///
/// So `en-US-x-Foo` and `en-US-x-foo` are TWO values here, where RFC 5646's own canonical form would
/// lower-case both. That is the price of the seat being lossless rather than a defect in it: a
/// private-use sequence's meaning belongs to the private party that wrote it, and this type cannot
/// know that its case is insignificant. The tail is carried exactly as the file spelled it, and a
/// caller comparing tails as identifiers gets what the file said.
///
/// **The tail is not a seat anything asks a question ABOUT**, and that follows from the same fact.
/// There is nothing here that could be asked honestly of a text seat holding a mixture of three
/// grammars: what a caller wants to know about an identity is which language, script or region it
/// names, and each of those is a seat of its own. The tail rides along with them.
///
/// **The envelope the tail is checked against is deliberately loose**: each of its subtags must be
/// one to eight ASCII alphanumeric characters, which is the outer shape of every variant, extension
/// and private-use subtag together. `en-USA` is admitted, though `USA` is not a legal variant,
/// because a container writes what its muxer believed and a metadata layer's job is to carry it —
/// the same posture that admits an unregistered language subtag.
///
/// # An extlang has no seat, so it rides the tail — and takes the rest of the tag with it
///
/// `zh-yue` holds `zh` with a tail of `yue`, because `yue` is three letters and a script is four
/// while a region is two or three DIGITS. That is the roster doing what it says: the seats are
/// language, script and region, and BCP 47's extlang position is none of them.
///
/// The consequence is worth stating rather than discovering: the tail is a TAIL, so once a subtag
/// falls into it every later subtag does too. `zh-yue-Hant-HK` holds `zh` with a tail of
/// `yue-Hant-HK` — the script and region are IN the tail, and no seat accessor reaches them. The
/// tag round-trips exactly, and the LANGUAGE is still `zh`, which is the question most callers ask.
/// Reading its script off the seat is what is lost, and a `zh-Hant-HK` tag — which is what a
/// well-formed file writes — has that.
///
/// # Equality is the four seats, so it is the CANONICAL identity
///
/// `GER-latn-de` and `de-DE` are one value, because each seat folded before it was filled and the
/// suppression then removed the script. That is what makes a stored row and a later query meet:
/// four seats hold the canonical parts, and a value built from any spelling of the same identity
/// fills the same four.
///
/// It is EQUALITY of identities and not of languages, which the same pair says from the other side:
/// `de` and `de-DE` are two values, because one names a region and the other does not. Asking about
/// the language they share is [`language`](Self::language)'s business, not equality's.
///
/// # Ordering is the four seats in order, and it means what a text sort means
///
/// [`Ord`] is derived, so it compares the language first, then the script, then the region, then
/// the tail — each alphabetically, each absent sorting before present. It is a total, stable order,
/// and it is nothing else: `de-AT` sorts before `de-DE` because `A` sorts before `D`.
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::composite::language")
)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LanguageId {
  language: Language,
  script: Option<ScriptSubtag>,
  region: Option<Region>,
  rest: Option<Utf8Bytes>,
}

impl LanguageId {
  /// Read a whole tag, WIDE — any case, `_` or `-`, grandfathered or composed.
  ///
  /// The one door in, as each seat's own `new` is for its part.
  ///
  /// # The order the rules fire in
  ///
  /// 1. **`_` becomes `-`.** A filename-safe tag (`zh_Hans_CN`) is the same tag.
  /// 2. **The GRANDFATHERED table**, over the whole tag, case-folded. `i-klingon` is `tlh` and
  ///    `zh-guoyu` is `cmn`. Five grandfathered tags name no replacement, and those fall through to
  ///    the ordinary parse — where two of them (`cel-gaulish`, `zh-min`) turn out to be ordinary
  ///    compositions and the three beginning `i-` are refused, a one-letter primary subtag being
  ///    outside the grammar.
  /// 3. **The SEATS, positionally.** The first subtag is the language. A four-letter subtag next is
  ///    the script; a two-letter or three-digit one after that is the region; everything from the
  ///    first subtag that is neither is the tail.
  /// 4. **The SUPPRESSION.** A declared script the language already implies is dropped.
  /// 5. **AND THEN 2 TO 4 AGAIN, until the rendering STOPS MOVING** — see
  ///    [`canonicalized`](Self::canonicalized), which is where the reason lives. Rules 3 and 4
  ///    rewrite the whole tag, so their output can be a tag rule 2 folds; iterating is what makes
  ///    the answer independent of the order the rules happen to be written in.
  ///
  /// # Errors
  ///
  /// [`ParseLanguageIdError`], naming the seat that refused and carrying its own error — so a
  /// caller sending `e-US` is told what a language subtag is, by the type that knows.
  ///
  /// A seat is only ASKED about a subtag of its own shape, so the shape test and the seat's grammar
  /// cannot disagree: `zh-Hans-XYZ` never reaches the region door at all, `XYZ` being neither two
  /// letters nor three digits, and the tail's looser envelope takes it.
  pub fn new(text: &str) -> Result<Self, ParseLanguageIdError> {
    if text.is_empty() {
      return Err(ParseLanguageIdError::Empty);
    }

    // The separator fold happens FIRST and for the whole tag, so every rule below reads one
    // spelling. It is the only place this type rewrites its input outside a seat's own door.
    let normalized;
    let tag = if text.contains('_') {
      normalized = text.replace('_', "-");
      normalized.as_str()
    } else {
      text
    };

    // The whole-tag lookup runs on the ARRIVING text before anything else, and it has to: the three
    // tags beginning `i-` are grandfathered and are not compositions at all, so a road that reached
    // this table only through a composed value would refuse them. The loop below is the same fold
    // asked again of what the composition rules produced.
    let composed = match grandfathered(tag) {
      Some(preferred) => Self::composed(preferred)?,
      None => Self::composed(tag)?,
    };

    composed.canonicalized()
  }

  /// Apply the whole-tag fold to a FIXED POINT — the last rule, and the one that makes the others
  /// order-independent.
  ///
  /// # The class of defect this closes
  ///
  /// Every rule in this door REWRITES THE TAG, and the rewritten tag is what the next rule reads.
  /// The whole-tag table is keyed by text, so any rule whose output lands in its preimage moves the
  /// tag again — and a canonicalisation that stopped after one pass would render text that parses
  /// back as a DIFFERENT value:
  ///
  /// ```text
  ///   en-Latn-GB-oed   ──suppression──►  en-GB-oed  ──grandfathered──►  en-GB-oxendict
  ///   eng-GB-oed       ──alpha-3──────►  en-GB-oed  ──grandfathered──►  en-GB-oxendict
  ///   nor-bok          ──alpha-3──────►  no-bok     ──grandfathered──►  nb
  ///   zho-guoyu        ──alpha-3──────►  zh-guoyu   ──grandfathered──►  cmn
  /// ```
  ///
  /// None of those four is grandfathered AS WRITTEN. Each becomes one, and stopping short would
  /// leave a value whose rendering serde and the wire codec store and then read back as something
  /// else. Guarding the pairs one at a time is what produced this rule rather than the pairs: the
  /// fold is applied until the rendering is STABLE, so a fold's interaction with another does not
  /// have to be anticipated to be handled.
  ///
  /// # Why it terminates, and where that is settled
  ///
  /// Not here. The registries are static, so the bound is a fact about them and
  /// [`cargo xtask gen-lang`](super::registry::MAX_GRANDFATHERED_HOPS) is where it is PROVEN: the
  /// generator walks every grandfathered tag through these same rules, follows the chain each
  /// fold's output opens, refuses a registry in which one cycles, and emits the longest chain it
  /// found as [`MAX_GRANDFATHERED_HOPS`](super::registry::MAX_GRANDFATHERED_HOPS). This loop
  /// iterates at most that many times — so a registry bump that lengthened a chain fails
  /// GENERATION, and cannot reach production as a fold that quietly stops short.
  ///
  /// The `debug_assert!` after the loop is the other half of that bargain, asserting what the proof
  /// promised: the value the loop hands back names no further fold.
  fn canonicalized(mut self) -> Result<Self, ParseLanguageIdError> {
    for _ in 0..registry::MAX_GRANDFATHERED_HOPS {
      let Some(preferred) = self.folds_onto() else {
        return Ok(self);
      };
      self = Self::composed(preferred)?;
    }

    debug_assert!(
      self.folds_onto().is_none(),
      "the whole-tag fold did not reach a fixed point in {} hops — the generated bound and the \
       registry disagree, which `cargo xtask check` is supposed to have caught",
      registry::MAX_GRANDFATHERED_HOPS
    );

    Ok(self)
  }

  /// The replacement the registry names for THIS identity's own canonical text, or [`None`] where
  /// its text is not a grandfathered tag.
  ///
  /// The loop's one question, and it is asked of the RENDERING rather than of the seats because the
  /// table is keyed by whole tags. Nothing is allocated to ask it: the canonical text is written
  /// lower-cased straight into a stack buffer no wider than the widest tag the table holds, and a
  /// value too wide for that buffer cannot be in the table and answers [`None`] without a lookup —
  /// which is every ordinary tag, so the loop costs a bounded copy and a binary search.
  fn folds_onto(&self) -> Option<&'static str> {
    let mut buffer = [0u8; GRANDFATHERED_MAX];
    registry::grandfathered_preferred(self.lowered_canonical(&mut buffer)?)
  }

  /// This identity's canonical text, ASCII-lowered into `buffer`, or [`None`] where it is wider
  /// than the buffer.
  ///
  /// The same text [`Display`] writes, in the case the grandfathered table is keyed by — so it is
  /// [`to_string`](std::string::ToString::to_string) followed by
  /// [`to_ascii_lowercase`](str::to_ascii_lowercase) with neither allocation, which is what lets
  /// the fold be asked on every construction rather than only where a caller suspects it.
  fn lowered_canonical<'buffer>(
    &self,
    buffer: &'buffer mut [u8; GRANDFATHERED_MAX],
  ) -> Option<&'buffer str> {
    let mut width = push_lowered(buffer, 0, self.language.as_str())?;

    for part in [
      self.script.as_ref().map(ScriptSubtag::as_str),
      self.region.as_ref().map(Region::as_str),
      self.rest.as_ref().map(Utf8Bytes::as_str),
    ]
    .into_iter()
    .flatten()
    {
      width = push_lowered(buffer, width, "-")?;
      width = push_lowered(buffer, width, part)?;
    }

    let written = &buffer[..width];

    debug_assert!(
      simdutf8::compat::from_utf8(written).is_ok(),
      "the four seats are ASCII, so their rendering is valid UTF-8"
    );

    // SAFETY: every byte written above is ASCII, so the buffer holds valid UTF-8. The three bounded
    // seats hold ASCII by construction — each door refuses a non-ASCII subtag before a value
    // exists — the separator is `-`, and the tail went through `tail`, which admits only ASCII
    // alphanumerics. `to_ascii_lowercase` maps an ASCII byte to an ASCII byte, so it cannot leave
    // the alphabet the claim rests on, and `width` is what `push_lowered` returned rather than the
    // buffer's own length — so the slice holds only bytes this call wrote.
    Some(unsafe { core::str::from_utf8_unchecked(written) })
  }

  /// The three seats and the tail, read positionally out of a normalized tag.
  fn composed(tag: &str) -> Result<Self, ParseLanguageIdError> {
    let parts: Vec<&str> = tag.split('-').collect();

    if parts.iter().any(|part| part.is_empty()) {
      return Err(ParseLanguageIdError::EmptySubtag);
    }

    let language = Language::new(parts[0]).map_err(ParseLanguageIdError::Language)?;
    let mut at = 1;

    let script = match parts.get(at) {
      Some(part) if script_shaped(part) => {
        at += 1;
        Some(ScriptSubtag::new(part).map_err(ParseLanguageIdError::ScriptSubtag)?)
      }
      _ => None,
    };

    let region = match parts.get(at) {
      Some(part) if region_shaped(part) => {
        at += 1;
        Some(Region::new(part).map_err(ParseLanguageIdError::Region)?)
      }
      _ => None,
    };

    // The tail is a contiguous SUFFIX of the tag, so it is SLICED rather than rebuilt from the
    // parts: each consumed subtag cost its own length plus the separator after it, so the sum is
    // exactly where the tail begins. Slicing is what makes "verbatim" true of the separators too.
    let consumed: usize = parts[..at].iter().map(|part| part.len() + 1).sum();
    let rest = match consumed < tag.len() {
      true => Some(tail(&tag[consumed..])?),
      false => None,
    };

    Ok(Self::compose(language, script, region, rest))
  }

  /// A tag from its four parts, with the suppression applied — and **the crate's own road, not a
  /// public one**.
  ///
  /// It TRUSTS its `rest`: the three subtags arrive through their own doors and cannot be
  /// malformed, but the tail is a bare text seat with nothing about its type that says it went
  /// through the envelope. A public constructor with that hole would let a caller mint a
  /// `LanguageId` whose rendering does not parse back, so the seat is `pub(crate)` and every
  /// in-crate caller hands it a tail that [`tail`] has just checked.
  ///
  /// **The public road from parts is [`Display`] then [`FromStr`]**: rendering four seats and
  /// walking the standard door validates all four in one pass, which is the same road the storage
  /// layers downstream take when they reassemble a tag out of separate columns.
  ///
  /// The SUPPRESSION lives here and only here, which is what keeps the two routes in agreement — a
  /// `LanguageId` built from `en` and `Latn` is the same value as one parsed from `"en-Latn"`, and
  /// both are `en`. It is applied here rather than at [`ScriptSubtag`]'s own door because it is a
  /// fact about the PAIR: a script means nothing beside a language that implies it, and everything
  /// beside one that does not.
  ///
  /// **And it fires only where it is REVERSIBLE.** Dropping a subtag rewrites the rendered tag, and
  /// the rendered tag is this type's stored identity — so the fold is guarded by
  /// [`tail_opens_in_the_script_slot`], which is where the one case that would rewrite it into a
  /// DIFFERENT value is turned away.
  ///
  /// **It is ONE PASS, and not the whole canonicalisation.** The suppression it applies rewrites
  /// the tag, so its output can be a tag the whole-tag table folds again — which is
  /// [`canonicalized`](Self::canonicalized)'s subject, and why the door calls this in a loop rather
  /// than once. A caller reaching this directly gets the composition rules applied; a caller
  /// walking the public road ([`Display`] then [`FromStr`]) gets the fixed point.
  ///
  /// NOT `const`, and the reason is the SUPPRESSION rather than the seats: it reads
  /// `Suppress-Script` out of the generated table, and a binary search over a static is not a
  /// `const fn`. Nothing here drops — every seat is moved into place, never overwritten.
  #[must_use]
  pub(crate) fn compose(
    language: Language,
    script: Option<ScriptSubtag>,
    region: Option<Region>,
    rest: Option<Utf8Bytes>,
  ) -> Self {
    Self {
      script: match (&script, language.suppressed_script()) {
        (Some(declared), Some(implied))
          if declared.as_str() == implied && !tail_opens_in_the_script_slot(rest.as_ref()) =>
        {
          None
        }
        _ => script,
      },
      language,
      region,
      rest,
    }
  }

  /// The language, which every identity has.
  ///
  /// BY VALUE, as the other two bounded seats are: a [`Language`] is eight bytes of inline ASCII
  /// and [`Copy`], so a copy is a register move and a borrow would only make the caller's life
  /// harder.
  #[inline]
  #[must_use]
  pub const fn language(&self) -> Language {
    self.language
  }

  /// The script, where one was declared and the language does not already imply it.
  #[inline]
  #[must_use]
  pub const fn script(&self) -> Option<ScriptSubtag> {
    self.script
  }

  /// The region, where one was declared.
  #[inline]
  #[must_use]
  pub const fn region(&self) -> Option<Region> {
    self.region
  }

  /// Everything past the region, verbatim — variants, extensions and the private-use sequence.
  ///
  /// A BORROW, where the three seats above are copies, and the asymmetry is the grammar's: the
  /// three are bounded and this one is not, so it is the household's only heap-backed seat and the
  /// only one a caller has to ask for by reference.
  #[inline]
  #[must_use]
  pub const fn rest(&self) -> Option<&Utf8Bytes> {
    self.rest.as_ref()
  }
}

/// The undetermined tag, `und`.
///
/// **`und` is a VALUE, and it is not the absence of a tag** — the distinction
/// [`Language`]'s own doc argues, and the one this row is most at risk of blurring. A track nobody
/// tagged is an `Option::None`; a track a muxer tagged `und` has an identity, and it says the muxer
/// looked and could not tell. This is the second of those, which is what a wire codec needs when it
/// has to seed a value before reading one.
impl Default for LanguageId {
  #[inline]
  fn default() -> Self {
    Self::from(Language::UND)
  }
}

/// The widest subtag a tail may hold — the outer envelope of every variant, extension and
/// private-use subtag BCP 47 defines.
const TAIL_SUBTAG_MAX: usize = 8;

/// One tail through its envelope, held verbatim.
///
/// The envelope is deliberately LOOSE: one to eight ASCII alphanumeric characters per subtag, which
/// is the outer shape of every variant, extension and private-use subtag together rather than any
/// one of their grammars. See [`LanguageId`], where the reason is the same one that admits an
/// unregistered language subtag.
fn tail(text: &str) -> Result<Utf8Bytes, ParseLanguageIdError> {
  for part in text.split('-') {
    if part.is_empty() {
      return Err(ParseLanguageIdError::EmptySubtag);
    }
    if part.len() > TAIL_SUBTAG_MAX {
      return Err(ParseLanguageIdError::TailWidth);
    }
    if let Some(outside) = part
      .chars()
      .find(|character| !character.is_ascii_alphanumeric())
    {
      return Err(ParseLanguageIdError::Tail(outside));
    }
  }

  Ok(Utf8Bytes::from(text))
}

/// The widest whole tag the registry grandfathered, in bytes.
///
/// Nothing wider can be one, so nothing wider is looked up — which is what lets the case fold for
/// that lookup use a stack buffer rather than allocating a lower-cased copy of every tag that comes
/// through the door. `the_grandfathered_table_fits_the_lookup_buffer` is the pin that keeps the
/// shortcut from silently missing a tag the registry adds.
///
/// `pub(super)` rather than private: it is also the width the generated registry's `GRANDFATHERED`
/// key column is stored inline at, which is `subtag::MAX`'s reason for the same visibility — a
/// table's row width is not `id`'s secret once something outside the type needs to store the same
/// tag shape.
pub(super) const GRANDFATHERED_MAX: usize = 16;

/// The replacement the registry names for a grandfathered tag, or [`None`] for a tag that is not
/// grandfathered or is grandfathered without one.
///
/// The five with no replacement answer [`None`] here and fall through to the ordinary parse, which
/// is the honest handling: two of them ARE ordinary compositions read subtag by subtag, and the
/// three beginning `i-` are refused by the language door for a reason it can state.
///
/// UTF-8 validity of the folded buffer is PROVEN BY PROVENANCE, so the conversion back is
/// unchecked — see the `SAFETY` note, and [`subtag::Cased::as_str`](super::subtag) where the same
/// trade is argued. The claim is checked by `simdutf8` under `debug_assert!`, which is the
/// configuration this crate's `miri` and sanitizer lanes run in.
///
/// This buffer is WIDER than the case fold's, and unlike that one it is not preceded by an
/// alphabet test — a whole tag reaches here before any subtag door has seen it, so the bytes may be
/// non-ASCII. That costs nothing: `to_ascii_lowercase` leaves every byte above `0x7F` exactly as it
/// found it, so a multi-byte sequence is copied through unchanged rather than re-cased.
fn grandfathered(tag: &str) -> Option<&'static str> {
  if tag.len() > GRANDFATHERED_MAX {
    return None;
  }

  let mut lowered = [0u8; GRANDFATHERED_MAX];
  for (at, byte) in tag.as_bytes().iter().enumerate() {
    lowered[at] = byte.to_ascii_lowercase();
  }
  let lowered = &lowered[..tag.len()];

  debug_assert!(
    simdutf8::compat::from_utf8(lowered).is_ok(),
    "an ASCII case fold over a `&str` must leave valid UTF-8"
  );

  // SAFETY: these bytes are `tag.as_bytes()` — a `&str`, so valid UTF-8 — copied one for one, with
  // `to_ascii_lowercase` applied. That map is the identity on every byte outside `0x41..=0x5A`, so
  // it touches no continuation byte and no leading byte of a multi-byte sequence, and it maps an
  // ASCII byte to an ASCII byte. The prefix is therefore the same UTF-8 sequence `tag` is, with
  // some ASCII letters re-cased. It is `tag.len()` long, which the guard above has bounded by
  // `GRANDFATHERED_MAX`, the buffer's own width.
  let lowered = unsafe { core::str::from_utf8_unchecked(lowered) };

  registry::grandfathered_preferred(lowered)
}

/// Append `part`'s ASCII lower case to `buffer` at `width`, answering the width that leaves — or
/// [`None`] where it would not fit.
///
/// Overflow is an ANSWER rather than a failure, and that is the shortcut
/// [`lowered_canonical`](LanguageId::lowered_canonical) rests on: the only question asked of this
/// buffer is whether the text is a grandfathered tag, no such tag is wider than
/// [`GRANDFATHERED_MAX`], so text that does not fit is text the table cannot hold.
fn push_lowered(buffer: &mut [u8; GRANDFATHERED_MAX], width: usize, part: &str) -> Option<usize> {
  let end = width.checked_add(part.len())?;
  if end > GRANDFATHERED_MAX {
    return None;
  }

  for (at, byte) in part.as_bytes().iter().enumerate() {
    buffer[width + at] = byte.to_ascii_lowercase();
  }

  Some(end)
}

/// Is this subtag in the SCRIPT position's shape — exactly four ASCII letters?
fn script_shaped(part: &str) -> bool {
  part.len() == 4
    && part
      .chars()
      .all(|character| character.is_ascii_alphabetic())
}

/// Is this subtag in the REGION position's shape — two ASCII letters, or three ASCII digits?
fn region_shaped(part: &str) -> bool {
  let letters = part.len() == 2
    && part
      .chars()
      .all(|character| character.is_ascii_alphabetic());
  let digits = part.len() == 3 && part.chars().all(|character| character.is_ascii_digit());

  letters || digits
}

/// Would a REPARSE of the suppressed text read the tail's first subtag as the SCRIPT?
///
/// The guard on the `Suppress-Script` fold, and the whole of why that fold is not unconditional.
///
/// # The invariant it defends
///
/// [`Display`] and the door are inverse, and everything downstream leans on that: serde writes the
/// rendered tag and reads it back through [`LanguageId::new`], and the wire codec does the same. So
/// a canonicalisation here does not merely tidy a rendering — it decides what a stored row
/// deserialises to, and it may rewrite the text ONLY where the rewritten text reads back as the
/// same four seats.
///
/// # The case where the suppression would not
///
/// The tail is a TAIL, so it holds whatever the seats declined, and its envelope is deliberately
/// loose — which makes `Cyrl` a legal tail subtag. `en-Latn-Cyrl` fills both: `Latn` on the script
/// seat, `Cyrl` on the tail. Drop `Latn` and the rendering is `en-Cyrl`, whose script slot is now
/// VACANT and first in line, so a reparse reads `Cyrl` as the script and an empty tail. Three
/// answers change — [`script`](LanguageId::script), [`rest`](LanguageId::rest) and equality — and
/// every stored `en-Latn-Cyrl` becomes a different identity on its next read. `en-Latn-Latn` is the
/// same case with the tail's subtag landing back on the seat it was never on.
///
/// Retaining the script is what this answers `true` for, and retaining is always safe: with the
/// script in the text, the script slot is filled by the subtag that belongs in it and the tail
/// begins exactly where it began.
///
/// # Why there is no REGION-shaped arm
///
/// The sibling question is the same one asked of the next slot down — with the script gone, could a
/// reparse's REGION slot claim a tail opening with two letters or three digits? It would; no value
/// that reaches here can be in that state. `composed` offers the subtag after the script to the
/// region door FIRST, so only a subtag that door's shape test turned down can become the head of a
/// tail: an empty [`region`](LanguageId::region) beside a region-shaped tail head is
/// unconstructible. That parse road is the ONLY road that fills `rest` — `From<Language>` and
/// `Default` leave it empty, `compose` is `pub(crate)` and reached from nowhere else, there is no
/// setter, and serde and the wire codec are that same road under other names. The pin on the door's
/// greed is `the_region_slot_takes_its_subtag_before_a_tail_can_open_with_one`.
///
/// Worth saying, since it is why the census had to be total rather than defensive: retaining the
/// script would NOT have rescued that state. The region slot sits after the script slot, so a
/// script in the text does not shield the tail from it — only the door's greed does.
///
/// # Why it does not consult the region seat
///
/// With a region filled, the vacated script slot is taken by the region subtag — two letters or
/// three digits, never four letters — so the tail cannot climb into it, and `en-Latn-US-Cyrl` could
/// safely render as `en-US-Cyrl`. This test does not look, and the price is that such a tag keeps a
/// script it could have dropped. The price is nil: a four-letter tail head is outside BCP 47's tail
/// grammar to begin with — a variant is five to eight alphanumerics, or four characters beginning
/// with a DIGIT, and an extension or private-use singleton is one character — so every tag this
/// retains a script on is already outside the well-formed set. One shape test on one subtag is the
/// whole guard, and the answer it gives is never wrong, only occasionally generous.
fn tail_opens_in_the_script_slot(rest: Option<&Utf8Bytes>) -> bool {
  rest.is_some_and(|tail| tail.as_str().split('-').next().is_some_and(script_shaped))
}

/// The canonical spelling: the seats that are filled, in order, joined by `-`.
///
/// The inverse of [`FromStr`] over canonical text, which is what makes it the one text form
/// everything else in this crate — serde, the wire codec — reads and writes.
impl fmt::Display for LanguageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.language.as_str())?;

    if let Some(script) = &self.script {
      write!(f, "-{script}")?;
    }
    if let Some(region) = &self.region {
      write!(f, "-{region}")?;
    }
    if let Some(rest) = &self.rest {
      write!(f, "-{}", rest.as_str())?;
    }

    Ok(())
  }
}

/// The canonical tag, in quotes — not the four seats, which would make an assertion message four
/// times longer than the value it is about.
impl fmt::Debug for LanguageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "LanguageId({:?})", self.to_string())
  }
}

impl FromStr for LanguageId {
  type Err = ParseLanguageIdError;

  #[inline]
  fn from_str(text: &str) -> Result<Self, Self::Err> {
    Self::new(text)
  }
}

/// The standard fallible conversion, which is [`FromStr`] under the name a generic caller reaches
/// for. One door, so there is no second grammar to drift.
impl TryFrom<&str> for LanguageId {
  type Error = ParseLanguageIdError;

  #[inline]
  fn try_from(text: &str) -> Result<Self, Self::Error> {
    Self::new(text)
  }
}

/// The same door, off the text seat this household carries — so a tag that arrived as a
/// [`Utf8Bytes`] crosses without a caller reaching for `.as_str()` first.
impl TryFrom<Utf8Bytes> for LanguageId {
  type Error = ParseLanguageIdError;

  #[inline]
  fn try_from(text: Utf8Bytes) -> Result<Self, Self::Error> {
    Self::new(text.as_str())
  }
}

/// A tag with no other seats filled — the shape a bare language subtag composes to.
impl From<Language> for LanguageId {
  #[inline]
  fn from(language: Language) -> Self {
    Self {
      language,
      script: None,
      region: None,
      rest: None,
    }
  }
}

/// A string does not name a language identity.
///
/// The three seat variants CARRY the seat's own error rather than restating it, which is what keeps
/// one grammar per subtag kind: a caller sending `zh-Hans-XYZ` is told that a region written in
/// letters is exactly two, by [`Region`]'s own words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ParseLanguageIdError {
  /// Nothing was sent — an ABSENT identity rather than a malformed one, whose home is an
  /// `Option::None`.
  #[error("a language tag opens with a language subtag, and nothing was sent")]
  Empty,
  /// A subtag between two separators is empty: `en--US`, or a tag with a trailing `-`.
  #[error("a language tag has no empty subtag between its separators")]
  EmptySubtag,
  /// The first subtag is not a language. This is where `i-default` and `i-mingo` land — a
  /// one-letter primary subtag is outside the grammar, and those two are grandfathered tags the
  /// registry names no replacement for.
  #[error("the tag's language subtag is not one: {0}")]
  Language(#[source] ParseLanguageError),
  /// A subtag in the script position is four letters and still not a script. Unreachable today, the
  /// position test being the same width and alphabet the script door checks; it is propagated
  /// rather than unwrapped so that a widened position test cannot become a panic.
  #[error("the tag's script subtag is not one: {0}")]
  ScriptSubtag(#[source] ParseScriptSubtagError),
  /// A subtag in the region position is not a region, for the same reason and with the same
  /// posture.
  #[error("the tag's region subtag is not one: {0}")]
  Region(#[source] ParseRegionError),
  /// A tail subtag holds a character that is neither an ASCII letter nor an ASCII digit, carrying
  /// the first one found.
  #[error(
    "a variant, extension or private-use subtag is letters and digits, so `{0}` is not one of its \
     characters"
  )]
  Tail(char),
  /// A tail subtag is more than eight characters, which no variant, extension or private-use
  /// subtag is.
  #[error("a variant, extension or private-use subtag is at most eight characters")]
  TailWidth,
}
