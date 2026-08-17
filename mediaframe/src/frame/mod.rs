//! Frame primitives + the typed source-format `*Frame<'a, BE>` borrow types.
//!
//! ## Always-available primitives
//!
//! - [`Dimensions`] — a `(width, height)` pair in pixels.
//! - [`Rect`] — an axis-aligned integer rectangle (used for visible-region
//!   crops on `VideoFrame`).
//! - [`Rotation`] — display rotation (0 / 90 / 180 / 270).
//! - [`SampleAspectRatio`] — pixel aspect ratio (SAR).
//! - [`Plane<B>`] — one plane of pixel data, generic over the buffer type.
//! - [`VideoFrame<P, B>`] — runtime-tagged frame (no timestamp).
//! - [`TimestampedFrame<F>`] — orthogonal time-carrying wrapper.
//!
//! ## Typed `*Frame<'a, BE>` borrow types (feature-gated)
//!
//! Each pixel-format family is gated behind its own feature flag so
//! consumers compile only the formats they need. Enable an individual
//! family (e.g. `yuv-planar`) or the `frame` umbrella to opt in.
//!
//! | Feature           | Formats                                              |
//! |-------------------|------------------------------------------------------|
//! | `yuv-planar`      | Yuv420p / 422p / 444p / 440p / 411p / 410p + 9-16bit |
//! | `yuv-semi-planar` | NV12 / 16 / 21 / 24 / 42, P010 / 210 / 410 families  |
//! | `yuva`            | YUVA planar 8-bit + high-bit                         |
//! | `yuv-packed`      | YUYV422, UYVY422, YVYU422, UYYVYY411                 |
//! | `yuv-444-packed`  | V410/XV30/XV36/AYUV64/VUYA/VUYX/V30X/AYUV/UYVA/VYU444 |
//! | `y2xx`            | Y210 / Y212 / Y216                                   |
//! | `v210`            | V210                                                 |
//! | `rgb`             | Rgb24/Bgr24/Rgba/Bgra + 16-bit + Rgb96/Rgba128       |
//! | `rgb-float`       | Rgbf32 / Rgbf16 + Rgbaf32 / Rgbaf16                  |
//! | `rgb-legacy`      | Rgb444/555/565 + Bgr counterparts                    |
//! | `gbr`             | Gbrp / Gbrap + 9-16bit + float                       |
//! | `gray`            | Gray8-16/32, Grayf16/32, Ya8/16, Yaf16/32            |
//! | `bayer`           | Bayer 8-16bit, 4 patterns                            |
//! | `xyz`             | Xyz12                                                |
//! | `mono`            | Monoblack / Monowhite / Pal8                         |
//! | `frame`           | umbrella — enables every sub-feature above           |

// === Primitives (always available) ===

// ---- Shared error payload structs (used by per-family `*FrameError` enums) ----
//
// Variant names carry the per-plane / per-axis semantics
// (`InsufficientYStride`, `InsufficientUPlane`, …); the payload carries the
// shape-only data (the offending number + the reference number).
// Each payload has:
//   - private fields,
//   - a `pub const fn new(...)` constructor,
//   - one `pub const fn field(&self) -> T` getter per field,
//   - `#[inline]` on all methods.
// thiserror `#[error("...", .0.field())]` routes Display lookups
// through the getters so the original messages are preserved
// verbatim.

/// `width × height` carried by zero-dimension errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) or height ({height}) is zero")]
pub struct ZeroDimension {
  width: u32,
  height: u32,
}

impl ZeroDimension {
  /// Constructs a `ZeroDimension` payload.
  #[inline]
  pub const fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
  /// Returns the supplied width.
  #[inline]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Returns the supplied height.
  #[inline]
  pub const fn height(&self) -> u32 {
    self.height
  }
}

/// `width × height` carried by dimension-overflow errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("dimensions {width} × {height} overflow")]
pub struct DimensionOverflow {
  width: u32,
  height: u32,
}

impl DimensionOverflow {
  /// Constructs a `DimensionOverflow` payload.
  #[inline]
  pub const fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
  /// Returns the supplied width.
  #[inline]
  pub const fn width(&self) -> u32 {
    self.width
  }
  /// Returns the supplied height.
  #[inline]
  pub const fn height(&self) -> u32 {
    self.height
  }
}

/// Plane stride is smaller than what the declared geometry requires.
/// The variant name (e.g. `InsufficientYStride` vs `InsufficientUvStride`)
/// tells the caller which plane and what unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("stride ({stride}) is smaller than minimum ({min})")]
pub struct InsufficientStride {
  stride: u32,
  min: u32,
}

impl InsufficientStride {
  /// Constructs a `InsufficientStride` payload.
  #[inline]
  pub const fn new(stride: u32, min: u32) -> Self {
    Self { stride, min }
  }
  /// Returns the caller-supplied stride.
  #[inline]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Returns the required minimum.
  #[inline]
  pub const fn min(&self) -> u32 {
    self.min
  }
}

/// Plane buffer is shorter than the declared geometry requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("plane has {actual} bytes/samples but at least {expected} are required")]
pub struct InsufficientPlane {
  expected: usize,
  actual: usize,
}

impl InsufficientPlane {
  /// Constructs a `InsufficientPlane` payload.
  #[inline]
  pub const fn new(expected: usize, actual: usize) -> Self {
    Self { expected, actual }
  }
  /// Returns the minimum required length.
  #[inline]
  pub const fn expected(&self) -> usize {
    self.expected
  }
  /// Returns the actual length supplied.
  #[inline]
  pub const fn actual(&self) -> usize {
    self.actual
  }
}

/// Declared geometry (`stride × rows`) doesn't fit in `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("declared geometry overflows usize: stride={stride} * rows={rows}")]
pub struct GeometryOverflow {
  stride: u32,
  rows: u32,
}

impl GeometryOverflow {
  /// Constructs a `GeometryOverflow` payload.
  #[inline]
  pub const fn new(stride: u32, rows: u32) -> Self {
    Self { stride, rows }
  }
  /// Returns the stride that overflowed.
  #[inline]
  pub const fn stride(&self) -> u32 {
    self.stride
  }
  /// Returns the row count that overflowed.
  #[inline]
  pub const fn rows(&self) -> u32 {
    self.rows
  }
}

/// Width-alignment violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) {required}")]
pub struct WidthAlignment {
  /// Sink's configured width.
  width: usize,
  /// The alignment requirement that was violated.
  required: WidthAlignmentRequirement,
}

impl WidthAlignment {
  /// Constructs a new `WidthAlignment` payload.
  #[inline]
  const fn new(width: usize, required: WidthAlignmentRequirement) -> Self {
    Self { width, required }
  }

  /// Constructs a `WidthAlignment` payload for odd widths.
  #[inline]
  pub const fn odd(width: usize) -> Self {
    Self::new(width, WidthAlignmentRequirement::Even)
  }

  /// Constructs a `WidthAlignment` payload for widths that are not a
  #[inline]
  pub const fn multiple_of_four(width: usize) -> Self {
    Self::new(width, WidthAlignmentRequirement::MultipleOfFour)
  }

  /// Sink's configured width.
  #[inline]
  pub const fn width(&self) -> usize {
    self.width
  }

  /// The alignment requirement that was violated.
  #[inline]
  pub const fn required(&self) -> WidthAlignmentRequirement {
    self.required
  }
}

/// Discriminates which width-alignment rule was violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant, Display)]
#[non_exhaustive]
pub enum WidthAlignmentRequirement {
  /// Width must be even — 4:2:0 / 4:2:2 chroma-pair stride.
  #[display("is odd")]
  Even,
  /// Width must be a multiple of 4. Fired by planar 4:1:0
  /// ([`Yuv410p`](crate::source::Yuv410p)) and packed 4:1:1
  /// ([`Uyyvyy411`](crate::source::Uyyvyy411)). Note: planar 4:1:1
  /// ([`Yuv411p`](crate::source::Yuv411p)) accepts non-4-aligned
  /// widths via `width.div_ceil(4)` for the chroma row and is NOT
  /// covered by this discriminant.
  #[display("is not a multiple of 4")]
  MultipleOfFour,
}

/// Frame `width` value carried by per-row width-overflow errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("width ({width}) overflow")]
pub struct WidthOverflow {
  width: u32,
}

impl WidthOverflow {
  /// Constructs a `WidthOverflow` payload.
  #[inline]
  pub const fn new(width: u32) -> Self {
    Self { width }
  }
  /// Returns the supplied width.
  #[inline]
  pub const fn width(&self) -> u32 {
    self.width
  }
}

/// `BITS` const-generic value carried by unsupported-bits errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("unsupported BITS ({bits})")]
pub struct UnsupportedBits {
  bits: u32,
}

impl UnsupportedBits {
  /// Constructs an `UnsupportedBits` payload.
  #[inline]
  pub const fn new(bits: u32) -> Self {
    Self { bits }
  }
  /// Returns the supplied `BITS` value.
  #[inline]
  pub const fn bits(&self) -> u32 {
    self.bits
  }
}

/// A `(width, height)` pair in pixels.
///
/// Lives alongside the rest of the frame primitives because the same
/// pair shows up everywhere a video stream is described — the coded
/// dimensions of a `VideoFrame`, the `coded_*` parameters a backend
/// adapter takes when opening a decoder, the per-plane layout helpers
/// in a WebCodecs adapter, etc. Passing it as a single struct rather
/// than two separate `u32` arguments removes a long-running footgun
/// (silent argument swap) and gives a natural place to hang helpers
/// like [`Self::is_zero`] or `Display`.
///
/// `u32` width / height matches WebCodecs' `coded_width` /
/// `coded_height` typing in `web_sys` and FFmpeg's
/// `AVCodecContext::width` / `height`. 65535×65535 (the smaller `u16`
/// packing some adjacent crates use) covers every realistic
/// resolution; the `u32` choice here keeps the public API plug-
/// compatible with both adapter typings.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::dimensions")
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dimensions {
  width: u32,
  height: u32,
}

impl Dimensions {
  /// Constructs a `Dimensions` with the specified width and height
  /// in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }

  /// Returns the width in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the height in pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Sets the width (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_width(mut self, width: u32) -> Self {
    self.width = width;
    self
  }

  /// Sets the width in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_width(&mut self, width: u32) -> &mut Self {
    self.width = width;
    self
  }

  /// Sets the height (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_height(mut self, height: u32) -> Self {
    self.height = height;
    self
  }

  /// Sets the height in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_height(&mut self, height: u32) -> &mut Self {
    self.height = height;
    self
  }

  /// Returns `true` when both width and height are zero — typically
  /// the default-constructed / unset state.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_zero(&self) -> bool {
    self.width == 0 && self.height == 0
  }
}

impl core::fmt::Display for Dimensions {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}x{}", self.width, self.height)
  }
}

/// The error [`Dimensions`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Opaque and sealed; the rejected input is deliberately not retained.
/// `#[non_exhaustive]` keeps it constructible only here, so it can grow
/// structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a WIDTHxHEIGHT dimension pair")]
#[non_exhaustive]
pub struct ParseDimensionsError;

impl core::str::FromStr for Dimensions {
  type Err = ParseDimensionsError;

  /// Parses the `WIDTHxHEIGHT` form [`Display`](core::fmt::Display)
  /// renders (`"1920x1080"`).
  ///
  /// # Errors
  ///
  /// Returns [`ParseDimensionsError`] unless the input is exactly two
  /// `u32` values separated by a single `x`.
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (w, h) = s.split_once('x').ok_or(ParseDimensionsError)?;
    let width = w.parse().map_err(|_| ParseDimensionsError)?;
    let height = h.parse().map_err(|_| ParseDimensionsError)?;
    Ok(Self::new(width, height))
  }
}

/// An axis-aligned integer rectangle.
///
/// Used for `VideoFrame::visible_rect` (FFmpeg crop /
/// WebCodecs `visibleRect` / ProRes RAW `CleanAperture`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::rect")
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
  x: u32,
  y: u32,
  width: u32,
  height: u32,
}

impl Rect {
  /// Constructs a `Rect` at `(x, y)` with the given size.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
    Self {
      x,
      y,
      width,
      height,
    }
  }

  /// Returns the X coordinate of the top-left corner.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn x(&self) -> u32 {
    self.x
  }

  /// Returns the Y coordinate of the top-left corner.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn y(&self) -> u32 {
    self.y
  }

  /// Returns the width.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the height.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Sets the X coordinate (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_x(mut self, x: u32) -> Self {
    self.x = x;
    self
  }
  /// Sets the Y coordinate (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_y(mut self, y: u32) -> Self {
    self.y = y;
    self
  }
  /// Sets the width (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_width(mut self, w: u32) -> Self {
    self.width = w;
    self
  }
  /// Sets the height (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_height(mut self, h: u32) -> Self {
    self.height = h;
    self
  }

  /// Sets the X coordinate in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_x(&mut self, x: u32) -> &mut Self {
    self.x = x;
    self
  }
  /// Sets the Y coordinate in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_y(&mut self, y: u32) -> &mut Self {
    self.y = y;
    self
  }
  /// Sets the width in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_width(&mut self, w: u32) -> &mut Self {
    self.width = w;
    self
  }
  /// Sets the height in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_height(&mut self, h: u32) -> &mut Self {
    self.height = h;
    self
  }
}

/// Display rotation applied to the decoded picture before presentation.
///
/// Read from the FFmpeg display matrix side data
/// (`AV_FRAME_DATA_DISPLAYMATRIX` → `av_display_rotation_get`, which
/// returns a counter-clockwise angle in degrees) and from the
/// WebCodecs `VideoFrame` rotation attribute. Only the four
/// axis-aligned multiples of 90° are representable — every container
/// rotation tag in practice is one of these. Any other / future /
/// corrupt wire value is **rejected** by [`Self::from_u32`] rather than
/// silently collapsed to a valid rotation; a *name* this build does not
/// enumerate is carried verbatim as [`Self::Other`], the crate-wide
/// extension idiom.
///
/// The angle is the **clockwise** rotation to apply for display
/// (matching WebCodecs' `rotation`); callers normalising FFmpeg's
/// counter-clockwise convention negate accordingly. [`Self::D0`] is
/// the default (no rotation / square presentation).
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::rotation")
)]
pub enum Rotation {
  /// No rotation.
  #[default]
  D0,
  /// 90° clockwise.
  D90,
  /// 180°.
  D180,
  /// 270° clockwise (= 90° counter-clockwise).
  D270,
  /// A slug this vocabulary does not enumerate — carried verbatim,
  /// ASCII-folded to lowercase by the parse gate. The crate-wide
  /// extension idiom: a downstream backend naming a value mediaframe
  /// has never heard of keeps that **name**, and it round-trips through
  /// `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(SmolStr),
}

impl Rotation {
  /// Degree string for this rotation (`"0"` / `"90"` / `"180"` /
  /// `"270"`); [`Self::Other`] renders the name it carries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::D0 => "0",
      Self::D90 => "90",
      Self::D180 => "180",
      Self::D270 => "270",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id: `0`/`1`/`2`/`3` for
  /// `D0`/`D90`/`D180`/`D270`. Stable and append-only.
  ///
  /// [`None`] for [`Self::Other`]: it names a rotation this build does
  /// not enumerate, and there is no id to invent for it.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::D0 => 0,
      Self::D90 => 1,
      Self::D180 => 2,
      Self::D270 => 3,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the stable `u32` wire id produced by
  /// [`Self::to_u32`]. [`None`] for an unrecognised value — never a
  /// silent collapse to a default rotation.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::D0,
      1 => Self::D90,
      2 => Self::D180,
      3 => Self::D270,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name, ASCII-folded
  /// to the crate's lowercase canon.
  ///
  /// The **one** construction path for [`Self::Other`]: folding here is what
  /// keeps the whole value space lowercase-canonical, so the derived `Eq` /
  /// `Hash` compare names rather than spellings. Constructing the variant
  /// directly bypasses the fold and is not the supported spelling.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::Other(crate::parse::fold_owned(slug.as_ref()))
  }
}

/// The error [`Rotation`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Opaque and sealed: the input is deliberately not retained (these types
/// are available at the crate's no-alloc tier, where there is nowhere to
/// put an owned copy, and the input is attacker-controlled on the
/// deserialization path). `#[non_exhaustive]` keeps it constructible only
/// here, so it can grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a rotation")]
#[non_exhaustive]
pub struct ParseRotationError;

impl core::str::FromStr for Rotation {
  type Err = ParseRotationError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseError`](crate::parse::ParseError) only at the
  /// no-alloc tier, where the vocabulary is closed. With `alloc` this
  /// parse is **total**: a slug this type does not name rides
  /// [`Self::Other`], ASCII-folded to lowercase by [`Self::other`].
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s);
    Ok(match folded {
      "0" => Self::D0,
      "90" => Self::D90,
      "180" => Self::D180,
      "270" => Self::D270,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::other(s),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseRotationError),
    })
  }
}

/// Pixel (sample) aspect ratio — the ratio of a pixel's display
/// width to its display height.
///
/// Read from `AVStream.sample_aspect_ratio` /
/// `AVFrame.sample_aspect_ratio` (an FFmpeg `AVRational`) and from
/// the WebCodecs display-size derivation. A `0:1` numerator in
/// FFmpeg means "unknown"; callers normalise that to the `1:1`
/// default (square pixels) before constructing this type.
///
/// `den` is a [`core::num::NonZeroI64`] so a SAR can never have a
/// zero denominator, and the numerator cannot be negative; the manual
/// [`Default`] is `1:1` (square), mirroring `mediatime::Timebase`'s
/// non-proto-zero default.
///
/// Represented as a newtype over [`Rational`] — the single source of
/// truth for "exact ratio with a non-zero denominator". The fields
/// are private; the entire public method API (and the `buffa` wire
/// format) is unchanged, delegating to the inner `Rational`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::sample_aspect_ratio")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleAspectRatio(Rational);

impl Default for SampleAspectRatio {
  /// `1:1` — square pixels.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self(Rational::default())
  }
}

impl SampleAspectRatio {
  /// Constructs a `SampleAspectRatio` from an explicit
  /// numerator / (non-zero) denominator.
  ///
  /// # Panics
  ///
  /// Panics if `num < 0` or `den < 0`, as [`Rational::new`] does. For
  /// a fallible path, build the ratio with [`Rational::try_new`] and
  /// wrap it — `SampleAspectRatio` is `From<Rational>`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(num: i64, den: core::num::NonZeroI64) -> Self {
    Self(Rational::new(num, den))
  }

  /// Returns the numerator (display-width units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn num(&self) -> i64 {
    self.0.num()
  }

  /// Returns the (non-zero) denominator (display-height units).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn den(&self) -> core::num::NonZeroI64 {
    self.0.den()
  }

  /// `true` when the pixels are square (`num == den`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_square(&self) -> bool {
    self.0.num() == self.0.den().get()
  }

  /// Returns this SAR as a generic [`Rational`] — the underlying
  /// representation. Purely additive interop; `SampleAspectRatio`'s
  /// public method API is unchanged.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rational(&self) -> Rational {
    self.0
  }

  /// Alias of [`Self::rational`] — views this SAR as a generic
  /// [`Rational`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_rational(&self) -> Rational {
    self.rational()
  }

  /// Sets the numerator (consuming builder).
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Rational::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_num(mut self, num: i64) -> Self {
    self.0 = self.0.with_num(num);
    self
  }

  /// Sets the denominator (consuming builder).
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Rational::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_den(mut self, den: core::num::NonZeroI64) -> Self {
    self.0 = self.0.with_den(den);
    self
  }

  /// Sets the numerator in place.
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Rational::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_num(&mut self, num: i64) -> &mut Self {
    self.0.set_num(num);
    self
  }

  /// Sets the denominator in place.
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Rational::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_den(&mut self, den: core::num::NonZeroI64) -> &mut Self {
    self.0.set_den(den);
    self
  }
}

impl core::fmt::Display for SampleAspectRatio {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}:{}", self.0.num(), self.0.den())
  }
}

impl core::str::FromStr for SampleAspectRatio {
  type Err = ParseSampleAspectRatioError;

  /// Parses the `NUM:DEN` form [`Display`](core::fmt::Display) renders
  /// (`"40:33"`). The separator is a colon, not the `/` [`Rational`]
  /// uses — an aspect ratio is conventionally written `a:b`, and the
  /// two spellings stay distinguishable.
  ///
  /// # Errors
  ///
  /// Returns [`ParseSampleAspectRatioError`] when the input is not two
  /// `i64` values separated by a single `:`, or when the pair violates
  /// [`Rational::try_new`]'s invariant (`num >= 0`, `den > 0`).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_ratio(s, ':')
      .map(Self)
      .map_err(|kind| ParseSampleAspectRatioError { kind })
  }
}

impl From<SampleAspectRatio> for Rational {
  /// Unwraps the inner [`Rational`] — `SampleAspectRatio` is a newtype
  /// over `Rational`. Additive interop; `SampleAspectRatio`'s own
  /// public method API is unchanged.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(sar: SampleAspectRatio) -> Self {
    sar.0
  }
}

impl From<Rational> for SampleAspectRatio {
  /// Wraps a generic [`Rational`] as a pixel/sample aspect ratio.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn from(rate: Rational) -> Self {
    Self(rate)
  }
}

/// `NonZeroI64` for 1 — [`Rational`]'s default denominator, and the
/// clamp target when a malformed denominator arrives on the wire.
///
/// Spelled out rather than reached for as `NonZeroI64::MIN`, which is
/// `i64::MIN` — a value [`Rational::new`] rejects. (The pre-`i64`
/// `NonZeroU32::MIN` *was* `1`, so the swap is a silent trap.)
pub(crate) const DEN_ONE: core::num::NonZeroI64 = match core::num::NonZeroI64::new(1) {
  Some(v) => v,
  None => unreachable!(),
};

/// A generic exact ratio `num / den` — a non-negative numerator over a
/// strictly positive denominator.
///
/// The reusable rational primitive the rest of the frame layer builds
/// on (e.g. [`FrameRate`]). `den` is a [`core::num::NonZeroI64`] so a
/// ratio can never have a zero denominator, and [`Self::new`] rejects
/// the negative half of both fields; the manual [`Default`] is `1/1`
/// (the multiplicative identity), mirroring [`SampleAspectRatio`]'s
/// non-proto-zero default and `mediatime::Timebase`'s convention.
///
/// This is the format-agnostic numerator/denominator pair; semantic
/// wrappers ([`SampleAspectRatio`] for pixel aspect, [`FrameRate`] for
/// frames-per-second) carry the domain meaning. A `0` numerator is a
/// valid representable state (e.g. an "unknown" FFmpeg `AVRational`
/// `0/1`) — see [`Self::is_zero`].
///
/// # Why `i64` and not `mediatime::Timebase`'s `i32`
///
/// `mediatime::Timebase` is signed 32-bit because it must round-trip
/// into an FFmpeg `AVRational` (`{int num; int den;}`) and because it
/// is an *arithmetic operand*: it multiplies against `i64` PTS values,
/// and the rescale overflow proofs need `num < 2^32` to keep the
/// product inside `i128`.
///
/// Neither applies here. `mediaframe` is a pure receiver — nothing in
/// it is handed back to a decoder SDK — and a `Rational` never
/// multiplies against a PTS, so it carries no width proof. What does
/// apply is the storage end (`sqlx` has no `Type<Postgres>` for `u32`,
/// so a `u32` widens to `i64` to be stored regardless) and the ingest
/// end (R3D metadata returns `unsigned int`, and ISO BMFF `pasp` is
/// `unsigned int(32)`, both of which `i32` would have to *reject*).
/// `i64` is a superset of every source and deletes a conversion with
/// its error path at each boundary.
///
/// Values are stored exactly as declared — the constructor does **not**
/// reduce to lowest terms, so a stream declaring `2/4` reads back as
/// `2/4`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::rational")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
  #[cfg_attr(feature = "serde", serde(deserialize_with = "de_num"))]
  num: i64,
  #[cfg_attr(feature = "serde", serde(deserialize_with = "de_den"))]
  den: core::num::NonZeroI64,
}

impl Default for Rational {
  /// `1/1` — the multiplicative identity.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self {
      num: 1,
      den: DEN_ONE,
    }
  }
}

impl Rational {
  /// Constructs a `Rational` from an explicit
  /// numerator / (non-zero) denominator.
  ///
  /// # Panics
  ///
  /// - Panics if `num < 0` (a negative ratio is meaningless for the
  ///   aspect / rate domains this primitive serves).
  /// - Panics if `den <= 0` (`NonZeroI64` rules out zero; this rules
  ///   out the negative denominators an `AVRational` would tolerate).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(num: i64, den: core::num::NonZeroI64) -> Self {
    assert!(num >= 0, "rational numerator must not be negative");
    assert!(den.get() > 0, "rational denominator must be positive");

    Self { num, den }
  }

  /// Fallible variant of [`Self::new`]: returns `None` instead of
  /// panicking when `num < 0` or `den < 0`. Accepts `num == 0` (the
  /// "unknown" `0/1` `AVRational`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn try_new(num: i64, den: core::num::NonZeroI64) -> Option<Self> {
    if num >= 0 && den.get() > 0 {
      Some(Self { num, den })
    } else {
      None
    }
  }

  /// Returns the numerator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn num(&self) -> i64 {
    self.num
  }

  /// Returns the (non-zero) denominator.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn den(&self) -> core::num::NonZeroI64 {
    self.den
  }

  /// `true` when the numerator is `0` (the ratio is exactly zero —
  /// e.g. an "unknown" `0/1` FFmpeg `AVRational`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_zero(&self) -> bool {
    self.num == 0
  }

  /// Sets the numerator (consuming builder).
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Self::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_num(mut self, num: i64) -> Self {
    self.set_num(num);
    self
  }

  /// Sets the denominator (consuming builder).
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Self::new`] does.
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_den(mut self, den: core::num::NonZeroI64) -> Self {
    self.set_den(den);
    self
  }

  /// Sets the numerator in place.
  ///
  /// # Panics
  ///
  /// Panics if `num < 0`, as [`Self::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_num(&mut self, num: i64) -> &mut Self {
    // Routed through the constructor so the sign invariants have
    // exactly one enforcement site. Direct field assignment would let
    // a mutator mint the negative values `new` refuses.
    *self = Self::new(num, self.den);
    self
  }

  /// Sets the denominator in place.
  ///
  /// # Panics
  ///
  /// Panics if `den < 0`, as [`Self::new`] does.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_den(&mut self, den: core::num::NonZeroI64) -> &mut Self {
    *self = Self::new(self.num, den);
    self
  }
}

/// Field validators for [`Rational`]'s derived `Deserialize`.
///
/// The derive assigns fields directly, bypassing [`Rational::new`].
/// While the fields were `u32`/`NonZeroU32` their types made every
/// invariant violation unrepresentable; `i64`/`NonZeroI64` no longer
/// do, so deserialization would otherwise be a second construction
/// path able to mint a `Rational` the constructor rejects. The two
/// invariants are independent per field, so a `deserialize_with` on
/// each is enough — no intermediate representation and no allocation.
/// [`SampleAspectRatio`] and [`FrameRate`] derive through `Rational`,
/// so guarding it guards them.
#[cfg(feature = "serde")]
mod de {
  use core::num::NonZeroI64;

  use serde::{Deserialize, Deserializer, de::Error};

  pub(super) fn de_num<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    let v = i64::deserialize(d)?;
    if v < 0 {
      return Err(D::Error::custom("rational numerator must not be negative"));
    }
    Ok(v)
  }

  pub(super) fn de_den<'de, D: Deserializer<'de>>(d: D) -> Result<NonZeroI64, D::Error> {
    let v = NonZeroI64::deserialize(d)?;
    if v.get() < 0 {
      return Err(D::Error::custom("rational denominator must be positive"));
    }
    Ok(v)
  }
}

#[cfg(feature = "serde")]
use de::{de_den, de_num};

impl core::fmt::Display for Rational {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}/{}", self.num, self.den)
  }
}

/// Splits `NUM<sep>DEN` and runs the pair through [`Rational::try_new`], so
/// parsing cannot become a second construction path that mints a ratio the
/// constructor would reject.
/// Why a ratio spelling was rejected.
///
/// Public because the two cases are genuinely different: a caller
/// forwarding user input wants to say "that is not a ratio" for the first
/// and "that ratio is not representable" for the second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RatioParseKind {
  /// The input does not have the shape at all — no separator, a
  /// non-numeric component, or trailing text.
  Malformed,
  /// The input parsed, but the pair violates [`Rational::try_new`]'s
  /// invariant (`num >= 0`, `den > 0`).
  OutOfRange,
}

impl core::fmt::Display for RatioParseKind {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str(match self {
      Self::Malformed => "malformed",
      Self::OutOfRange => "value out of range",
    })
  }
}

/// The error [`Rational`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Carries [`RatioParseKind`] — unlike the name vocabularies, a ratio can
/// fail for two reasons a caller reports differently. The rejected input
/// itself is not retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a NUM/DEN rational: {kind}")]
pub struct ParseRationalError {
  kind: RatioParseKind,
}

impl ParseRationalError {
  /// Why the input was rejected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn kind(&self) -> RatioParseKind {
    self.kind
  }
}

/// The error [`SampleAspectRatio`]'s [`FromStr`](core::str::FromStr)
/// returns. Same two cases as [`ParseRationalError`], over the `NUM:DEN`
/// spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a NUM:DEN sample aspect ratio: {kind}")]
pub struct ParseSampleAspectRatioError {
  kind: RatioParseKind,
}

impl ParseSampleAspectRatioError {
  /// Why the input was rejected.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn kind(&self) -> RatioParseKind {
    self.kind
  }
}

fn parse_ratio(s: &str, sep: char) -> Result<Rational, RatioParseKind> {
  let (n, d) = s.split_once(sep).ok_or(RatioParseKind::Malformed)?;
  let num: i64 = n.parse().map_err(|_| RatioParseKind::Malformed)?;
  let den: i64 = d.parse().map_err(|_| RatioParseKind::Malformed)?;
  let den = core::num::NonZeroI64::new(den).ok_or(RatioParseKind::OutOfRange)?;
  Rational::try_new(num, den).ok_or(RatioParseKind::OutOfRange)
}

impl core::str::FromStr for Rational {
  type Err = ParseRationalError;

  /// Parses the `NUM/DEN` form [`Display`](core::fmt::Display) renders
  /// (`"30000/1001"`).
  ///
  /// # Errors
  ///
  /// Returns [`ParseRationalError`] when the input is not two `i64`
  /// values separated by a single `/`, or when the pair violates
  /// [`Rational::try_new`]'s invariant (`num >= 0`, `den > 0`).
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    parse_ratio(s, '/').map_err(|kind| ParseRationalError { kind })
  }
}

/// The frame rate of a video stream as an exact [`Rational`]
/// (frames per second) plus a variable-frame-rate marker.
///
/// `rate` is the nominal frames-per-second ratio (e.g. `30000/1001`
/// for NTSC, `25/1` for PAL). `is_vfr` records that the stream is
/// variable-frame-rate, in which case `rate` is the average / nominal
/// rate only and per-frame timing must be taken from the timestamps.
///
/// This is deliberately **not** [`mediatime::Timebase`]: a frame rate
/// is *not* a presentation-timestamp timebase. They are reciprocal-ish
/// but distinct concepts (a 30000/1001 fps stream is commonly carried
/// on a 1/90000 or 1/1000 PTS timebase) — `mediatime` documents that
/// distinction and intentionally models only the PTS timebase, so the
/// frame-rate concept lives here as its own type.
///
/// The [`Default`] is `{ rate: Rational::default() (1/1),
/// is_vfr: false }`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::frame_rate")
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate {
  rate: Rational,
  is_vfr: bool,
}

impl FrameRate {
  /// Constructs a `FrameRate` from an exact frames-per-second
  /// [`Rational`] and a variable-frame-rate flag.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(rate: Rational, is_vfr: bool) -> Self {
    Self { rate, is_vfr }
  }

  /// Returns the nominal frames-per-second ratio.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn rate(&self) -> Rational {
    self.rate
  }

  /// `true` when the stream is variable-frame-rate (the [`Self::rate`]
  /// is then an average / nominal value only).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn is_vfr(&self) -> bool {
    self.is_vfr
  }

  /// Sets the rate (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_rate(mut self, rate: Rational) -> Self {
    self.rate = rate;
    self
  }

  /// Marks the stream variable-frame-rate (`is_vfr = true`; consuming
  /// builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_is_vfr(mut self) -> Self {
    self.is_vfr = true;
    self
  }

  /// Assigns the raw VFR flag (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_is_vfr(mut self, is_vfr: bool) -> Self {
    self.is_vfr = is_vfr;
    self
  }

  /// Sets the rate in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_rate(&mut self, rate: Rational) -> &mut Self {
    self.rate = rate;
    self
  }

  /// Marks the stream variable-frame-rate (`is_vfr = true`) in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_is_vfr(&mut self) -> &mut Self {
    self.is_vfr = true;
    self
  }

  /// Assigns the raw VFR flag in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_is_vfr(&mut self, is_vfr: bool) -> &mut Self {
    self.is_vfr = is_vfr;
    self
  }

  /// Clears the VFR flag (`is_vfr = false`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_is_vfr(&mut self) -> &mut Self {
    self.is_vfr = false;
    self
  }
}

/// Interlacing / field order of a video stream.
///
/// Mirrors FFmpeg `AVFieldOrder`
/// (`AVCodecContext::field_order` / `AVFrame` derived state) with the
/// exact numeric code points: `AV_FIELD_UNKNOWN = 0`,
/// `AV_FIELD_PROGRESSIVE = 1`, `AV_FIELD_TT = 2`,
/// `AV_FIELD_BB = 3`, `AV_FIELD_TB = 4`, `AV_FIELD_BT = 5`. Any
/// other / future / corrupt wire value is preserved verbatim as
/// [`None`] from [`Self::from_u32`] rather than collapsed; a *name*
/// this build does not enumerate rides [`Self::Other`].
///
/// FFmpeg's own `AV_FIELD_UNKNOWN` sentinel is code `0`, so the
/// [`Default`] is the named [`Self::Unknown`] — the same
/// FFmpeg-names-its-own-absence precedent as
/// [`PixelFormat::None`](crate::pixel_format::PixelFormat::None).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::field_order")
)]
pub enum FieldOrder {
  /// Field order not known — FFmpeg's own `AV_FIELD_UNKNOWN` (code `0`),
  /// and the [`Default`].
  ///
  /// A **named** member of the FFmpeg vocabulary, not an escape arm: it
  /// carries no payload, owns the slug `"unknown"`, and round-trips
  /// exactly. "The container did not say" is a field order a stream can
  /// state, and it is the state a freshly-defaulted descriptor is in.
  Unknown,
  /// Progressive (not interlaced) — `AV_FIELD_PROGRESSIVE`.
  Progressive,
  /// Top coded first, top displayed first — `AV_FIELD_TT`.
  Tt,
  /// Bottom coded first, bottom displayed first — `AV_FIELD_BB`.
  Bb,
  /// Top coded first, bottom displayed first — `AV_FIELD_TB`.
  Tb,
  /// Bottom coded first, top displayed first — `AV_FIELD_BT`.
  Bt,
  /// A slug this vocabulary does not enumerate — carried verbatim,
  /// ASCII-folded to lowercase by the parse gate. The crate-wide
  /// extension idiom: a downstream backend naming a value mediaframe
  /// has never heard of keeps that **name**, and it round-trips through
  /// `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(SmolStr),
}

impl Default for FieldOrder {
  /// [`Self::Unknown`] — FFmpeg's `AV_FIELD_UNKNOWN`, code `0`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Unknown
  }
}

impl FieldOrder {
  /// Lowercase slug for this field order (`"progressive"` / `"tt"` /
  /// `"bb"` / `"tb"` / `"bt"`); [`Self::Unknown`] is FFmpeg's own named
  /// `"unknown"`, and [`Self::Other`] renders the name it carries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Unknown => "unknown",
      Self::Progressive => "progressive",
      Self::Tt => "tt",
      Self::Bb => "bb",
      Self::Tb => "tb",
      Self::Bt => "bt",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id = the FFmpeg `AVFieldOrder` code
  /// (`Unknown`=0, `Progressive`=1, `Tt`=2, `Bb`=3, `Tb`=4, `Bt`=5).
  ///
  /// [`None`] for [`Self::Other`]: FFmpeg has no code for a name it
  /// does not know.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Unknown => 0,
      Self::Progressive => 1,
      Self::Tt => 2,
      Self::Bb => 3,
      Self::Tb => 4,
      Self::Bt => 5,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVFieldOrder` code produced by
  /// [`Self::to_u32`]. Code `0` is FFmpeg's own `AV_FIELD_UNKNOWN` and
  /// decodes to the named [`Self::Unknown`]; any other unrecognised id
  /// yields [`None`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::Unknown,
      1 => Self::Progressive,
      2 => Self::Tt,
      3 => Self::Bb,
      4 => Self::Tb,
      5 => Self::Bt,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name, ASCII-folded
  /// to the crate's lowercase canon.
  ///
  /// The **one** construction path for [`Self::Other`]: folding here is what
  /// keeps the whole value space lowercase-canonical, so the derived `Eq` /
  /// `Hash` compare names rather than spellings. Constructing the variant
  /// directly bypasses the fold and is not the supported spelling.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::Other(crate::parse::fold_owned(slug.as_ref()))
  }
}

/// The error [`FieldOrder`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Opaque and sealed: the input is deliberately not retained (these types
/// are available at the crate's no-alloc tier, where there is nowhere to
/// put an owned copy, and the input is attacker-controlled on the
/// deserialization path). `#[non_exhaustive]` keeps it constructible only
/// here, so it can grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a field-order name")]
#[non_exhaustive]
pub struct ParseFieldOrderError;

impl core::str::FromStr for FieldOrder {
  type Err = ParseFieldOrderError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseError`](crate::parse::ParseError) only at the
  /// no-alloc tier, where the vocabulary is closed. With `alloc` this
  /// parse is **total**: a slug this type does not name rides
  /// [`Self::Other`], ASCII-folded to lowercase by [`Self::other`].
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s);
    Ok(match folded {
      "unknown" => Self::Unknown,
      "progressive" => Self::Progressive,
      "tt" => Self::Tt,
      "bb" => Self::Bb,
      "tb" => Self::Tb,
      "bt" => Self::Bt,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::other(s),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseFieldOrderError),
    })
  }
}

/// Stereoscopic-3D packing mode of a video stream.
///
/// Mirrors FFmpeg `AVStereo3DType` (the `AV_FRAME_DATA_STEREO3D`
/// side-data `type`) with the exact numeric code points:
/// `AV_STEREO3D_2D = 0` (named [`Self::Mono`]),
/// `AV_STEREO3D_SIDEBYSIDE = 1`, `AV_STEREO3D_TOPBOTTOM = 2`,
/// `AV_STEREO3D_FRAMESEQUENCE = 3`, `AV_STEREO3D_CHECKERBOARD = 4`,
/// `AV_STEREO3D_SIDEBYSIDE_QUINCUNX = 5`, `AV_STEREO3D_LINES = 6`,
/// `AV_STEREO3D_COLUMNS = 7`. Any other / future / corrupt wire
/// value is **rejected** by [`Self::from_u32`]; a *name* this build
/// does not enumerate rides [`Self::Other`], the crate-wide extension
/// idiom shared with [`Rotation`] / the colour enums.
///
/// The [`Default`] is [`Self::Mono`] — a *real* code (value `0`,
/// FFmpeg `AV_STEREO3D_2D`, plain monoscopic video), so the default
/// is a real mode rather than an absence (the colour-enum named-default
/// precedent, e.g. `DcpTargetGamut::DciP3`), distinct from
/// [`FieldOrder`] whose `0` *is* FFmpeg's UNKNOWN sentinel.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, IsVariant)]
#[display("{}", self.as_str())]
#[non_exhaustive]
#[cfg_attr(
  feature = "quickcheck",
  derive(::quickcheck_richderive::Arbitrary),
  quickcheck(arbitrary = "crate::quickcheck_helpers::coded::stereo_mode")
)]
pub enum StereoMode {
  /// Plain monoscopic (non-stereo) video — `AV_STEREO3D_2D` (code
  /// `0`). The [`Default`].
  Mono,
  /// Side-by-side — `AV_STEREO3D_SIDEBYSIDE`.
  SideBySide,
  /// Top-bottom — `AV_STEREO3D_TOPBOTTOM`.
  TopBottom,
  /// Frame-sequential — `AV_STEREO3D_FRAMESEQUENCE`.
  FrameSequence,
  /// Checkerboard — `AV_STEREO3D_CHECKERBOARD`.
  Checkerboard,
  /// Side-by-side quincunx — `AV_STEREO3D_SIDEBYSIDE_QUINCUNX`.
  SideBySideQuincunx,
  /// Interleaved by rows — `AV_STEREO3D_LINES`.
  Lines,
  /// Interleaved by columns — `AV_STEREO3D_COLUMNS`.
  Columns,
  /// A slug this vocabulary does not enumerate — carried verbatim,
  /// ASCII-folded to lowercase by the parse gate. The crate-wide
  /// extension idiom: a downstream backend naming a value mediaframe
  /// has never heard of keeps that **name**, and it round-trips through
  /// `as_str` / `FromStr` / `serde` intact.
  ///
  /// Requires the `alloc` feature (`std` includes it) — the payload is
  /// heap-capable. At the no-alloc tier the vocabulary is closed and an
  /// unrecognised slug is rejected instead.
  #[cfg(any(feature = "std", feature = "alloc"))]
  Other(SmolStr),
}

impl Default for StereoMode {
  /// [`Self::Mono`] — FFmpeg `AV_STEREO3D_2D` (code `0`), plain
  /// monoscopic video — a real mode, not an absence; the colour-enum
  /// named-default precedent.
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn default() -> Self {
    Self::Mono
  }
}

impl StereoMode {
  /// Lowercase slug for this stereo mode; [`Self::Other`] renders the
  /// name it carries.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Mono => "mono",
      Self::SideBySide => "side-by-side",
      Self::TopBottom => "top-bottom",
      Self::FrameSequence => "frame-sequence",
      Self::Checkerboard => "checkerboard",
      Self::SideBySideQuincunx => "side-by-side-quincunx",
      Self::Lines => "lines",
      Self::Columns => "columns",
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(s) => s.as_str(),
    }
  }

  /// Stable `u32` wire id = the FFmpeg `AVStereo3DType` code
  /// (`Mono`=0, `SideBySide`=1, `TopBottom`=2, `FrameSequence`=3,
  /// `Checkerboard`=4, `SideBySideQuincunx`=5, `Lines`=6,
  /// `Columns`=7).
  ///
  /// [`None`] for [`Self::Other`]: FFmpeg has no code for a name it
  /// does not know.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn to_u32(&self) -> Option<u32> {
    Some(match self {
      Self::Mono => 0,
      Self::SideBySide => 1,
      Self::TopBottom => 2,
      Self::FrameSequence => 3,
      Self::Checkerboard => 4,
      Self::SideBySideQuincunx => 5,
      Self::Lines => 6,
      Self::Columns => 7,
      #[cfg(any(feature = "std", feature = "alloc"))]
      Self::Other(_) => return None,
    })
  }

  /// Decodes from the FFmpeg `AVStereo3DType` code produced by
  /// [`Self::to_u32`]. The canonical codes map to their named variants;
  /// any other id yields [`None`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn from_u32(v: u32) -> Option<Self> {
    Some(match v {
      0 => Self::Mono,
      1 => Self::SideBySide,
      2 => Self::TopBottom,
      3 => Self::FrameSequence,
      4 => Self::Checkerboard,
      5 => Self::SideBySideQuincunx,
      6 => Self::Lines,
      7 => Self::Columns,
      _ => return None,
    })
  }
  /// The open escape for a slug this vocabulary does not name, ASCII-folded
  /// to the crate's lowercase canon.
  ///
  /// The **one** construction path for [`Self::Other`]: folding here is what
  /// keeps the whole value space lowercase-canonical, so the derived `Eq` /
  /// `Hash` compare names rather than spellings. Constructing the variant
  /// directly bypasses the fold and is not the supported spelling.
  #[cfg(any(feature = "std", feature = "alloc"))]
  pub fn other(slug: impl AsRef<str>) -> Self {
    Self::Other(crate::parse::fold_owned(slug.as_ref()))
  }
}

/// The error [`StereoMode`]'s [`FromStr`](core::str::FromStr) returns.
///
/// Opaque and sealed: the input is deliberately not retained (these types
/// are available at the crate's no-alloc tier, where there is nowhere to
/// put an owned copy, and the input is attacker-controlled on the
/// deserialization path). `#[non_exhaustive]` keeps it constructible only
/// here, so it can grow structure later without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("not a stereo-mode name")]
#[non_exhaustive]
pub struct ParseStereoModeError;

impl core::str::FromStr for StereoMode {
  type Err = ParseStereoModeError;

  /// Parses the canonical slug [`Self::as_str`] renders, the exact
  /// inverse of [`Display`](core::fmt::Display) for every **named**
  /// variant.
  ///
  /// # Errors
  ///
  /// Returns [`ParseError`](crate::parse::ParseError) only at the
  /// no-alloc tier, where the vocabulary is closed. With `alloc` this
  /// parse is **total**: a slug this type does not name rides
  /// [`Self::Other`], ASCII-folded to lowercase by [`Self::other`].
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut buf = [0u8; crate::parse::FOLD_CAP];
    // An input too long to fold cannot name a variant either, so the
    // unfolded original falls through to the miss arm.
    let folded = crate::parse::fold(s, &mut buf).unwrap_or(s);
    Ok(match folded {
      "mono" => Self::Mono,
      "side-by-side" => Self::SideBySide,
      "top-bottom" => Self::TopBottom,
      "frame-sequence" => Self::FrameSequence,
      "checkerboard" => Self::Checkerboard,
      "side-by-side-quincunx" => Self::SideBySideQuincunx,
      "lines" => Self::Lines,
      "columns" => Self::Columns,
      #[cfg(any(feature = "std", feature = "alloc"))]
      _ => Self::other(s),
      #[cfg(not(any(feature = "std", feature = "alloc")))]
      _ => return Err(ParseStereoModeError),
    })
  }
}

/// One plane of pixel data.
///
/// Generic over the buffer type `B` so the same `Plane` shape works
/// for owned (`Vec<u8>`, `bytes::Bytes`), borrowed (`&'a [u8]`), or
/// custom backend-supplied buffers. The bound `B: AsRef<[u8]>` lives
/// at the use site (`VideoFrame<P, B: AsRef<[u8]>, …>`); `Plane` itself
/// is unbounded so it can be used in const contexts.
///
/// `stride` is bytes per row for video planes, or total plane size
/// in bytes for audio planar formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Plane<B> {
  data: B,
  stride: u32,
}

impl<B> Plane<B> {
  /// Constructs a `Plane` from a buffer and a stride.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(data: B, stride: u32) -> Self {
    Self { data, stride }
  }

  /// Returns the stride in bytes.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn stride(&self) -> u32 {
    self.stride
  }

  /// Borrows the underlying buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn data_ref(&self) -> &B {
    &self.data
  }

  /// Mutably borrows the underlying buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn data_mut(&mut self) -> &mut B {
    &mut self.data
  }

  /// Consumes the plane and returns the underlying buffer.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_data(self) -> B {
    self.data
  }

  /// Sets the stride (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_stride(mut self, stride: u32) -> Self {
    self.stride = stride;
    self
  }

  /// Sets the stride in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_stride(&mut self, stride: u32) -> &mut Self {
    self.stride = stride;
    self
  }
}

/// A runtime-tagged video frame.
///
/// Generic parameters:
/// - `P` — pixel-format identifier. Typically [`crate::pixel_format::PixelFormat`]
///   in mediadecode-style runtime-tagged pipelines, but `P` is left unbounded
///   so backends can substitute a richer type (e.g. an FFmpeg
///   `AVPixelFormat` newtype that round-trips to `PixelFormat`).
/// - `B` — plane data buffer type. Each populated `Plane<B>` carries one
///   plane's bytes; `B: AsRef<[u8]>` at the consumer (e.g. `&'a [u8]`,
///   `Vec<u8>`, `bytes::Bytes`, refcounted FFmpeg buffer).
///
/// `dimensions` is the **coded** width / height; [`Self::visible_rect`]
/// (when present) is the displayable subregion (FFmpeg crop /
/// WebCodecs `visibleRect` / ProRes RAW `CleanAperture`).
///
/// `plane_count` is the number of populated entries in `planes`. Four
/// slots cover every realistic format: NV12 = 2, YUV420P = 3, YUVA /
/// packed-with-alpha = 4, packed RGB / Bayer CFA = 1.
///
/// **No timestamp.** PTS / duration ride on the orthogonal
/// [`TimestampedFrame<F>`] wrapper so the pixel-data layer stays
/// independent of the timekeeping layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoFrame<P, B> {
  dimensions: Dimensions,
  visible_rect: Option<Rect>,
  pixel_format: P,
  plane_count: u8,
  planes: [Plane<B>; 4],
  color: crate::color::Info,
}

impl<P, B> VideoFrame<P, B> {
  /// Constructs a `VideoFrame`. `visible_rect` defaults to `None`,
  /// color to `Info::UNSPECIFIED`.
  ///
  /// # Panics
  ///
  /// Panics if `plane_count > 4`. The fixed-size `planes` array has
  /// four slots; passing a larger `plane_count` would later trip
  /// slice indexing inside [`Self::planes`] far from the
  /// construction site. Asserting here fails fast.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(
    dimensions: Dimensions,
    pixel_format: P,
    planes: [Plane<B>; 4],
    plane_count: u8,
  ) -> Self {
    assert!(
      plane_count as usize <= 4,
      "VideoFrame::new: plane_count exceeds the fixed 4-plane array",
    );
    Self {
      dimensions,
      visible_rect: None,
      pixel_format,
      plane_count,
      planes,
      color: crate::color::Info::UNSPECIFIED,
    }
  }

  /// Returns the coded dimensions.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn dimensions(&self) -> Dimensions {
    self.dimensions
  }

  /// Returns the coded width (shortcut for `dimensions().width()`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn width(&self) -> u32 {
    self.dimensions.width()
  }

  /// Returns the coded height (shortcut for `dimensions().height()`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn height(&self) -> u32 {
    self.dimensions.height()
  }

  /// Returns the visible / clean-aperture rectangle, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn visible_rect(&self) -> Option<Rect> {
    self.visible_rect
  }

  /// Returns a reference to the pixel-format identifier.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn pixel_format_ref(&self) -> &P {
    &self.pixel_format
  }

  /// Returns the populated plane count.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn plane_count(&self) -> u8 {
    self.plane_count
  }

  /// Returns the populated planes as a slice.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn planes(&self) -> &[Plane<B>] {
    &self.planes[..self.plane_count as usize]
  }

  /// Returns one plane by index, or `None` if out of range.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn plane(&self, i: usize) -> Option<&Plane<B>> {
    if i < self.plane_count as usize {
      self.planes.get(i)
    } else {
      None
    }
  }

  /// Returns the color metadata.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn color(&self) -> crate::color::Info {
    self.color.clone()
  }

  /// Sets the visible rect to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_visible_rect(mut self, v: Rect) -> Self {
    self.visible_rect = Some(v);
    self
  }

  /// Assigns the raw visible-rect wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_visible_rect(mut self, v: Option<Rect>) -> Self {
    self.visible_rect = v;
    self
  }

  /// Sets the color metadata (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn with_color(mut self, v: crate::color::Info) -> Self {
    self.color = v;
    self
  }

  /// Sets the visible rect to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_visible_rect(&mut self, v: Rect) -> &mut Self {
    self.visible_rect = Some(v);
    self
  }

  /// Assigns the raw visible-rect wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_visible_rect(&mut self, v: Option<Rect>) -> &mut Self {
    self.visible_rect = v;
    self
  }

  /// Clears the visible rect (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_visible_rect(&mut self) -> &mut Self {
    self.visible_rect = None;
    self
  }

  /// Sets the color metadata in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_color(&mut self, v: crate::color::Info) -> &mut Self {
    self.color = v;
    self
  }
}

/// Wraps any inner `F` with optional PTS + duration timestamps.
///
/// This is the orthogonal time-carrying layer. The inner `F` stays
/// pure pixel data — `VideoFrame<P, B>` for runtime-tagged decoder
/// output, or a colconv-typed `Yuv420pFrame<'a, BE>` borrow type for
/// zero-copy conversion pipelines. Composition rather than inheritance
/// keeps the mediaframe data layer independent of any timekeeping
/// convention.
///
/// Timestamps use [`mediatime::Timestamp`], a rational-time type from
/// the `mediatime` crate (no_std, zero deps, exact arithmetic). Both
/// PTS and duration are `Option` because backends do not always know
/// them.
///
/// `duration` is deliberately the **same** `mediatime::Timestamp`
/// (timebase ticks) as `pts`, mirroring FFmpeg's `AVFrame.duration`
/// — an `int64` in the stream `time_base`, *not* a wall-clock value.
/// It is intentionally **not** a `core::time::Duration`: that would
/// lose exact rational-timebase precision and diverge from the
/// FFmpeg / `mediatime` model this crate faithfully mirrors. (Codex
/// adversarial-review F2 — reviewed and intentionally kept as-is.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimestampedFrame<F> {
  pts: Option<mediatime::Timestamp>,
  // Timebase ticks, like FFmpeg `AVFrame.duration` — see type doc.
  duration: Option<mediatime::Timestamp>,
  frame: F,
}

impl<F> TimestampedFrame<F> {
  /// Constructs a `TimestampedFrame`. PTS and duration default to
  /// `None`.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new(frame: F) -> Self {
    Self {
      pts: None,
      duration: None,
      frame,
    }
  }

  /// Returns the presentation timestamp, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn pts(&self) -> Option<mediatime::Timestamp> {
    self.pts
  }

  /// Returns the duration, if any.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn duration(&self) -> Option<mediatime::Timestamp> {
    self.duration
  }

  /// Borrows the inner frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn frame_ref(&self) -> &F {
    &self.frame
  }

  /// Mutably borrows the inner frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn frame_mut(&mut self) -> &mut F {
    &mut self.frame
  }

  /// Consumes the wrapper and returns the inner frame.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn into_frame(self) -> F {
    self.frame
  }

  /// Sets the PTS to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_pts(mut self, v: mediatime::Timestamp) -> Self {
    self.pts = Some(v);
    self
  }

  /// Assigns the raw PTS wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_pts(mut self, v: Option<mediatime::Timestamp>) -> Self {
    self.pts = v;
    self
  }

  /// Sets the duration to `Some(v)` (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn with_duration(mut self, v: mediatime::Timestamp) -> Self {
    self.duration = Some(v);
    self
  }

  /// Assigns the raw duration wrapper (consuming builder).
  #[must_use]
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn maybe_duration(mut self, v: Option<mediatime::Timestamp>) -> Self {
    self.duration = v;
    self
  }

  /// Sets the PTS to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_pts(&mut self, v: mediatime::Timestamp) -> &mut Self {
    self.pts = Some(v);
    self
  }

  /// Assigns the raw PTS wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_pts(&mut self, v: Option<mediatime::Timestamp>) -> &mut Self {
    self.pts = v;
    self
  }

  /// Clears the PTS (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_pts(&mut self) -> &mut Self {
    self.pts = None;
    self
  }

  /// Sets the duration to `Some(v)` in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn set_duration(&mut self, v: mediatime::Timestamp) -> &mut Self {
    self.duration = Some(v);
    self
  }

  /// Assigns the raw duration wrapper in place.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn update_duration(&mut self, v: Option<mediatime::Timestamp>) -> &mut Self {
    self.duration = v;
    self
  }

  /// Clears the duration (`None`).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear_duration(&mut self) -> &mut Self {
    self.duration = None;
    self
  }
}

// === Per-family Frame modules (feature-gated) ===

#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod planar_8bit;
#[cfg(feature = "yuv-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-planar")))]
mod subsampled_high_bit_planar;
use derive_more::{Display, IsVariant};
#[cfg(feature = "yuv-planar")]
pub use planar_8bit::*;
#[cfg(any(feature = "std", feature = "alloc"))]
use smol_str::SmolStr;
#[cfg(feature = "yuv-planar")]
pub use subsampled_high_bit_planar::*;

#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod nv20;
#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod semi_planar_8bit;
#[cfg(feature = "yuv-semi-planar")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-semi-planar")))]
mod subsampled_high_bit_pn;
#[cfg(feature = "yuv-semi-planar")]
pub use nv20::*;
#[cfg(feature = "yuv-semi-planar")]
pub use semi_planar_8bit::*;
#[cfg(feature = "yuv-semi-planar")]
pub use subsampled_high_bit_pn::*;

#[cfg(feature = "yuva")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuva")))]
mod yuva;
#[cfg(feature = "yuva")]
pub use yuva::*;

#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod packed_yuv_4_1_1;
#[cfg(feature = "yuv-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-packed")))]
mod packed_yuv_8bit;
#[cfg(feature = "yuv-packed")]
pub use packed_yuv_4_1_1::*;
#[cfg(feature = "yuv-packed")]
pub use packed_yuv_8bit::*;

#[cfg(feature = "yuv-444-packed")]
#[cfg_attr(docsrs, doc(cfg(feature = "yuv-444-packed")))]
mod packed_yuv_4_4_4;
#[cfg(feature = "yuv-444-packed")]
pub use packed_yuv_4_4_4::*;

#[cfg(feature = "y2xx")]
#[cfg_attr(docsrs, doc(cfg(feature = "y2xx")))]
mod y2xx;
#[cfg(feature = "y2xx")]
pub use y2xx::*;

#[cfg(feature = "v210")]
#[cfg_attr(docsrs, doc(cfg(feature = "v210")))]
mod v210;
#[cfg(feature = "v210")]
pub use v210::*;

#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_10bit;
#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_16bit;
#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_32bit;
#[cfg(feature = "rgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb")))]
mod packed_rgb_8bit;
#[cfg(feature = "rgb")]
pub use packed_rgb_8bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_10bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_16bit::*;
#[cfg(feature = "rgb")]
pub use packed_rgb_32bit::*;

#[cfg(feature = "rgb-float")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-float")))]
mod packed_rgb_f16;
#[cfg(feature = "rgb-float")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-float")))]
mod packed_rgb_float;
#[cfg(feature = "rgb-float")]
pub use packed_rgb_f16::*;
#[cfg(feature = "rgb-float")]
pub use packed_rgb_float::*;

#[cfg(feature = "rgb-legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "rgb-legacy")))]
mod legacy_rgb;
#[cfg(feature = "rgb-legacy")]
pub use legacy_rgb::*;

#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod planar_gbr_8bit;
#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod planar_gbr_float;
#[cfg(feature = "gbr")]
#[cfg_attr(docsrs, doc(cfg(feature = "gbr")))]
mod planar_gbr_high_bit;
#[cfg(feature = "gbr")]
pub use planar_gbr_8bit::*;
#[cfg(feature = "gbr")]
pub use planar_gbr_float::*;
#[cfg(feature = "gbr")]
pub use planar_gbr_high_bit::*;

#[cfg(feature = "gray")]
#[cfg_attr(docsrs, doc(cfg(feature = "gray")))]
mod gray;
#[cfg(feature = "gray")]
pub use gray::*;

#[cfg(feature = "bayer")]
#[cfg_attr(docsrs, doc(cfg(feature = "bayer")))]
mod bayer;
#[cfg(feature = "bayer")]
pub use bayer::*;

#[cfg(feature = "xyz")]
#[cfg_attr(docsrs, doc(cfg(feature = "xyz")))]
mod xyz12;
#[cfg(feature = "xyz")]
pub use xyz12::*;

#[cfg(feature = "mono")]
#[cfg_attr(docsrs, doc(cfg(feature = "mono")))]
mod mono1bit;
#[cfg(feature = "mono")]
#[cfg_attr(docsrs, doc(cfg(feature = "mono")))]
mod pal8;
#[cfg(feature = "mono")]
pub use mono1bit::*;
#[cfg(feature = "mono")]
pub use pal8::*;

// === Tests ===

#[cfg(test)]
mod tests_primitives {
  use super::*;

  #[test]
  fn dimensions_construction_and_accessors() {
    let d = Dimensions::new(1920, 1080);
    assert_eq!(d.width(), 1920);
    assert_eq!(d.height(), 1080);
    assert!(!d.is_zero());
    assert!(Dimensions::default().is_zero());
  }

  #[test]
  fn dimensions_builder() {
    let d = Dimensions::new(0, 0).with_width(640).with_height(480);
    assert_eq!(d.width(), 640);
    assert_eq!(d.height(), 480);
  }

  #[cfg(feature = "std")]
  #[test]
  fn dimensions_display() {
    assert_eq!(std::format!("{}", Dimensions::new(1920, 1080)), "1920x1080");
  }

  #[test]
  fn rect_construction_and_accessors() {
    let r = Rect::new(10, 20, 1280, 720);
    assert_eq!(r.x(), 10);
    assert_eq!(r.y(), 20);
    assert_eq!(r.width(), 1280);
    assert_eq!(r.height(), 720);
  }

  #[test]
  fn rect_builder_chains() {
    let r = Rect::default()
      .with_x(8)
      .with_y(8)
      .with_width(640)
      .with_height(360);
    assert_eq!((r.x(), r.y(), r.width(), r.height()), (8, 8, 640, 360));
  }

  #[test]
  fn rotation_defaults_and_as_str() {
    assert!(matches!(Rotation::default(), Rotation::D0));
    assert_eq!(Rotation::D0.as_str(), "0");
    assert_eq!(Rotation::D90.as_str(), "90");
    assert_eq!(Rotation::D180.as_str(), "180");
    assert_eq!(Rotation::D270.as_str(), "270");
    assert!(Rotation::D90.is_d_90());
  }

  #[test]
  fn rotation_u32_round_trip_and_escape() {
    for r in [Rotation::D0, Rotation::D90, Rotation::D180, Rotation::D270] {
      assert_eq!(Rotation::from_u32(r.to_u32().unwrap()), Some(r));
    }
    assert_eq!(Rotation::from_u32(0), Some(Rotation::D0));
    assert_eq!(Rotation::from_u32(3), Some(Rotation::D270));
    // Unrecognised → rejected, never a silent collapse to D0.
    assert_eq!(Rotation::from_u32(99), None);
  }

  /// The escape carries a name, and has no numeric spelling. Needs the
  /// allocator — at the no-alloc tier the vocabulary is closed.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[test]
  fn rotation_escape_keeps_its_name() {
    let odd = Rotation::other("45");
    assert_eq!(odd.as_str(), "45");
    assert_eq!(odd.to_u32(), None);
    assert_eq!("45".parse(), Ok(odd));
  }

  #[test]
  fn sample_aspect_ratio_default_is_square() {
    let s = SampleAspectRatio::default();
    assert_eq!(s.num(), 1);
    assert_eq!(s.den().get(), 1);
    assert!(s.is_square());
  }

  #[test]
  fn sample_aspect_ratio_construction_and_builders() {
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    let s = SampleAspectRatio::new(40, nz(33));
    assert_eq!(s.num(), 40);
    assert_eq!(s.den().get(), 33);
    assert!(!s.is_square());
    let s2 = SampleAspectRatio::default().with_num(16).with_den(nz(9));
    assert_eq!((s2.num(), s2.den().get()), (16, 9));
    let mut s3 = SampleAspectRatio::default();
    s3.set_num(4).set_den(nz(3));
    assert_eq!((s3.num(), s3.den().get()), (4, 3));
  }

  #[cfg(feature = "std")]
  #[test]
  fn sample_aspect_ratio_display() {
    let nz = core::num::NonZeroI64::new(11).unwrap();
    assert_eq!(std::format!("{}", SampleAspectRatio::new(10, nz)), "10:11");
  }

  #[test]
  fn plane_holds_owned_buffer() {
    let p: Plane<[u8; 4]> = Plane::new([1, 2, 3, 4], 4);
    assert_eq!(p.stride(), 4);
    assert_eq!(p.data_ref(), &[1, 2, 3, 4]);
    let raw = p.into_data();
    assert_eq!(raw, [1, 2, 3, 4]);
  }

  #[test]
  fn plane_holds_borrowed_buffer() {
    let backing = [10u8, 20, 30, 40];
    let p: Plane<&[u8]> = Plane::new(&backing[..], 2);
    assert_eq!(p.stride(), 2);
    assert_eq!(*p.data_ref(), &[10, 20, 30, 40][..]);
  }

  #[test]
  fn plane_with_stride_builder() {
    let p = Plane::new([0u8; 2], 0).with_stride(64);
    assert_eq!(p.stride(), 64);
  }

  // ---------- VideoFrame -------------------------------------------------

  use crate::{color::Info, pixel_format::PixelFormat};

  #[test]
  fn video_frame_construction_defaults() {
    let planes: [Plane<&[u8]>; 4] = [
      Plane::new(&[][..], 16),
      Plane::new(&[][..], 8),
      Plane::new(&[][..], 8),
      Plane::new(&[][..], 0),
    ];
    let vf = VideoFrame::new(Dimensions::new(16, 16), PixelFormat::Yuv420p, planes, 3);
    assert_eq!(vf.dimensions(), Dimensions::new(16, 16));
    assert_eq!(vf.width(), 16);
    assert_eq!(vf.height(), 16);
    assert_eq!(*vf.pixel_format_ref(), PixelFormat::Yuv420p);
    assert_eq!(vf.plane_count(), 3);
    assert!(vf.visible_rect().is_none());
    assert_eq!(vf.color(), Info::UNSPECIFIED);
  }

  #[test]
  fn video_frame_planes_slice_uses_plane_count() {
    let planes: [Plane<u32>; 4] = [
      Plane::new(1, 0),
      Plane::new(2, 0),
      Plane::new(3, 0),
      Plane::new(4, 0),
    ];
    let vf = VideoFrame::new(Dimensions::new(2, 2), PixelFormat::Yuv420p, planes, 2);
    assert_eq!(vf.planes().len(), 2);
    assert_eq!(*vf.plane(0).unwrap().data_ref(), 1);
    assert_eq!(*vf.plane(1).unwrap().data_ref(), 2);
    assert!(vf.plane(2).is_none());
    assert!(vf.plane(7).is_none());
  }

  #[test]
  #[should_panic(expected = "plane_count exceeds the fixed 4-plane array")]
  fn video_frame_new_panics_on_plane_count_over_4() {
    let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
    let _ = VideoFrame::new(Dimensions::new(1, 1), PixelFormat::Yuv420p, planes, 5);
  }

  #[test]
  fn video_frame_with_visible_rect_and_color_chain() {
    let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
    let vf = VideoFrame::new(Dimensions::new(8, 8), PixelFormat::Yuv420p, planes, 3)
      .with_visible_rect(Rect::new(0, 0, 6, 6));
    assert_eq!(vf.visible_rect(), Some(Rect::new(0, 0, 6, 6)));
  }

  // ---------- TimestampedFrame ------------------------------------------

  #[test]
  fn timestamped_frame_construction_defaults() {
    let tf: TimestampedFrame<&'static str> = TimestampedFrame::new("payload");
    assert!(tf.pts().is_none());
    assert!(tf.duration().is_none());
    assert_eq!(*tf.frame_ref(), "payload");
  }

  #[test]
  fn timestamped_frame_into_frame_consumes() {
    let tf = TimestampedFrame::new(42u32);
    let raw = tf.into_frame();
    assert_eq!(raw, 42);
  }

  #[test]
  fn timestamped_frame_pts_builder() {
    let tb = mediatime::Timebase::new(1, core::num::NonZeroI32::new(1000).unwrap());
    let ts = mediatime::Timestamp::new(1000, tb);
    let tf = TimestampedFrame::new(0u8).with_pts(ts).with_duration(ts);
    assert_eq!(tf.pts(), Some(ts));
    assert_eq!(tf.duration(), Some(ts));
  }

  #[test]
  fn timestamped_frame_wraps_video_frame() {
    let planes: [Plane<()>; 4] = [Plane::new((), 0); 4];
    let vf = VideoFrame::new(Dimensions::new(4, 4), PixelFormat::Yuv420p, planes, 3);
    let tf = TimestampedFrame::new(vf);
    assert_eq!(tf.frame_ref().dimensions(), Dimensions::new(4, 4));
  }

  // ---------- Rational --------------------------------------------------

  #[test]
  fn rational_default_is_one_over_one() {
    let r = Rational::default();
    assert_eq!(r.num(), 1);
    assert_eq!(r.den().get(), 1);
    assert!(!r.is_zero());
  }

  #[test]
  fn rational_construction_builders_and_is_zero() {
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    let r = Rational::new(30000, nz(1001));
    assert_eq!(r.num(), 30000);
    assert_eq!(r.den().get(), 1001);
    assert!(!r.is_zero());
    let z = Rational::new(0, nz(1));
    assert!(z.is_zero());
    let r2 = Rational::default().with_num(24).with_den(nz(1));
    assert_eq!((r2.num(), r2.den().get()), (24, 1));
    let mut r3 = Rational::default();
    r3.set_num(16).set_den(nz(9));
    assert_eq!((r3.num(), r3.den().get()), (16, 9));
  }

  #[cfg(feature = "std")]
  #[test]
  fn rational_display() {
    let nz = core::num::NonZeroI64::new(1001).unwrap();
    assert_eq!(std::format!("{}", Rational::new(30000, nz)), "30000/1001");
  }

  // ---------- Rational sign / width invariants --------------------------
  //
  // `num`/`den` were `u32`/`NonZeroU32`, where the types made every
  // invariant unrepresentable. Under `i64`/`NonZeroI64` only
  // "denominator is non-zero" is still enforced by the type; the sign
  // half moved into `new`, so it needs pinning here.

  #[test]
  fn rational_rejects_negative_numerator() {
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    assert!(Rational::try_new(-1, nz(1)).is_none());
    assert!(Rational::try_new(i64::MIN, nz(1)).is_none());
    // `0` is a legal degenerate ratio (FFmpeg's "unknown" `0/1`).
    assert!(Rational::try_new(0, nz(1)).is_some());
  }

  #[test]
  fn rational_rejects_negative_denominator() {
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    assert!(Rational::try_new(1, nz(-1)).is_none());
    assert!(Rational::try_new(1, nz(i64::MIN)).is_none());
  }

  #[test]
  fn rational_zero_denominator_is_unrepresentable() {
    // Not a runtime check in `new` — `NonZeroI64` has no zero value at
    // all, so the state cannot be constructed to be rejected.
    assert!(core::num::NonZeroI64::new(0).is_none());
  }

  #[test]
  #[should_panic(expected = "rational numerator must not be negative")]
  fn rational_new_panics_on_negative_numerator() {
    let _ = Rational::new(-1, DEN_ONE);
  }

  #[test]
  #[should_panic(expected = "rational denominator must be positive")]
  fn rational_new_panics_on_negative_denominator() {
    let nz = core::num::NonZeroI64::new(-2).unwrap();
    let _ = Rational::new(1, nz);
  }

  #[test]
  #[should_panic(expected = "rational numerator must not be negative")]
  fn rational_set_num_routes_through_new() {
    // The four mutators are the invariant hole a direct field
    // assignment would leave open; each goes through `new`.
    let mut r = Rational::default();
    r.set_num(-1);
  }

  #[test]
  #[should_panic(expected = "rational denominator must be positive")]
  fn rational_set_den_routes_through_new() {
    let mut r = Rational::default();
    r.set_den(core::num::NonZeroI64::new(-3).unwrap());
  }

  #[test]
  #[should_panic(expected = "rational numerator must not be negative")]
  fn rational_with_num_routes_through_new() {
    let _ = Rational::default().with_num(-1);
  }

  #[test]
  #[should_panic(expected = "rational denominator must be positive")]
  fn rational_with_den_routes_through_new() {
    let _ = Rational::default().with_den(core::num::NonZeroI64::new(-3).unwrap());
  }

  #[test]
  fn rational_accepts_i64_max_at_both_positions() {
    let nz = core::num::NonZeroI64::new(i64::MAX).unwrap();
    let r = Rational::new(i64::MAX, nz);
    assert_eq!(r.num(), i64::MAX);
    assert_eq!(r.den().get(), i64::MAX);
  }

  #[test]
  fn rational_accepts_values_above_u32_max() {
    // The capability the widening buys: a numerator (and denominator)
    // the previous `u32` representation could not hold at all.
    let big = i64::from(u32::MAX) + 1;
    let nz = core::num::NonZeroI64::new(big).unwrap();
    let r = Rational::new(big, nz);
    assert_eq!(r.num(), big);
    assert_eq!(r.den().get(), big);
    // And through the semantic wrappers, which carry no width of their own.
    let sar = SampleAspectRatio::new(big, nz);
    assert_eq!((sar.num(), sar.den().get()), (big, big));
    assert_eq!(FrameRate::new(r, false).rate(), r);
  }

  #[test]
  fn sample_aspect_ratio_fallible_path_is_try_new_plus_from() {
    // `SampleAspectRatio::new` panics like `Rational::new`; the
    // fallible route is the existing `From<Rational>`.
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    let ok = Rational::try_new(40, nz(33)).map(SampleAspectRatio::from);
    assert_eq!(ok, Some(SampleAspectRatio::new(40, nz(33))));
    let bad = Rational::try_new(-40, nz(33)).map(SampleAspectRatio::from);
    assert!(bad.is_none());
  }

  // ---------- SampleAspectRatio ↔ Rational interop ----------------------

  #[test]
  fn sample_aspect_ratio_rational_interop() {
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    let sar = SampleAspectRatio::new(40, nz(33));
    let via_method: Rational = sar.as_rational();
    let via_from: Rational = Rational::from(sar);
    let via_into: Rational = sar.into();
    assert_eq!(via_method, Rational::new(40, nz(33)));
    assert_eq!(via_method, via_from);
    assert_eq!(via_from, via_into);
    // Default 1:1 SAR maps to the 1/1 Rational default.
    assert_eq!(
      SampleAspectRatio::default().as_rational(),
      Rational::default()
    );
  }

  #[test]
  fn sample_aspect_ratio_rational_round_trip_both_ways() {
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    // SAR -> Rational -> SAR
    let sar = SampleAspectRatio::new(40, nz(33));
    let r: Rational = sar.into();
    let back: SampleAspectRatio = r.into();
    assert_eq!(back, sar);
    assert_eq!(sar.rational(), r);
    assert_eq!(sar.rational(), sar.as_rational());
    // Rational -> SAR -> Rational
    let r2 = Rational::new(16, nz(9));
    let s2 = SampleAspectRatio::from(r2);
    assert_eq!((s2.num(), s2.den().get()), (16, 9));
    assert_eq!(Rational::from(s2), r2);
  }

  #[test]
  fn sample_aspect_ratio_default_is_one_to_one() {
    let d = SampleAspectRatio::default();
    assert_eq!((d.num(), d.den().get()), (1, 1));
    assert!(d.is_square());
    assert_eq!(d, SampleAspectRatio::new(1, DEN_ONE));
  }

  #[test]
  fn sample_aspect_ratio_eq_and_hash_parity() {
    use core::hash::{Hash, Hasher};
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    let a = SampleAspectRatio::new(40, nz(33));
    let b = SampleAspectRatio::default().with_num(40).with_den(nz(33));
    assert_eq!(a, b);

    fn h(s: &SampleAspectRatio) -> u64 {
      // `no_std`-friendly deterministic hasher (FNV-1a).
      struct Fnv(u64);
      impl Hasher for Fnv {
        fn finish(&self) -> u64 {
          self.0
        }
        fn write(&mut self, bytes: &[u8]) {
          for &x in bytes {
            self.0 = (self.0 ^ x as u64).wrapping_mul(0x0100_0000_01b3);
          }
        }
      }
      let mut hasher = Fnv(0xcbf2_9ce4_8422_2325);
      s.hash(&mut hasher);
      hasher.finish()
    }
    assert_eq!(h(&a), h(&b));
  }

  // ---------- FrameRate -------------------------------------------------

  #[test]
  fn frame_rate_default_is_one_over_one_cfr() {
    let fr = FrameRate::default();
    assert_eq!(fr.rate(), Rational::default());
    assert!(!fr.is_vfr());
  }

  #[test]
  fn frame_rate_construction_and_builders() {
    let nz = |n: i64| core::num::NonZeroI64::new(n).unwrap();
    let ntsc = Rational::new(30000, nz(1001));
    let fr = FrameRate::new(ntsc, false);
    assert_eq!(fr.rate(), ntsc);
    assert!(!fr.is_vfr());
    let vfr = FrameRate::default().with_rate(ntsc).with_is_vfr();
    assert_eq!(vfr.rate(), ntsc);
    assert!(vfr.is_vfr());
    let mut fr3 = FrameRate::default();
    fr3.set_rate(Rational::new(25, nz(1))).set_is_vfr();
    assert_eq!(fr3.rate(), Rational::new(25, nz(1)));
    assert!(fr3.is_vfr());
    // raw-wrapper + clear forms
    let fr4 = FrameRate::default().maybe_is_vfr(true);
    assert!(fr4.is_vfr());
    let mut fr5 = FrameRate::default();
    fr5.update_is_vfr(true);
    assert!(fr5.is_vfr());
    fr5.clear_is_vfr();
    assert!(!fr5.is_vfr());
  }

  // ---------- FieldOrder ------------------------------------------------

  #[test]
  fn field_order_default_is_unknown_and_as_str() {
    assert_eq!(FieldOrder::default(), FieldOrder::Unknown);
    assert_eq!(FieldOrder::Unknown.as_str(), "unknown");
    // FFmpeg names its own absence, so `"unknown"` round-trips exactly —
    // it is a variant, not the old payload-collapsing escape.
    assert_eq!("unknown".parse(), Ok(FieldOrder::Unknown));
    assert_eq!(FieldOrder::Progressive.as_str(), "progressive");
    assert_eq!(FieldOrder::Tt.as_str(), "tt");
    assert_eq!(FieldOrder::Bb.as_str(), "bb");
    assert_eq!(FieldOrder::Tb.as_str(), "tb");
    assert_eq!(FieldOrder::Bt.as_str(), "bt");
    assert!(FieldOrder::Progressive.is_progressive());
  }

  #[test]
  fn field_order_u32_round_trip_and_escape() {
    for f in [
      FieldOrder::Unknown,
      FieldOrder::Progressive,
      FieldOrder::Tt,
      FieldOrder::Bb,
      FieldOrder::Tb,
      FieldOrder::Bt,
    ] {
      assert_eq!(FieldOrder::from_u32(f.to_u32().unwrap()), Some(f));
    }
    assert_eq!(FieldOrder::from_u32(1), Some(FieldOrder::Progressive));
    assert_eq!(FieldOrder::from_u32(5), Some(FieldOrder::Bt));
    // FFmpeg's own UNKNOWN sentinel (0) decodes to the named variant.
    assert_eq!(FieldOrder::from_u32(0), Some(FieldOrder::Unknown));
    assert_eq!(FieldOrder::from_u32(99), None);
  }

  // ---------- StereoMode ------------------------------------------------

  #[test]
  fn stereo_mode_default_is_mono_and_as_str() {
    assert_eq!(StereoMode::default(), StereoMode::Mono);
    assert_eq!(StereoMode::Mono.as_str(), "mono");
    assert_eq!(StereoMode::SideBySide.as_str(), "side-by-side");
    assert_eq!(StereoMode::Columns.as_str(), "columns");
    assert!(StereoMode::Mono.is_mono());
  }

  #[test]
  fn stereo_mode_u32_round_trip_and_escape() {
    for s in [
      StereoMode::Mono,
      StereoMode::SideBySide,
      StereoMode::TopBottom,
      StereoMode::FrameSequence,
      StereoMode::Checkerboard,
      StereoMode::SideBySideQuincunx,
      StereoMode::Lines,
      StereoMode::Columns,
    ] {
      assert_eq!(StereoMode::from_u32(s.to_u32().unwrap()), Some(s));
    }
    assert_eq!(StereoMode::from_u32(0), Some(StereoMode::Mono));
    assert_eq!(StereoMode::from_u32(7), Some(StereoMode::Columns));
    assert_eq!(StereoMode::from_u32(99), None);
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[test]
  fn stereo_mode_escape_keeps_its_name() {
    let vendor = StereoMode::other("Anaglyph");
    assert_eq!(vendor.as_str(), "anaglyph");
    assert_eq!(vendor.to_u32(), None);
    assert_eq!("anaglyph".parse(), Ok(vendor));
  }

  /// Every named variant of the three coded frame enums must survive
  /// `as_str()` → `FromStr`, with no shared slugs.
  #[test]
  fn every_named_frame_enum_variant_round_trips_through_its_slug() {
    macro_rules! sweep {
      ($ty:ty) => {{
        let mut named = 0usize;
        let mut codes = [0u32; 32];
        for code in 0..=1024u32 {
          let Some(value) = <$ty>::from_u32(code) else {
            continue;
          };
          let slug = value.as_str();
          assert_eq!(
            slug.parse::<$ty>(),
            Ok(value.clone()),
            "{} slug {slug:?} does not parse back to {value:?}",
            stringify!($ty)
          );
          assert!(
            !slug.bytes().any(|b| b.is_ascii_uppercase()),
            "{} slug {slug:?} is not lowercase-canonical",
            stringify!($ty)
          );
          {
            let mut upper = [0u8; 64];
            let n = slug.len();
            upper[..n].copy_from_slice(slug.as_bytes());
            upper[..n].make_ascii_uppercase();
            let upper = core::str::from_utf8(&upper[..n]).unwrap();
            assert_eq!(
              upper.parse::<$ty>(),
              Ok(value.clone()),
              "{} does not fold {upper:?} onto {slug:?}",
              stringify!($ty)
            );
          }
          for prior in codes.iter().take(named) {
            let prior = <$ty>::from_u32(*prior).expect("recorded code names a variant");
            assert_ne!(
              prior.as_str(),
              slug,
              "{} has two variants spelled {slug:?}",
              stringify!($ty)
            );
          }
          codes[named] = code;
          named += 1;
        }
        assert!(
          named > 0,
          "{} sweep found no named variants",
          stringify!($ty)
        );
      }};
    }

    sweep!(Rotation);
    sweep!(FieldOrder);
    sweep!(StereoMode);
  }

  #[test]
  fn field_order_names_its_own_unknown() {
    // `"unknown"` is a *name* now, not a payload-collapsing arm: on
    // `FieldOrder` it is FFmpeg's own variant.
    assert_eq!("unknown".parse(), Ok(FieldOrder::Unknown));
  }

  #[cfg(any(feature = "std", feature = "alloc"))]
  #[test]
  fn frame_enum_escape_keeps_the_name_it_was_given() {
    // Elsewhere `"unknown"` rides the escape like any other name this
    // build does not enumerate.
    assert_eq!("unknown".parse(), Ok(Rotation::other("unknown")));
    assert_eq!("unknown".parse(), Ok(StereoMode::other("unknown")));
    assert_eq!(
      "not-a-rotation".parse::<Rotation>().unwrap().as_str(),
      "not-a-rotation"
    );
  }

  /// The geometry types render an injective form, so `FromStr` is a true
  /// inverse of `Display` for every value — not only the named ones.
  // `std::format!` needs the allocator; these types themselves are
  // available at the no-alloc tier, where the round trip is untestable.
  #[cfg(any(feature = "std", feature = "alloc"))]
  #[test]
  fn geometry_display_round_trips_through_from_str() {
    use core::num::NonZeroI64;

    let nz = |n: i64| NonZeroI64::new(n).unwrap();

    for dims in [
      Dimensions::default(),
      Dimensions::new(1920, 1080),
      Dimensions::new(u32::MAX, u32::MAX),
    ] {
      assert_eq!(std::format!("{dims}").parse(), Ok(dims));
    }

    for ratio in [
      Rational::default(),
      Rational::new(30_000, nz(1001)),
      Rational::new(0, nz(1)),
      Rational::new(i64::MAX, nz(i64::MAX)),
    ] {
      assert_eq!(std::format!("{ratio}").parse(), Ok(ratio));
    }

    for sar in [
      SampleAspectRatio::default(),
      SampleAspectRatio::new(40, nz(33)),
      SampleAspectRatio::new(16, nz(9)),
    ] {
      assert_eq!(std::format!("{sar}").parse(), Ok(sar));
    }
  }

  /// The separators are part of each type's contract: a SAR is written
  /// `a:b` and a bare ratio `a/b`, so neither accepts the other's form.
  #[test]
  fn geometry_separators_are_not_interchangeable() {
    assert!("40/33".parse::<SampleAspectRatio>().is_err());
    assert!("40:33".parse::<Rational>().is_err());
    assert!("1920X1080".parse::<Dimensions>().is_err());
  }

  /// Parsing routes through `Rational::try_new`, so it cannot mint a
  /// value the constructor rejects — the invariant has exactly one gate.
  #[test]
  fn geometry_parsing_cannot_bypass_the_constructor_invariant() {
    // `num < 0` and `den <= 0` are what `Rational::try_new` refuses.
    assert!("-5/4".parse::<Rational>().is_err());
    assert!("5/-4".parse::<Rational>().is_err());
    assert!("5/0".parse::<Rational>().is_err());
    assert!("-1:1".parse::<SampleAspectRatio>().is_err());

    // A ratio fails for two reasons a caller reports differently, so
    // unlike the name vocabularies its error carries which.
    assert_eq!(
      "-5/4".parse::<Rational>().unwrap_err().kind(),
      RatioParseKind::OutOfRange
    );
    assert_eq!(
      "not-a-ratio".parse::<Rational>().unwrap_err().kind(),
      RatioParseKind::Malformed
    );
    assert_eq!(
      "-1:1".parse::<SampleAspectRatio>().unwrap_err().kind(),
      RatioParseKind::OutOfRange
    );
  }

  #[test]
  fn geometry_rejects_malformed_input() {
    for bad in ["", "1920", "1920x", "x1080", "1920x1080x1", "axb", " 1x2"] {
      assert!(
        bad.parse::<Dimensions>().is_err(),
        "{bad:?} should not parse as Dimensions"
      );
    }
    for bad in ["", "30000", "30000/", "/1001", "a/b", "1/2/3"] {
      assert!(
        bad.parse::<Rational>().is_err(),
        "{bad:?} should not parse as Rational"
      );
    }
  }
}

// === Frame-family tests (feature-gated) ===

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests;
