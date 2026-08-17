//! Centralised `serde` implementations for the descriptor enums
//! (`feature = "serde"`).
//!
//! The wire shape mirrors what the storage backends (sqlx / mongodb /
//! async-graphql) independently chose, so a serde-`json` column matches
//! their representation byte-for-byte:
//!
//! - **Every vocabulary enum** — codecs, formats, the colour enums, the
//!   pixel format, the frame coded enums — serializes as its canonical
//!   `as_str()` slug: `VideoCodec::H264` ⇄ `"h264"`, `color::Matrix::Bt709`
//!   ⇄ `"bt709"`, `Other("x265")` ⇄ `"x265"` (no `{"Other": …}` wrapper).
//!   One extension idiom, one wire shape. Round-trip total wherever the
//!   `Other(SmolStr)` arm exists (the `alloc` tier); at the no-alloc tier
//!   the same enums are closed, so an unrecognised slug is a serde error
//!   rather than a silently-invented value.
//! - **`TrackDisposition`** is the one numeric wire left: it is a bit set,
//!   not a name vocabulary, so it serializes as its `u32` bits.
//! - **Strictly-closed coded enums (no `Other` arm)** —
//!   [`crate::subtitle::TrackOrigin`] and [`crate::audio::BitRateMode`] —
//!   serialize as their `u32` code but **reject** unknown wire codes as
//!   serde errors instead of silently collapsing them to the default
//!   variant (which `from_u32` would do). This is intentional: a corrupt
//!   or out-of-range value on the wire must fail loudly rather than
//!   masquerade as `Embedded` / `Cbr`. The check is backed by each type's
//!   `try_from_u32(v: u32) -> Option<Self>` method.
//!
//! The plain data structs (`color::Info`, `frame::Dimensions`,
//! `audio::Tags`, …) derive serde at their definition site; the
//! validated structs (`capture::GeoLocation`, `audio::Fingerprint`,
//! `audio::CoverArt`, `frame::WhiteBalance`,
//! `frame::ColorCorrectionMatrix`) route deserialize through their
//! checking constructors there too. `lang::Language` carries a bespoke BCP-47
//! string impl in its module.

/// Implements `Serialize` / `Deserialize` for an *open* enum via its
/// canonical string slug (`as_str()` to serialize, [`FromStr`] to parse).
/// The `FromStr` impl is total (`Err = Infallible`) — unknown slugs ride
/// the enum's `Other` arm — but the deserializer surfaces any error as a
/// serde error for forward-compatibility.
///
/// [`FromStr`]: core::str::FromStr
macro_rules! serde_via_str {
  ($t:path) => {
    impl serde::Serialize for $t {
      #[inline]
      fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
      }
    }

    impl<'de> serde::Deserialize<'de> for $t {
      fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl serde::de::Visitor<'_> for V {
          type Value = $t;
          fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(concat!("a ", stringify!($t), " slug string"))
          }
          #[inline]
          fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<$t>().map_err(serde::de::Error::custom)
          }
        }
        de.deserialize_str(V)
      }
    }
  };
}

/// Implements `Serialize` / `Deserialize` via a `u32` whose every value is
/// meaningful wire data — the bit-set case, where the number *is* the
/// value and there is no name to spell. `TrackDisposition` is the only
/// such type; name vocabularies use [`serde_via_str!`] instead.
macro_rules! serde_via_code {
  ($t:path) => {
    impl serde::Serialize for $t {
      #[inline]
      fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_u32(self.to_u32())
      }
    }

    impl<'de> serde::Deserialize<'de> for $t {
      #[inline]
      fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(<$t>::from_u32(<u32 as serde::Deserialize>::deserialize(
          de,
        )?))
      }
    }
  };
}

/// Implements `Serialize` / `Deserialize` for a **strictly-closed**
/// FFmpeg-coded enum — one with no escape arm at all — via its
/// `to_u32()` / `try_from_u32()` pair. Adversarial / corrupt codes outside
/// the enumerated set are rejected as serde errors instead of silently
/// canonicalising to the default variant (which `from_u32` would do).
// Both invocations (`TrackOrigin` / `BitRateMode`) are heap-tier — gated on
// `any(feature = "std", feature = "alloc")`. Under bare `--features serde`
// (no-alloc tier) they are cfg'd out and the macro is unused; the `allow`
// silences the resulting `unused_macros` lint, exactly as for `serde_via_str!`.
#[allow(unused_macros)]
macro_rules! serde_via_code_strict {
  ($t:path) => {
    impl serde::Serialize for $t {
      #[inline]
      fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_u32(self.to_u32())
      }
    }

    impl<'de> serde::Deserialize<'de> for $t {
      #[inline]
      fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let v = <u32 as serde::Deserialize>::deserialize(de)?;
        <$t>::try_from_u32(v).ok_or_else(|| {
          serde::de::Error::custom(::std::format!(
            "{}: unknown wire code {}",
            stringify!($t),
            v
          ))
        })
      }
    }
  };
}

// ── The bit set: a number is its only faithful spelling ──
serde_via_code!(crate::disposition::TrackDisposition);

// ── Name vocabularies available at every capability tier ──
// Open at the `alloc` tier (an unrecognised slug rides `Other`), closed at
// the no-alloc tier (it is a serde error) — one wire shape either way.
serde_via_str!(crate::color::Matrix);
serde_via_str!(crate::color::Primaries);
serde_via_str!(crate::color::Transfer);
serde_via_str!(crate::color::DynamicRange);
serde_via_str!(crate::color::ChromaLocation);
serde_via_str!(crate::color::DcpTargetGamut);
serde_via_str!(crate::pixel_format::PixelFormat);
serde_via_str!(crate::frame::Rotation);
serde_via_str!(crate::frame::FieldOrder);
serde_via_str!(crate::frame::StereoMode);

// ── The RAW / bayer vocabularies (behind the `bayer` feature) ──
// Closed: they name sensor layouts and demosaic algorithms, not an open
// space a backend extends, so an unrecognised slug is a serde error.
// `WhiteBalance` / `ColorCorrectionMatrix` are float structs and carry
// their own validating impls at their definition site.
#[cfg(feature = "bayer")]
serde_via_str!(crate::frame::BayerPattern);
#[cfg(feature = "bayer")]
serde_via_str!(crate::frame::BayerDemosaic);
#[cfg(feature = "bayer")]
serde_via_str!(crate::frame::WbChannel);

// ── Name vocabularies that need the allocator for their own payloads ──
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::VideoCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::AudioCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::codec::SubtitleCodec);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::container::Format);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::subtitle::Format);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::ChannelLayout);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::SampleFormat);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_str!(crate::audio::ContainerFormat);

// ── Strictly-closed coded enums (no `Unknown` escape) ──
// Use `serde_via_code_strict!` — adversarial / unknown wire codes are
// rejected as serde errors, not silently canonicalised to the default
// (which `from_u32` would do for `TrackOrigin::from_u32(999) == Embedded`).
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_code_strict!(crate::subtitle::TrackOrigin);
#[cfg(any(feature = "std", feature = "alloc"))]
serde_via_code_strict!(crate::audio::BitRateMode);

#[cfg(all(test, feature = "std"))]
mod tests {
  use crate::{
    audio::{ChannelLayout, CoverArt, Fingerprint, Tags},
    capture::GeoLocation,
    codec::VideoCodec,
    color::{self, Matrix},
    disposition::TrackDisposition,
    frame::{Dimensions, FrameRate, Rational, SampleAspectRatio},
    lang::Language,
  };

  fn round_trip<T>(v: &T) -> T
  where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
  {
    let json = serde_json::to_string(v).unwrap();
    let back: T = serde_json::from_str(&json).unwrap();
    assert_eq!(*v, back, "round-trip mismatch via {json}");
    back
  }

  #[test]
  fn open_enum_serializes_as_slug() {
    assert_eq!(
      serde_json::to_string(&VideoCodec::H264).unwrap(),
      "\"h264\""
    );
    round_trip(&VideoCodec::H264);
    // Unknown slug rides the `Other` arm losslessly.
    let custom = VideoCodec::Other(smol_str::SmolStr::new("zzcodec"));
    assert_eq!(serde_json::to_string(&custom).unwrap(), "\"zzcodec\"");
    round_trip(&custom);
    round_trip(&ChannelLayout::default());
  }

  #[test]
  fn colour_enum_serializes_as_its_name() {
    assert_eq!(serde_json::to_string(&Matrix::Bt709).unwrap(), "\"bt709\"");
    round_trip(&Matrix::Bt709);
    // A name this build does not enumerate rides the `Other` arm, and
    // keeps its name across the wire — the old numeric shape handed the
    // reader a bare code with nothing to call it.
    let vendor = Matrix::other("acescct");
    assert_eq!(serde_json::to_string(&vendor).unwrap(), "\"acescct\"");
    round_trip(&vendor);
  }

  #[test]
  fn structs_round_trip() {
    round_trip(&color::Info::default());
    round_trip(&Dimensions::new(1920, 1080));
    round_trip(&SampleAspectRatio::new(
      40,
      core::num::NonZeroI64::new(33).unwrap(),
    ));
    round_trip(&Tags::new().with_title("Song").with_year(2026));
    round_trip(&(TrackDisposition::DEFAULT | TrackDisposition::FORCED));
  }

  #[test]
  fn rational_deserialize_rejects_out_of_range_fields() {
    // The derived `Deserialize` assigns fields directly, so it is a
    // second construction path; under `i64`/`NonZeroI64` the types no
    // longer carry the sign invariant, and the `deserialize_with`
    // guards are what stop it minting a value `Rational::new` rejects.
    assert!(serde_json::from_str::<Rational>(r#"{"num":2,"den":4}"#).is_ok());
    assert!(serde_json::from_str::<Rational>(r#"{"num":-5,"den":4}"#).is_err());
    assert!(serde_json::from_str::<Rational>(r#"{"num":5,"den":-4}"#).is_err());
    // `NonZeroI64`'s own deserializer rejects zero before the guard runs.
    assert!(serde_json::from_str::<Rational>(r#"{"num":5,"den":0}"#).is_err());
    // The wrappers derive through `Rational`, so guarding it guards them.
    assert!(serde_json::from_str::<SampleAspectRatio>(r#"{"num":-1,"den":1}"#).is_err());
    assert!(
      serde_json::from_str::<FrameRate>(r#"{"rate":{"num":-1,"den":1},"is_vfr":false}"#).is_err()
    );
  }

  #[test]
  fn rational_serde_survives_above_u32_max() {
    let big = i64::from(u32::MAX) + 1;
    let r = Rational::new(big, core::num::NonZeroI64::new(big).unwrap());
    round_trip(&r);
  }

  #[test]
  fn language_round_trips_as_bcp47() {
    let l = Language::from_bcp47("zh-Hant-TW").unwrap();
    assert_eq!(serde_json::to_string(&l).unwrap(), "\"zh-Hant-TW\"");
    round_trip(&l);
    round_trip(&Language::default());
  }

  #[test]
  fn validated_structs_check_on_deserialize() {
    let g = GeoLocation::try_new(48.8584, 2.2945, Some(330.0)).unwrap();
    round_trip(&g);
    // Out-of-range latitude is rejected, not silently materialised.
    assert!(
      serde_json::from_str::<GeoLocation>(r#"{"lat":999.0,"lon":0.0,"altitude":null}"#).is_err()
    );

    let fp = Fingerprint::try_new("chromaprint", &b"\x01\x02\x03"[..]).unwrap();
    round_trip(&fp);
    // Empty algorithm violates the invariant and must be rejected.
    assert!(serde_json::from_str::<Fingerprint>(r#"{"algorithm":"","value":[1,2,3]}"#).is_err());

    let art = CoverArt::try_new("image/png", &b"\x89PNG"[..]).unwrap();
    round_trip(&art);
    // Empty mime violates the invariant and must be rejected.
    assert!(serde_json::from_str::<CoverArt>(r#"{"mime":"","data":[1]}"#).is_err());
  }

  // ── Codex round 1 findings ──

  /// `SampleFormat` is a plain name vocabulary now: the numeric arm that
  /// forced a bespoke two-shape codec is gone, so it rides the same slug
  /// wire as every other vocabulary, on both human-readable and binary
  /// formats.
  #[test]
  fn sample_format_rides_the_slug_wire() {
    use crate::audio::SampleFormat;
    assert_eq!(
      serde_json::to_string(&SampleFormat::S16).unwrap(),
      "\"s16\""
    );
    round_trip(&SampleFormat::S16);
    let other = SampleFormat::other("custom");
    assert_eq!(serde_json::to_string(&other).unwrap(), "\"custom\"");
    round_trip(&other);
  }

  /// Strictly-closed coded enums (no escape arm) must REJECT unknown
  /// wire codes instead of silently mapping them to the default. Previously
  /// `from_u32(999)` quietly returned `Embedded` / `Cbr`, so corrupt input
  /// looked like valid data on the consumer side.
  #[test]
  fn closed_coded_enums_reject_unknown_codes() {
    use crate::{audio::BitRateMode, subtitle::TrackOrigin};

    for o in [
      TrackOrigin::Embedded,
      TrackOrigin::Sidecar,
      TrackOrigin::External,
    ] {
      round_trip(&o);
    }
    for m in [BitRateMode::Cbr, BitRateMode::Vbr, BitRateMode::Abr] {
      round_trip(&m);
    }

    // Out-of-range codes are rejected — not canonicalised to the default.
    assert!(serde_json::from_str::<TrackOrigin>("999").is_err());
    assert!(serde_json::from_str::<TrackOrigin>("3").is_err());
    assert!(serde_json::from_str::<BitRateMode>("999").is_err());
    assert!(serde_json::from_str::<BitRateMode>("3").is_err());
  }

  // ── Codex round 2 findings ──

  /// The slug wire has to survive a non-self-describing binary format
  /// too — the earlier bespoke codec branched on `is_human_readable()`
  /// precisely because a bare `deserialize_any` does not work there.
  #[test]
  fn sample_format_postcard_binary_roundtrip() {
    use crate::audio::SampleFormat;

    fn binary_round_trip(v: &SampleFormat) -> SampleFormat {
      let bytes = postcard::to_allocvec(v).expect("postcard serialize");
      postcard::from_bytes::<SampleFormat>(&bytes).expect("postcard deserialize")
    }

    assert_eq!(binary_round_trip(&SampleFormat::S16), SampleFormat::S16);
    let other = SampleFormat::other("custom");
    assert_eq!(binary_round_trip(&other), other);
  }

  /// Default-backed metadata structs must accept sparse JSON — missing
  /// fields default rather than failing — so older / partial records
  /// remain readable as the schema evolves. `serde(default)` at the
  /// container level routes missing fields through `Default`.
  #[test]
  fn sparse_json_uses_serde_default_on_default_backed_structs() {
    use crate::{
      audio::{Loudness, Tags},
      capture::Device,
    };

    // Tags: only `title` present; the rest fall back to absent sentinels.
    let t: Tags = serde_json::from_str(r#"{"title":"hello"}"#).unwrap();
    let expected = Tags::new().with_title(smol_str::SmolStr::new("hello"));
    assert_eq!(t, expected);

    // Tags: completely empty object → fully-default value (no missing-field error).
    let empty: Tags = serde_json::from_str("{}").unwrap();
    assert_eq!(empty, Tags::default());

    // Device: only `make` present.
    let d: Device = serde_json::from_str(r#"{"make":"Apple"}"#).unwrap();
    let expected = Device::new().with_make(smol_str::SmolStr::new("Apple"));
    assert_eq!(d, expected);

    // Loudness: partial measurement.
    let l: Loudness = serde_json::from_str(r#"{"integrated_lufs":-23.0}"#).unwrap();
    assert_eq!(l, Loudness::new(-23.0, 0.0, 0.0, 0.0));
  }

  /// golden-rule §9: an absent `Option` field serializes to an *omitted
  /// key*, never `null`. Verified for every `Option`-bearing serde type.
  #[test]
  fn absent_option_fields_are_omitted_not_null() {
    use crate::{audio::Tags, capture::GeoLocation, color::HdrStaticMetadata};

    // `Tags.language` absent.
    let j = serde_json::to_string(&Tags::default()).unwrap();
    assert!(!j.contains("null"), "Tags emitted `null`: {j}");
    assert!(
      !j.contains("language"),
      "Tags emitted absent `language`: {j}"
    );

    // `HdrStaticMetadata` — both `mastering` / `content_light` absent.
    let j = serde_json::to_string(&HdrStaticMetadata::default()).unwrap();
    assert_eq!(
      j, "{}",
      "empty HdrStaticMetadata should serialize to `{{}}`"
    );

    // `GeoLocation.altitude` absent (hand-written `Serialize`).
    let g = GeoLocation::try_new(0.0, 0.0, None).unwrap();
    let j = serde_json::to_string(&g).unwrap();
    assert!(!j.contains("null"), "GeoLocation emitted `null`: {j}");
    assert!(
      !j.contains("altitude"),
      "GeoLocation emitted absent `altitude`: {j}"
    );
    // …and present `altitude` still round-trips.
    round_trip(&GeoLocation::try_new(0.0, 0.0, Some(12.5)).unwrap());
  }
}
