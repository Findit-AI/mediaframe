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
//!   rather than a silently-invented value. Deserialization goes through
//!   the type's `FromStr`, so it also reads the documented FFmpeg
//!   synonyms (`"gray"` → `PixelFormat::Gray8`, `"unknown"` →
//!   `color::Matrix::Unspecified`); serialization stays canonical, so a
//!   synonym read off the wire is written back in the canonical spelling.
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
mod tests;
