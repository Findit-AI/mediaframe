//! **This crate's self-granted [`ingraph`] citizenship** — the faces the
//! indexing framework reads its vocabulary types through, written here
//! because here is where those types live.
//!
//! Nothing in this module is reachable without the `ingraph` feature, and
//! nothing outside it changes when the feature is on.
//!
//! # Why the faces belong on this side
//!
//! Three things have to be attached to a type before a declaration can hold
//! a column of it — how the column is READ (its filter, and the
//! interpretation it takes when a declaration names none), what it IS in
//! storage, and how a keyset cursor pages past it. Every one of those is a
//! foreign trait, so only two crates may write them: the framework, or the
//! crate that owns the type.
//!
//! For a while it was neither — it was a THIRD party. Downstream of both,
//! a consumer kept a **mirror** per vocabulary: a local enum restating this
//! crate's variant list, with a crossing in each direction and a drift pin
//! over the pair. That works, and it costs a permanent restatement — one
//! that has to be edited every time a vocabulary here grows a word, in a
//! crate that has no reason to know the word exists. Twenty-one of them
//! stood against this crate alone.
//!
//! Self-granting removes the restatement instead of maintaining it. The
//! rows below are ordinary trait work — this crate's own types, the
//! framework's traits, orphan-clean — and they leave the vocabularies
//! exactly where they were: nothing about [`PixelFormat`](crate::pixel_format::PixelFormat)
//! changes shape, nothing about it is decided by `ingraph`, and a build
//! without the feature has never heard of it.
//!
//! # The roster is a CENSUS, and the mirrors are what it counted
//!
//! Twenty-one types, and they are exactly the ones a consumer restates
//! today — `mediagraph`'s `types::` tree, whose twenty-one declarations
//! each carry `remote = "mediaframe::…"` naming one of them. That is the
//! whole selection rule: a citizenship for a vocabulary nobody stores
//! would be surface nobody asked for, and this crate's public face is much
//! wider than its stored one (dimensions, rationals, plane geometry,
//! channel specs, loudness — values a graph moves THROUGH a node rather
//! than values a row keeps).
//!
//! | household | citizens |
//! |---|---|
//! | [`audio`](crate::audio) | `BitRateMode`, `ChannelLayout`, `SampleFormat`, `ContainerFormat` |
//! | [`codec`](crate::codec) | `VideoCodec`, `AudioCodec`, `SubtitleCodec` |
//! | [`color`](crate::color) | `Matrix`, `Primaries`, `Transfer`, `DynamicRange`, `ChromaLocation` |
//! | [`container`](crate::container) | `Format` |
//! | [`disposition`](crate::disposition) | `TrackDisposition` |
//! | [`frame`](crate::frame) | `BayerPattern`, `FieldOrder`, `Rotation`, `StereoMode` |
//! | [`pixel_format`](crate::pixel_format) | `PixelFormat` |
//! | [`subtitle`](crate::subtitle) | `Format`, `TrackOrigin` |
//!
//! [`BayerPattern`](crate::frame::BayerPattern) is the one citizen behind a
//! second gate: the type itself is `#[cfg(feature = "bayer")]`, so its
//! rows are, and a build taking `ingraph` without `bayer` grants
//! citizenship to the twenty types it has.
//!
//! # What a VOCABULARY citizenship is, in rows
//!
//! | row | what asks for it |
//! |---|---|
//! | [`EnumFilterMarker`] | the column's filter: the word-set operators an enumeration answers |
//! | [`DefaultMarker`] / [`DefaultVecMarker`] | the inference, so a column of this type needs no word in the declaration to be read |
//! | [`CursorValue`] | the keyset cursor, which every persisted column is audited for |
//! | [`ColumnKind`] / [`ColumnEq`] | the column's width, and when two of these values are one column value |
//!
//! [`ColumnEq`] is the one row that is not a delegation to something the
//! type already carries: it reads the WORD rather than the derived
//! `PartialEq`, because a column holds the word. See the row.
//!
//! The VARIANT LIST is not among them, and that is the property this whole
//! module rests on. Each vocabulary already carries its own `as_str`, its
//! own `FromStr` and its own `ROSTER`; the rows below name those rather
//! than restating them, so a variant added upstream of this crate reaches
//! the framework's faces with nothing here to edit — which is exactly what
//! a mirror could not do.
//!
//! It is also why the rows are hand-written where a fresh vocabulary would
//! wear `#[derive(ingraph::Enum)]`. That derive MINTS a text face —
//! `as_str`, `FromStr`, `VARIANTS`, `is_<variant>`, a refusal type — and
//! every one of those already exists here, with this crate's own
//! case-folding door behind it. A derive would be a second face over one
//! vocabulary, and the two would be free to disagree.
//!
//! # The stored form is the WORD, and the cursor is the same word
//!
//! Every vocabulary here renders as its canonical slug and parses back
//! from it, so that is what a cursor carries. A discriminant is not
//! available rather than merely worse: sixteen of the twenty are OPEN, and
//! the value their `Other` arm holds is a name FFmpeg minted after this
//! crate's table was generated — a value with no number to be.
//!
//! What that means at [`read_cursor`](CursorValue::read_cursor) differs by
//! posture, and it differs because the vocabularies do:
//!
//! - an OPEN vocabulary parses TOTALLY — its `FromStr::Err` is
//!   [`Infallible`](core::convert::Infallible) at this tier — so any UTF-8
//!   a client hands back names a value, the unrecognised ones landing in
//!   the escape under their own spelling. That is the escape doing its
//!   job, and it is the same answer the mirrors gave.
//! - a CLOSED one ([`BitRateMode`](crate::audio::BitRateMode),
//!   [`BayerPattern`](crate::frame::BayerPattern)) REFUSES a word outside
//!   its set, which is what keeps a forged cursor from becoming a page.
//!
//! Both halves check UTF-8 first: a cursor is a string a client hands
//! back, so the bytes are arbitrary.
//!
//! ## The cursor is the WORD, so it round-trips a value's WORD
//!
//! The read parses, which means it CANONICALISES — and that is worth
//! stating outright, because it makes the cursor a codec over the stored
//! form rather than over the Rust value. Two things follow, and both are
//! properties this crate already had before any of these rows existed:
//!
//! - a cursor spelled in another case comes back as the member it names.
//!   That is the fold at the vocabulary's own door, and the framework's
//!   emitted storage rows canonicalise on read for the same reason: a row
//!   written by another program reads back as the member it names.
//! - a value built through [`other()`](crate::codec::VideoCodec::other)
//!   carrying a slug the vocabulary DOES name comes back as the member
//!   that names it. That door folds case but does not PARSE, so it admits
//!   two shapes its own doc says the escape is not for: `other("h264")`,
//!   whose word is already `H264`'s, and `other("qt")`, whose word is
//!   `Mov`'s under another spelling. The first reads back as `H264` and
//!   the second as `Mov`, spelled `mov`.
//!
//! What that costs is exact: `read_cursor(write_cursor(v))` is `v` for
//! every value whose word is its own, and is the value that WORD names
//! otherwise. That is the invariant a keyset cursor actually needs — a
//! cursor names a position in the STORED order, and the stored order is
//! over words — and a cursor that tagged named values apart from escaped
//! ones would be the one face in this crate disagreeing with the column
//! it pages.
//!
//! **A boundary cannot walk on it**, which is the sharper question the
//! alias shape raises, and the answer is where a cursor is minted FROM. A
//! cursor is minted from a row's value, and a value out of a store has
//! been through the framework's own text decode, which parses — the
//! emitted read for an open vocabulary is `Self::new(stored)`, and the
//! retiring mirror's is its slug map. Both answer `Mov` for a row holding
//! `qt`. So a cursor's source value is already canonical, and a canonical
//! value's cursor is a FIXED POINT: decoding and re-encoding writes the
//! same bytes, asserted member by member on every citizen's page.
//!
//! None of this is the citizenship's to introduce or to fix. The shipped
//! `serde` feature moves both shapes identically today, on the released
//! crate, and the module's tests assert that beside the cursor's own
//! behaviour so the claim is checked rather than asserted. The road out is
//! `other()` parsing before it escapes, which would fix `serde` with it —
//! twenty released public constructors, and its own ticket rather than a
//! passenger on an additive feature.
//!
//! # What it does NOT carry, and where that line is
//!
//! The **storage bind** (`sqlx`'s `Type`/`Encode`/`Decode` and the
//! per-dialect carrier), the **document form** (`MongoCarrier`), the
//! **row read** (`Assemble`) and the **wire seats** (`ToGraphqlOutput` /
//! `ToGraphqlInput`) are absent. Each rides a feature of `ingraph`'s that
//! names a backend or a wire library, and this crate takes `ingraph`
//! without them — see the manifest row, which measures what the one word
//! it does take costs. A stream-descriptor vocabulary that pulled a SQL
//! driver, a BSON codec and a GraphQL runtime into its dependency graph to
//! describe a pixel format would be paying for a build it never runs.
//!
//! The rows are reachable when somebody wants them: `ingraph` publishes
//! `if_sqlite!`, `if_postgres!`, `if_mysql!` and `if_mongo!` precisely so
//! a per-backend half can be written beside a declaration and expand only
//! where that backend is compiled in. This module writes none of them
//! because no consumer has asked; a consumer that does can say so on the
//! ticket rather than discovering the gap.
//!
//! ## A PRECONDITION on whoever writes that half
//!
//! **Land the `other()` fix first.** The shape above — a door that folds
//! but does not parse — is harmless while these types are unstorable, and
//! stops being harmless the moment a `Carrier` and an `Assemble` exist for
//! them. A backend's encode writes the word (`as_str`) and its decode
//! parses it, so a row written from `other("qt")` is stored under `qt` and
//! read back as `Mov`. The in-memory value no longer matches the key it
//! came from, and a keyset boundary minted from it names `mov` while the
//! row still sorts at `qt` — which in an ascending `(format, id)` order
//! can return that row again on the next page.
//!
//! No cursor codec can repair that: by the time a value reaches
//! [`write_cursor`](CursorValue::write_cursor) the original spelling is
//! already gone. It has to be closed at one of the two ends that create it
//! — `other()` parsing before it escapes, so a non-canonical word is never
//! stored, or a backend half that keeps the raw stored spelling through
//! assembly. The first is the smaller change and fixes `serde` with it.
//!
//! Recorded here rather than in a ticket alone because this is the module
//! a future backend row would be written into, and the constraint is
//! invisible from the row itself.

use ingraph::{
  ColumnEq, ColumnKind, CursorValue, DefaultMarker, DefaultVecMarker, EnumFilterMarker, EnumMarker,
  FlagsFilterMarker, FlagsMarker, FlagsValue, ListMarker,
};

#[cfg(test)]
mod tests;

/// The six rows a VOCABULARY takes, written once and spent per citizen.
///
/// One macro rather than twenty hand-copied blocks, because the rows are
/// not per-type facts: each of them names something the vocabulary already
/// carries — its `as_str`, its `FromStr`, its `PartialEq` — so the only
/// thing that varies between two citizens is the path. A block per type
/// would be twenty chances to write `Rotation`'s cursor over
/// `FieldOrder`'s word.
///
/// What is NOT hidden by it is the openness split. A closed vocabulary's
/// `read_cursor` refuses and an open one's cannot, and neither arm is
/// written here — both fall out of the vocabulary's own `FromStr`, whose
/// `Err` is its own refusal in the first case and
/// [`Infallible`](core::convert::Infallible) in the second. The macro
/// spends `.ok()` on both, and the posture is the type's.
macro_rules! vocabulary {
  ($( $( #[$gate:meta] )* $ty:ty ),+ $(,)?) => {
    $(
      /// The filter a column of this vocabulary gets — the word-set
      /// operators, which is the whole reason an enumeration column is not
      /// a text column.
      $( #[$gate] )*
      impl EnumFilterMarker for $ty {
        type Filter = ingraph::EnumFilter<Self>;
      }

      /// How a column of this type is read when a declaration names no
      /// interpretation: as an enumeration, which is what it is.
      $( #[$gate] )*
      impl DefaultMarker for $ty {
        type Marker = EnumMarker<Self>;
      }

      /// And how a *collection* of them is read — a list of vocabulary
      /// values, each element under the reading above.
      $( #[$gate] )*
      impl DefaultVecMarker for $ty {
        type Marker = ListMarker<Vec<Self>, EnumMarker<Self>>;
      }

      /// The keyset cursor's byte form: the canonical slug, checked back
      /// through this vocabulary's own parse.
      $( #[$gate] )*
      impl CursorValue for $ty {
        fn write_cursor(&self, out: &mut Vec<u8>) {
          out.extend_from_slice(self.as_str().as_bytes());
        }

        fn read_cursor(bytes: &[u8]) -> Option<Self> {
          core::str::from_utf8(bytes).ok()?.parse().ok()
        }
      }

      /// One column, whatever a dialect widens it to.
      $( #[$gate] )*
      impl ColumnKind for $ty {}

      /// Two of these are one column value when their WORDS are.
      ///
      /// [`ColumnEq`] asks what it means for two STORED values of one
      /// column to be the same value, and for these vocabularies the
      /// stored value is the word: the SQL bind, the document form, the
      /// wire scalar and the page cursor all carry `as_str`. So this row
      /// reads what a column holds rather than offering a second opinion
      /// about the Rust value — which is [`regex::Regex`]'s row in
      /// `ingraph` exactly, for the same sentence.
      ///
      /// It is NOT the derived `PartialEq`, and the difference is one
      /// value class wide. `other()` folds case but does not parse, so a
      /// caller can build an escape carrying a word the roster already
      /// names — `other("h264")`, whose word is `H264`'s, and
      /// `other("qt")`, whose word is `Mov`'s under another spelling. The
      /// derive calls those distinct; a column cannot, because it has one
      /// word for them. On every value reachable through `FromStr`,
      /// through the roster, or out of a store — which is every value the
      /// retiring mirror could hold at all — the two answers coincide, so
      /// nothing at that seam changes.
      ///
      /// The relation is `str` equality, so it is reflexive, symmetric
      /// and transitive, which the emitted row equality that folds over it
      /// inherits.
      ///
      /// [`regex::Regex`]: https://docs.rs/regex
      $( #[$gate] )*
      impl ColumnEq for $ty {
        #[inline]
        fn column_eq(&self, other: &Self) -> bool {
          self.as_str() == other.as_str()
        }
      }
    )+
  };
}

vocabulary! {
  crate::audio::bit_rate_mode::BitRateMode,
  crate::audio::channel_layout::ChannelLayout,
  crate::audio::format::ContainerFormat,
  crate::audio::format::SampleFormat,
  crate::codec::AudioCodec,
  crate::codec::SubtitleCodec,
  crate::codec::VideoCodec,
  crate::color::ChromaLocation,
  crate::color::DynamicRange,
  crate::color::Matrix,
  crate::color::Primaries,
  crate::color::Transfer,
  crate::container::Format,
  #[cfg(feature = "bayer")]
  crate::frame::BayerPattern,
  crate::frame::FieldOrder,
  crate::frame::Rotation,
  crate::frame::StereoMode,
  crate::pixel_format::PixelFormat,
  crate::subtitle::format::Format,
  crate::subtitle::track_origin::TrackOrigin,
}

/// The schema name and the per-bit wire fields.
///
/// The two facts `bitflags` has no reason to carry, and the only two this
/// crate writes: the bit table itself is
/// [`bitflags::Flags::FLAGS`](bitflags::Flags::FLAGS), which
/// [`TrackDisposition`](crate::disposition::TrackDisposition) already has.
///
/// `FIELDS` is parallel to that table — same order, one entry per bit —
/// and the framework zips the two. The words are the constants'
/// (`DEFAULT`, `HEARING_IMPAIRED`, …); the fields are those words under
/// GraphQL's own casing convention, which is the single difference between
/// the two lists and the reason there are two.
impl FlagsValue for crate::disposition::TrackDisposition {
  const GRAPHQL_NAME: &'static str = "TrackDisposition";
  const FIELDS: &'static [&'static str] = &[
    "default",
    "dub",
    "original",
    "comment",
    "lyrics",
    "karaoke",
    "forced",
    "hearingImpaired",
    "visualImpaired",
    "cleanEffects",
    "attachedPic",
    "timedThumbnails",
    "nonDiegetic",
    "captions",
    "descriptions",
    "metadata",
    "dependent",
    "stillImage",
  ];
}

/// The filter a column of these bits gets — the set-theoretic words, which
/// is the whole reason a flags column is not an enumeration column.
impl FlagsFilterMarker for crate::disposition::TrackDisposition {
  type Filter = ingraph::FlagsFilter<Self>;
}

/// How a column of this type is read when a declaration names no
/// interpretation: as flags, which is what it is.
impl DefaultMarker for crate::disposition::TrackDisposition {
  type Marker = FlagsMarker<Self>;
}

/// And how a *collection* of them is read — a list of flags values, each
/// element under the reading above.
impl DefaultVecMarker for crate::disposition::TrackDisposition {
  type Marker = ListMarker<Vec<Self>, FlagsMarker<Self>>;
}

/// The keyset cursor's byte form: the bit pattern, big-endian, width-
/// checked coming back.
///
/// The pattern and not a word, because this type deliberately has no text
/// form — see its own note, where a bit SET is argued to take a number
/// where a vocabulary of NAMES takes a word.
///
/// # `from_bits_retain`, and the ONE place this row parts from its mirror
///
/// [`CursorValue`] asks a read to check its own width, its own charset and
/// its own DOMAIN, and the whole question here is what this type's domain
/// IS. It is every `u32`, stated by the type rather than inferred: the
/// bits are FFmpeg's `AV_DISPOSITION_*`, they are append-only, and
/// [`from_u32`](crate::disposition::TrackDisposition::from_u32) keeps a
/// bit this build has no name for verbatim — the type's own note promises
/// the round trip is lossless *for any wire value*. So a pattern with an
/// unnamed bit is not a forged value; it is a disposition from a demuxer
/// newer than this file.
///
/// A strict `from_bits` here would therefore break the codec law in the
/// direction that matters: `write_cursor` emits such a value's bits
/// faithfully and `read_cursor` would then refuse them, so a perfectly
/// legitimate row would mint a cursor that cannot page. The WIDTH check is
/// what still refuses bytes this type never wrote — four bytes or nothing
/// — and it is the only check that has a domain to check against.
///
/// The retiring mirror reads its cursor through `from_bits`, and this row
/// does not follow it there. That is deliberate and it is the only place
/// the move is not verbatim: the mirror could not REPRESENT an unnamed bit
/// in a value of its own, so the strict read cost it nothing, while this
/// type is built to carry one. `mediadecode`'s `PacketFlags` row is strict
/// for the same reason the mirror's is — a bit outside its declaration is
/// a value that type cannot spell — and the difference between the two
/// rows is a difference between the two types' domains, not a difference
/// of opinion about cursors.
impl CursorValue for crate::disposition::TrackDisposition {
  fn write_cursor(&self, out: &mut Vec<u8>) {
    out.extend_from_slice(&self.bits().to_be_bytes());
  }

  fn read_cursor(bytes: &[u8]) -> Option<Self> {
    Some(Self::from_bits_retain(u32::from_be_bytes(
      <[u8; 4]>::try_from(bytes).ok()?,
    )))
  }
}

/// One column, whatever a dialect widens it to.
impl ColumnKind for crate::disposition::TrackDisposition {}

/// Two of these are one column value when their bits are — which is what
/// the `PartialEq` the type already derives says.
impl ColumnEq for crate::disposition::TrackDisposition {
  #[inline]
  fn column_eq(&self, other: &Self) -> bool {
    self == other
  }
}
