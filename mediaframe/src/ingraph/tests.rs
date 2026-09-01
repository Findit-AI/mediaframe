//! One page per citizen, and each of them asks the same four questions:
//! does a column of this type resolve with nothing written at the
//! declaration, does every word this build names page a cursor, does the
//! type's OPENNESS survive the round trip, and are bytes that are not this
//! vocabulary's encoding refused.
//!
//! The tests are per TYPE rather than per driver because that is what a
//! failure has to name. The drivers below are shared — a citizenship is
//! the same six rows twenty times over — so what a `#[test]` adds is the
//! roster it walks and the posture it claims.

use super::*;

use crate::disposition::TrackDisposition;

/// **Every word this build names pages a cursor, and the cursor is the
/// word.**
///
/// Two assertions rather than one: a round trip alone would pass for a
/// codec that wrote its discriminant, and the stored form being the
/// canonical slug is the fact the whole module rests on — a mirror stored
/// the word, the storage rows a backend build writes store the word, and a
/// cursor minted by one process is handed back to another that may be
/// talking to a different dialect.
fn pages_every_word<T>(roster: &[T])
where
  T: CursorValue + core::fmt::Display + core::fmt::Debug + PartialEq,
{
  assert!(!roster.is_empty(), "an empty roster tests nothing");

  for value in roster {
    let mut out = Vec::new();
    value.write_cursor(&mut out);

    assert_eq!(
      out,
      value.to_string().as_bytes(),
      "the canonical word and nothing framing it",
    );
    let read_back = T::read_cursor(&out);
    assert_eq!(
      read_back.as_ref(),
      Some(value),
      "`{value}` does not page back"
    );

    // And the bytes are a FIXED POINT: decoding and re-encoding writes the
    // same cursor. This is the property a keyset BOUNDARY needs — a page
    // that resumes and re-mints must not walk — and it is asserted
    // separately from the round trip above because the two can come apart
    // for a value whose word is not its own.
    let mut again = Vec::new();
    read_back.expect("it paged back").write_cursor(&mut again);
    assert_eq!(again, out, "`{value}`'s cursor is not a fixed point");
  }
}

/// **An OPEN vocabulary absorbs a word it does not name**, carrying that
/// word's own spelling — which is what lets a column outlive an upstream
/// release that adds one.
///
/// The probe is lowercase because the escape folds ASCII case at the door:
/// a folded word is what the vocabulary stores, so a lowercase probe is
/// the one whose round trip is the identity rather than a normalisation.
fn absorbs_a_word_it_does_not_name<T>(probe: &str) -> T
where
  T: CursorValue + core::fmt::Display,
{
  let absorbed = T::read_cursor(probe.as_bytes())
    .expect("an open vocabulary's parse is total, so every word names a value");

  assert_eq!(
    absorbed.to_string(),
    probe,
    "an absorbed word keeps its own spelling",
  );

  absorbed
}

/// **A CLOSED vocabulary refuses a word outside its set**, which is what
/// keeps a forged cursor from becoming a page.
fn refuses_a_word_outside_its_set<T>(probe: &str)
where
  T: CursorValue + core::fmt::Debug,
{
  assert!(
    T::read_cursor(probe.as_bytes()).is_none(),
    "`{probe}` names no value of this vocabulary and must not come back out of a client's string",
  );
}

/// **Bytes that are not UTF-8 are refused by every citizen**, open ones
/// included.
///
/// The openness is about WORDS. A cursor is a string a client hands back,
/// and a byte sequence that is not text at all is not a word this
/// vocabulary declined to name — it is not a word.
fn refuses_bytes_that_are_not_text<T>()
where
  T: CursorValue + core::fmt::Debug,
{
  assert!(
    T::read_cursor(&[0xff, 0xfe]).is_none(),
    "a non-UTF-8 cursor is not this vocabulary's encoding",
  );
}

/// The one thing a `ROSTER` walk needs that the constants themselves do
/// not provide: a name to reach them through generically.
///
/// The `roster!` macro emits an inherent `ROSTER` per vocabulary, and an
/// inherent const cannot be named from a generic driver. This trait is one
/// line per citizen pointing at the constant that already exists — it
/// restates no member, and a vocabulary that grew one has nothing here to
/// edit.
///
/// Two of the twenty carry no `roster!` — the CLOSED pair, whose members
/// are written out below — so this is where their rosters live rather than
/// a second table beside a first.
trait Rostered: Sized + 'static {
  const ROSTER: &'static [Self];
}

macro_rules! rostered {
  ($( $ty:ty ),+ $(,)?) => {
    $(
      impl Rostered for $ty {
        const ROSTER: &'static [Self] = <$ty>::ROSTER;
      }
    )+
  };
}

rostered! {
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
  crate::frame::FieldOrder,
  crate::frame::Rotation,
  crate::frame::StereoMode,
  crate::pixel_format::PixelFormat,
  crate::subtitle::format::Format,
  crate::subtitle::track_origin::TrackOrigin,
}

/// The CLOSED pair's rosters, written out because neither vocabulary
/// carries a `roster!` of its own.
impl Rostered for crate::audio::bit_rate_mode::BitRateMode {
  const ROSTER: &'static [Self] = &[Self::Cbr, Self::Vbr, Self::Abr];
}

#[cfg(feature = "bayer")]
impl Rostered for crate::frame::BayerPattern {
  const ROSTER: &'static [Self] = &[Self::Bggr, Self::Rggb, Self::Grbg, Self::Gbrg];
}

/// The **resolution** rows, checked by construction: a column of this type
/// infers the enumeration reading, its filter is the enumeration filter,
/// and a collection of it infers a list of that reading — with nothing
/// written at the declaration.
///
/// Identity functions rather than `let _:` bindings, because the marker
/// types have private fields and cannot be built here. Nothing runs; each
/// fails to COMPILE if a row goes missing or names something else.
macro_rules! resolves_as_a_vocabulary {
  ($ty:ty) => {{
    fn is_the_enum_reading(m: EnumMarker<$ty>) -> <$ty as DefaultMarker>::Marker {
      m
    }
    fn is_the_enum_filter(f: ingraph::EnumFilter<$ty>) -> <$ty as EnumFilterMarker>::Filter {
      f
    }
    fn is_a_list_of_that_reading(
      m: ListMarker<Vec<$ty>, EnumMarker<$ty>>,
    ) -> <$ty as DefaultVecMarker>::Marker {
      m
    }

    let _ = (
      is_the_enum_reading,
      is_the_enum_filter,
      is_a_list_of_that_reading,
    );

    // A `ColumnKind` with no `SEGMENTS` of its own is the scalar answer:
    // one column, whatever a dialect widens it to.
    assert!(<$ty as ColumnKind>::SEGMENTS.is_empty());

    // And the sameness row reads the WORD. Every rostered member is its
    // own word, so on the roster it agrees with the vocabulary's `==`
    // both ways — which is what makes the seam with the retiring mirror
    // unchanged. The class where it deliberately does NOT agree has its
    // own page below.
    let roster = <$ty as Rostered>::ROSTER;
    for (index, value) in roster.iter().enumerate() {
      assert!(value.column_eq(value), "a member is its own column value");
      for other in &roster[index + 1..] {
        assert!(
          !value.column_eq(other),
          "two distinct members are two column values",
        );
      }
    }
  }};
}

/// One page per OPEN citizen: resolution, the roster's cursor walk, the
/// absorption its escape promises, and the refusal every citizen owes.
macro_rules! open_citizen {
  ($( $name:ident : $ty:ty, $probe:literal );+ $(;)?) => {
    $(
      #[test]
      fn $name() {
        resolves_as_a_vocabulary!($ty);
        pages_every_word(<$ty as Rostered>::ROSTER);

        let absorbed = absorbs_a_word_it_does_not_name::<$ty>($probe);
        assert!(
          !<$ty as Rostered>::ROSTER.contains(&absorbed),
          "the probe must be a word this build does NOT name, or it proves nothing",
        );

        let mut out = Vec::new();
        absorbed.write_cursor(&mut out);
        assert_eq!(
          <$ty>::read_cursor(&out).as_ref(),
          Some(&absorbed),
          "an absorbed word pages a cursor like any other",
        );

        let mut again = Vec::new();
        <$ty>::read_cursor(&out)
          .expect("it paged back")
          .write_cursor(&mut again);
        assert_eq!(again, out, "and its cursor is a fixed point too");

        refuses_bytes_that_are_not_text::<$ty>();
      }
    )+
  };
}

open_citizen! {
  the_channel_layout_citizenship:   crate::audio::channel_layout::ChannelLayout, "9.1.6.4";
  the_audio_container_citizenship:  crate::audio::format::ContainerFormat,       "shn";
  the_sample_format_citizenship:    crate::audio::format::SampleFormat,          "s24";
  the_audio_codec_citizenship:      crate::codec::AudioCodec,                    "notacodec";
  the_subtitle_codec_citizenship:   crate::codec::SubtitleCodec,                 "notacodec";
  the_video_codec_citizenship:      crate::codec::VideoCodec,                    "notacodec";
  the_chroma_location_citizenship:  crate::color::ChromaLocation,                "elsewhere";
  the_dynamic_range_citizenship:    crate::color::DynamicRange,                  "hdr-vivid";
  the_color_matrix_citizenship:     crate::color::Matrix,                        "unnamedmatrix";
  the_color_primaries_citizenship:  crate::color::Primaries,                     "unnamedprimaries";
  the_color_transfer_citizenship:   crate::color::Transfer,                      "unnamedtransfer";
  the_container_format_citizenship: crate::container::Format,                    "notacontainer";
  the_field_order_citizenship:      crate::frame::FieldOrder,                    "interleaved";
  the_rotation_citizenship:         crate::frame::Rotation,                      "37";
  the_stereo_mode_citizenship:      crate::frame::StereoMode,                    "quadrascopic";
  the_pixel_format_citizenship:     crate::pixel_format::PixelFormat,            "yuv420p24le";
  the_subtitle_format_citizenship:  crate::subtitle::format::Format,             "notaformat";
  the_track_origin_citizenship:     crate::subtitle::track_origin::TrackOrigin,  "conjured";
}

/// One page per CLOSED citizen. Everything the open pages ask, plus the
/// refusal that is the whole difference: a word outside the set names no
/// value, and the cursor says so rather than inventing one.
macro_rules! closed_citizen {
  ($( $( #[$gate:meta] )* $name:ident : $ty:ty, $probe:literal );+ $(;)?) => {
    $(
      $( #[$gate] )*
      #[test]
      fn $name() {
        resolves_as_a_vocabulary!($ty);
        pages_every_word(<$ty as Rostered>::ROSTER);
        refuses_a_word_outside_its_set::<$ty>($probe);
        refuses_bytes_that_are_not_text::<$ty>();
      }
    )+
  };
}

closed_citizen! {
  the_bit_rate_mode_citizenship: crate::audio::bit_rate_mode::BitRateMode, "crf";
  #[cfg(feature = "bayer")]
  the_bayer_pattern_citizenship: crate::frame::BayerPattern, "xtrans";
}

/// **The cursor CANONICALISES, and these are the two shapes where that is
/// visible.**
///
/// Pinned rather than left implicit, because both are consequences of the
/// cursor being the vocabulary's own text form and neither is a fact a
/// reader would guess from the six rows.
///
/// The first is a cursor spelled in another case — the fold at the
/// vocabulary's door, which the framework's emitted storage rows perform
/// on read for the same reason.
///
/// The second is the SHADOW VALUE: `other()` folds case but does not
/// parse, so `VideoCodec::other("h264")` is a distinct Rust value whose
/// word is `H264`'s. No face in this crate keeps the two apart — the
/// asserts below show `serde`, released and unrelated to this feature,
/// collapsing it exactly as the cursor does — so the cursor inherits that
/// answer rather than inventing a second one. Removing the shadow at its
/// source means making `other()` parse, which is a change to twenty
/// shipped constructors and its own ticket.
#[test]
fn the_cursor_canonicalises_case_and_the_shadow_escape() {
  use crate::codec::VideoCodec;

  let upper = VideoCodec::read_cursor(b"H264").expect("the parse is total");
  assert_eq!(
    upper,
    VideoCodec::H264,
    "a cursor's case is folded at the door"
  );

  let shadow = VideoCodec::other("h264");
  assert_ne!(shadow, VideoCodec::H264, "two Rust values");
  assert_eq!(
    shadow.as_str(),
    VideoCodec::H264.as_str(),
    "and ONE word — which is what every face of this crate stores",
  );

  let mut out = Vec::new();
  shadow.write_cursor(&mut out);
  assert_eq!(out, b"h264", "the word, as every other face writes it");
  assert_eq!(
    VideoCodec::read_cursor(&out),
    Some(VideoCodec::H264),
    "so it pages back as the member its word names",
  );
}

/// **The sameness row reads the WORD, and this is the class where that
/// differs from the derived `PartialEq`.**
///
/// [`ColumnEq`] answers whether two values are one STORED value, and for
/// these vocabularies the stored value is the word. Both shapes `other()`
/// admits — a word already a member's, and a word that is a member's under
/// another spelling — are therefore ONE column value with the member they
/// shadow, however the derive compares them.
///
/// Asserted rather than left to the row's doc, because this is the one row
/// in the module that does not delegate to something the type already
/// carries, and a reader has to be able to see the two answers part.
#[test]
fn two_values_with_one_word_are_one_column_value() {
  use crate::{codec::VideoCodec, container::Format};

  let shadow = VideoCodec::other("h264");
  assert_ne!(shadow, VideoCodec::H264, "the derive calls them two values");
  assert!(
    shadow.column_eq(&VideoCodec::H264),
    "a column holds one word for them, so it holds one value",
  );
  assert!(VideoCodec::H264.column_eq(&shadow), "and it is symmetric");

  // The ALIAS shape parts the other way: `qt` and `mov` are two different
  // words, so they are two different column values — even though the
  // vocabulary reads both as `Mov`.
  let quicktime = Format::other("qt");
  assert!(
    !quicktime.column_eq(&Format::Mov),
    "two words are two column values, whatever they parse to",
  );

  // And distinct members stay distinct, which is the property the roster
  // walk on every citizen's page asserts in full.
  assert!(!VideoCodec::H264.column_eq(&VideoCodec::Vp9));
}

/// **The ALIAS shadow, which is the shadow's sharper form** — and the
/// reason a keyset boundary still cannot walk.
///
/// `other()` folds case but does not parse, so it can also be handed a
/// slug the vocabulary recognises under ANOTHER name: `qt` is `Mov`'s and
/// `adts` is `Aac`'s. Such a value's word is not merely shared with a
/// member's, it is DIFFERENT from the word it reads back as — `qt` in,
/// `mov` out — so unlike the same-word shadow the encoded bytes move.
///
/// What stops that from moving a page boundary is where a cursor is minted
/// FROM. A cursor names a position in a stored order, and a value out of a
/// store has been through the framework's own text decode, which parses:
/// `ingraph`'s emitted read for an open vocabulary is `Self::new(stored)`
/// and the retiring mirror's is its slug map, and both answer `Mov` for a
/// row holding `qt`. So the value a cursor is minted from is already
/// canonical, and a canonical value's cursor is a fixed point — which
/// every citizen's page above asserts, member by member.
///
/// Reaching this at all takes a hand-built `other(alias)` used directly as
/// a boundary, never read back from a store and never produced by
/// `FromStr`. The release below shows the same value moving through the
/// shipped `serde` face, so the road out is `other()` parsing before it
/// escapes — twenty released constructors, and its own ticket.
#[test]
fn an_alias_escape_reads_back_as_the_member_it_names() {
  use crate::{audio::ContainerFormat, container::Format};

  let quicktime = Format::other("qt");
  assert_eq!(
    quicktime.as_str(),
    "qt",
    "the escape kept the word it was given"
  );
  assert_eq!(
    Format::read_cursor(b"qt"),
    Some(Format::Mov),
    "and `qt` is a word this vocabulary names, under `mov`",
  );

  let mut out = Vec::new();
  quicktime.write_cursor(&mut out);
  let read_back = Format::read_cursor(&out).expect("the parse is total");
  let mut again = Vec::new();
  read_back.write_cursor(&mut again);
  assert_eq!(again, b"mov", "the word moves ONCE");
  assert_eq!(
    Format::read_cursor(&again),
    Some(read_back),
    "and is a fixed point from there on — a resumed page does not walk",
  );

  // The same shape one household over, so the class is pinned rather than
  // one example of it.
  assert_eq!(ContainerFormat::other("adts").as_str(), "adts");
  assert_eq!(
    ContainerFormat::read_cursor(b"adts"),
    Some(ContainerFormat::Aac),
  );
}

/// The shadow value's collapse is **older than this feature**, and the
/// released `serde` face is the witness.
///
/// Stated here so the paragraph above rests on an assertion rather than a
/// claim: if a later release ever made `other()` parse, this page fails
/// and the cursor's note gets revisited with it.
#[test]
#[cfg(feature = "serde")]
fn the_shadow_escape_already_collapses_through_serde() {
  use crate::codec::VideoCodec;

  let shadow = VideoCodec::other("h264");
  let json = serde_json::to_string(&shadow).expect("a codec serialises as its word");
  assert_eq!(json, "\"h264\"");
  assert_eq!(
    serde_json::from_str::<VideoCodec>(&json).expect("and parses back"),
    VideoCodec::H264,
    "the released text form collapses the shadow too — the cursor is not \
     introducing this, it is inheriting it",
  );

  // And the ALIAS shadow moves through the same released face, word and
  // all, which is what makes `other()` the single source of both.
  let alias = crate::container::Format::other("qt");
  let json = serde_json::to_string(&alias).expect("a container serialises as its word");
  assert_eq!(json, "\"qt\"");
  assert_eq!(
    serde_json::from_str::<crate::container::Format>(&json).expect("and parses back"),
    crate::container::Format::Mov,
  );
}

/// **One table, read twice.**
///
/// The word a bit is spelled with comes from `bitflags`' own table and the
/// field a client selects it by comes from [`FlagsValue::FIELDS`], and the
/// framework zips them. A pair that drifted — an added bit, a renamed
/// constant — would show up here as a mismatched word, which is the one
/// failure two parallel lists can have.
///
/// The words are the ones the retiring mirror published, verbatim: this is
/// a MOVE of the wire face from the consumer that restated it, not a new
/// spelling for it.
#[test]
fn every_disposition_bit_carries_its_word_and_its_wire_field() {
  let table: Vec<_> = ingraph::flags::bits::<TrackDisposition>()
    .map(|bit| (bit.word, bit.field))
    .collect();

  assert_eq!(
    table,
    [
      ("DEFAULT", "default"),
      ("DUB", "dub"),
      ("ORIGINAL", "original"),
      ("COMMENT", "comment"),
      ("LYRICS", "lyrics"),
      ("KARAOKE", "karaoke"),
      ("FORCED", "forced"),
      ("HEARING_IMPAIRED", "hearingImpaired"),
      ("VISUAL_IMPAIRED", "visualImpaired"),
      ("CLEAN_EFFECTS", "cleanEffects"),
      ("ATTACHED_PIC", "attachedPic"),
      ("TIMED_THUMBNAILS", "timedThumbnails"),
      ("NON_DIEGETIC", "nonDiegetic"),
      ("CAPTIONS", "captions"),
      ("DESCRIPTIONS", "descriptions"),
      ("METADATA", "metadata"),
      ("DEPENDENT", "dependent"),
      ("STILL_IMAGE", "stillImage"),
    ],
  );
}

/// The field list is **exactly as long as the bit table**, which is what
/// makes the zip above total.
///
/// The framework zips rather than indexes, so a short list loses its tail
/// in silence — a bit that exists, is stored, and has no field for a client
/// to ask it by. This is the assertion that would fire the day a bit is
/// added to [`disposition`](crate::disposition) and the field list is not.
#[test]
fn the_disposition_field_list_names_every_declared_bit() {
  use bitflags::Flags as _;

  assert_eq!(
    <TrackDisposition as FlagsValue>::FIELDS.len(),
    TrackDisposition::FLAGS.len(),
    "a field per bit, or the framework's zip drops the tail",
  );
}

/// The schema publishes the type under its own Rust name — the operand
/// scalar and the filter input object are composed from this one string, so
/// it is the only name a client ever sees.
#[test]
fn the_disposition_schema_name_is_the_types_own() {
  assert_eq!(
    <TrackDisposition as FlagsValue>::GRAPHQL_NAME,
    "TrackDisposition",
  );
}

/// **The disposition citizenship resolves as FLAGS**, which is the whole
/// reason it is not one of the twenty above: a bit set answers
/// set-theoretic words, and a vocabulary answers word-set ones.
#[test]
fn the_track_disposition_citizenship() {
  fn is_the_flags_reading(
    m: FlagsMarker<TrackDisposition>,
  ) -> <TrackDisposition as DefaultMarker>::Marker {
    m
  }
  fn is_the_flags_filter(
    f: ingraph::FlagsFilter<TrackDisposition>,
  ) -> <TrackDisposition as FlagsFilterMarker>::Filter {
    f
  }
  fn is_a_list_of_that_reading(
    m: ListMarker<Vec<TrackDisposition>, FlagsMarker<TrackDisposition>>,
  ) -> <TrackDisposition as DefaultVecMarker>::Marker {
    m
  }

  let _ = (
    is_the_flags_reading,
    is_the_flags_filter,
    is_a_list_of_that_reading,
  );

  assert!(<TrackDisposition as ColumnKind>::SEGMENTS.is_empty());

  let default = TrackDisposition::DEFAULT;
  let also_default = TrackDisposition::from_bits_truncate(0x0000_0001);
  assert!(default.column_eq(&also_default));
  assert!(!default.column_eq(&TrackDisposition::FORCED));
  assert!(TrackDisposition::empty().column_eq(&TrackDisposition::empty()));
}

/// **A disposition cursor is four big-endian bytes, and every declared bit
/// pages through them.**
#[test]
fn the_disposition_cursor_round_trips_every_declared_bit() {
  use bitflags::Flags as _;

  for bit in TrackDisposition::FLAGS {
    let value = *bit.value();
    let mut out = Vec::new();
    value.write_cursor(&mut out);

    assert_eq!(out, value.bits().to_be_bytes(), "four bytes, big-endian");
    assert_eq!(TrackDisposition::read_cursor(&out), Some(value));
  }

  let mixed =
    TrackDisposition::DEFAULT | TrackDisposition::HEARING_IMPAIRED | TrackDisposition::ATTACHED_PIC;
  let mut out = Vec::new();
  mixed.write_cursor(&mut out);
  assert_eq!(
    TrackDisposition::read_cursor(&out),
    Some(mixed),
    "a subset costs exactly what one bit costs",
  );
}

/// **An UNNAMED bit pages, and a wrong WIDTH does not.**
///
/// The two halves are the whole of this type's cursor domain. Its domain
/// is every `u32` — the bits are append-only `AV_DISPOSITION_*` and
/// [`from_u32`](crate::disposition::TrackDisposition::from_u32) keeps one
/// this build cannot name verbatim — so a pattern with an unnamed bit is
/// a value a demuxer newer than this file legitimately produces, and a
/// cursor minted from it has to page back. A strict read would refuse
/// bytes `write_cursor` had just written, which is the codec law broken
/// from the inside rather than a forgery caught.
///
/// The WIDTH check is the one this type does have a domain for: four
/// bytes or nothing, whatever they hold.
///
/// The fixture is a real hole rather than an invented one: `NON_DIEGETIC`
/// is `0x0000_1000` and `CAPTIONS` is `0x0001_0000`, and the bits between
/// them carry no name upstream.
#[test]
fn an_unnamed_disposition_bit_pages_and_a_wrong_width_does_not() {
  const UNNAMED_BIT: u32 = 0x0000_2000;

  assert!(
    TrackDisposition::from_bits(UNNAMED_BIT).is_none(),
    "the fixture must be a bit this build does NOT name, or it proves nothing",
  );

  let from_a_newer_demuxer = TrackDisposition::from_u32(UNNAMED_BIT | 0x1);
  assert_eq!(
    from_a_newer_demuxer.bits(),
    0x0000_2001,
    "the type keeps a bit it cannot name — that is its own contract",
  );

  let mut out = Vec::new();
  from_a_newer_demuxer.write_cursor(&mut out);
  assert_eq!(
    TrackDisposition::read_cursor(&out),
    Some(from_a_newer_demuxer),
    "and what this row writes, it reads back — a legitimate value must not mint an \
     unpageable cursor",
  );

  assert_eq!(
    TrackDisposition::read_cursor(&[]),
    None,
    "no bytes, no value"
  );
  assert_eq!(
    TrackDisposition::read_cursor(&[0, 0, 1]),
    None,
    "three bytes are not this type's four",
  );
  assert_eq!(
    TrackDisposition::read_cursor(&[0, 0, 0, 0, 1]),
    None,
    "nor are five",
  );
}
