# Changelog

All notable changes to this crate are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

A vocabulary window: two closed enums stop pretending they might grow, one
closed enum opens, every open vocabulary publishes its roster, and the nine
that are compiled at every tier stop declaring an error their `alloc` build
cannot return.

**Breaking**, on three counts.

1. **`FromStr::Err` is now `Infallible` at the `alloc` / `std` tier** for
   the ten vocabularies compiled at every tier: `color::Matrix`,
   `color::Primaries`, `color::Transfer`, `color::DynamicRange`,
   `color::ChromaLocation`, `color::DcpTargetGamut`,
   `pixel_format::PixelFormat`, `frame::Rotation`, `frame::FieldOrder` and
   `frame::StereoMode`. Behaviour does not change at any tier — these
   parses have always been total wherever `Other` exists, and their docs
   have said so for releases. What changes is that the type now says it
   too. The `Parse*Error` types are untouched and still exported; the
   no-alloc tier still returns them.

   Downstream breaks are the places that *name* the old error: a `match`
   arm on it, a `From` impl or `?` conversion into a local error enum, and
   annotated or turbofished bindings that spell it. Code that merely
   propagated the error usually just deletes the arm. Where a value is
   wanted, the impossible error is discharged by an irrefutable binding —
   `let Ok(m) = s.parse::<Matrix>();` — which is stable today and is what
   this crate uses internally (`Result::into_ok` is the same thing once it
   stabilises).

2. **`subtitle::TrackOrigin` gained an `Other(SmolStr)` escape**, which
   moves it onto the crate-wide open-vocabulary shape and changes four
   surfaces: it is no longer `Copy`, `as_str` is no longer
   `const fn -> &'static str`, `to_u32` returns `Option<u32>`, and
   `FromStr::Err` is `Infallible` — **unconditionally**, with no `cfg`
   fork, because the `subtitle` module is compiled only at the `alloc`
   tier and so has no build in which the vocabulary closes.
   `ParseTrackOriginError` is kept and exported on the same policy as the
   ten above, and its doc states the one way it differs: their lean build
   still returns theirs, nothing returns this one today.

   **Both of its wire forms change**, so persisted 0.4.x values do not
   read back: `serde` moves from an integer code to the canonical slug
   string (so `0` no longer deserializes), and `buffa` field 1 moves from
   `Varint` to `LengthDelimited` (so a 0.4.x payload fails to decode
   rather than decoding wrongly). This is the same wire every other name
   vocabulary in the crate already uses.

3. **`subtitle::Format::PgsSub` is removed**, merged into
   `subtitle::Format::HdmvPgs`. **Stored data is unaffected** — the slug
   on disk is unchanged and parses to the surviving variant. Rust code
   naming `PgsSub` renames to `HdmvPgs`, which is the same value it
   already meant.

### Added

- **`ROSTER` on all eighteen open vocabularies.** `pub const ROSTER:
  &'static [Self]` lists the named variants in declaration order:
  `VideoCodec`, `AudioCodec`, `SubtitleCodec`, `audio::SampleFormat`,
  `audio::ContainerFormat`, `audio::ChannelLayout`, `container::Format`,
  `subtitle::Format`, the five `color` enums, `pixel_format::PixelFormat`,
  the three `frame` orientation enums, and `subtitle::TrackOrigin`.

  `#[non_exhaustive]` denies a downstream the `match` it would need to
  enumerate these itself, so every consumer that mirrors one of these
  vocabularies was going to hand-copy a list that silently rots at the
  next release. This publishes the list instead. It is a **slice**, not an
  array, so the count stays out of the type and a later addition remains a
  minor change, and it excludes the `Other` escape — the roster answers
  which names a build knows, and the escape is the arm carrying one it
  does not.

  Completeness is proved here rather than promised: an exhaustive `match`
  sits beside each type (`#[non_exhaustive]` does not bind the defining
  crate), so adding a variant without rostering it is `E0004` with the
  compiler naming it. Roster and witness are generated from one list per
  type, so there is no second list to drift.
- `subtitle::TrackOrigin::Derived` (slug `"derived"`, wire id `3`) — a
  track produced by a pass over the media (ASR transcript, machine
  translation, OCR) rather than obtained as subtitle text. `External` had
  been carrying that case in its doc while also meaning "downloaded";
  the two are now separate. Ids stay append-only.

### Changed

- **`frame::BayerPattern` is closed** — `#[non_exhaustive]` removed. The
  four standard arrangements are a geometric closure, not a snapshot of
  today's cameras: a 2×2 tile with one red, one blue and two greens admits
  exactly four top-left phases. Every CFA family that would want a fifth
  (Quad Bayer, X-Trans, RGBW, Foveon, monochrome) is a different tile shape
  and already leaves via a different type, as the type's own scope note has
  always said. Downstream matches keep compiling and gain a completeness
  proof.
- **`audio::BitRateMode` is closed** — `#[non_exhaustive]` removed. CBR /
  VBR / ABR is the whole of the *reporting* domain and has been stable for
  twenty-five years. The near misses are not members: CVBR is a shape of
  VBR, and CRF / CQP are encoder knobs describing how a file was produced,
  not a property the stream reports.
- **`subtitle::TrackOrigin` is an open vocabulary.** See breaking note 2.
  mediaframe is a shared library, not one pipeline's private enum: the set
  of provenances worth distinguishing belongs to whoever does the
  classifying, so a downstream tracking an origin this crate has not heard
  of now keeps its *name* rather than losing it to a nearby variant.
  `#[non_exhaustive]` is retained, so promoting a slug that rides `Other`
  today into a named variant tomorrow stays minor.
- **The ten all-tier vocabularies tell the truth about their parse.** See
  breaking note 1. The `cfg` predicate on the split is the same one that
  gates each type's `Other` arm, so the error type and the escape cannot
  drift apart, and each type carries an irrefutable-`let` proof that stops
  compiling if the error is narrowed back.

  With the set closed, the `buffa` string-enum codec stops guessing: its
  shared decoder replaced `from_str(..).unwrap_or_else(|_| unreachable!())`
  — unreachable only by argument — with an irrefutable binding, so all
  fourteen vocabularies it serves are now *proved* to parse totally at that
  tier, and adding one that cannot is a compile error.
- **`subtitle::Format::PgsSub` merged into `HdmvPgs`.** See breaking note
  3. One format wore two variant names rendering the same slug, so
  `FromStr` could return only one of them and `Display` was not invertible
  for the other. The survivor is the one whose name matches the canonical
  slug — the name the type's doc already crowned as FFmpeg-canonical.

## [0.4.0] - 2026-08-19

The FFmpeg pin moves `n8.1` → `n9.0` and every provenance label in the crate
is re-verified against it rather than re-typed.

**Breaking**: FFmpeg 9 drops the three `FF_API_V408_CODECID` codecs, so
`VideoCodec::{V308, V408, V410}` go with them — the strings still round-trip
through `VideoCodec::Other`.

### Added

- **FFmpeg synonyms on the parse side.** Where mediaframe's canonical slug
  and FFmpeg's own name for the same thing differ, `FromStr` now accepts
  both. Emission is unchanged and still injective: `as_str` / `Display` /
  serde render one canonical slug per variant and never a synonym, so
  `parse(display(x)) == x` and the `display ∘ parse` idempotence both still
  hold. The ten pairs, all read off FFmpeg's own name tables:
  `PixelFormat` — `gray` → `gray8`, `monob` → `monoblack`, `monow` →
  `monowhite` (the descriptor names `ffprobe` prints, against the
  `AV_PIX_FMT_<NAME>` identifiers this vocabulary is spelled after);
  `color::Matrix` — `gbr` → `rgb`, `unknown` → `unspecified`;
  `color::Primaries` and `color::DynamicRange` — `unknown` → `unspecified`;
  `color::Transfer` — `unknown` → `unspecified`, `bt470m` → `gamma22`,
  `bt470bg` → `gamma28`. A name copied off `ffprobe` now lands on the named
  variant and keeps its H.273 code instead of riding the `to_u32`-less
  escape. Nail tests prove no synonym shadows a canonical slug.
- `VideoCodec::WebpAnim` (`webp_anim`) and `AudioCodec::AppleApac`
  (`apple_apac`), both new in FFmpeg 9.

### Changed

- **`*Row::new` is `pub(crate)`; a hidden test door replaces the public
  promise.** Breaking. With the walkers no longer taking a selector, a public
  row constructor was the last way to build a row beside the description that
  chose its matrix — so it is gone from the public surface, and the one-source
  rule now holds at row grain for everything outside this crate. Covers all
  132 walker-generated row types plus `Pal8Row`, `BayerRow` and `BayerRow16`
  — 135 in all; the ten hand-written source rows that were already
  `pub(crate)` are unchanged, because there was no public promise there to
  replace. Nine of those ten also get no door for the same reason.
  `Xyz12Row` is the tenth and does get one, for a different reason: `xyz`
  was the only one of the fifteen format features with no door anywhere
  behind it, so the format had no way in for a kernel-parity suite at all.
  That is a hole in the coverage rather than a promise to replace, and it
  is now closed — every format feature owns at least one door, which is
  what the `row_test_door_doc` gate now says.

  The named exception is `#[doc(hidden)] *Row::for_tests`, emitted beside
  `new` with the identical parameter list — selector included — and
  forwarding to it, so the two cannot drift. It exists for one reason: a
  kernel-parity suite drives a single row kernel without materialising a
  frame, and there is no other way to reach one from outside. That is
  measured, not assumed — a census on 2026-08-19 found **493 such
  constructions across 85 files and 52 row types** in `pixon` alone, all of
  them test code, and every one of those 52 types has a door. It carries no
  stability promise and its doc says so.

  Nothing in this crate needed migrating: every in-tree row already came from
  a walker, which is why the door is exercised by a test of its own rather
  than by existing callers. **pixon's suites are the breakage**, and their
  migration (`Row::new` → `Row::for_tests`) rides pixon's own 0.4 bump.
- **The walkers stop taking a colour selector; the sink supplies it.**
  Breaking, across every `{fmt}_to` / `{fmt}_to_endian` entry point. The
  `matrix: KernelMatrix` parameter is **gone** — no deprecation — and
  `xyz12_to`'s `target_gamut: KernelGamut` with it. The value now travels on
  the sink contract:
  - `PixelSink::kernel_matrix(&self) -> KernelMatrix`, defaulting to
    `Unspecified` (the kernels' documented BT.709 posture).
  - `Xyz12Sink::target_gamut(&self) -> KernelGamut`, defaulting to `DciP3`
    (what `DcpTargetGamut` already documents for a caller who does not
    re-target). It sits on the XYZ12 subtrait rather than `PixelSink`
    because a gamut is an **output** axis with exactly one consumer, and a
    knob every other sink carries but no walker reads would be the same
    second door in a new place.

  Each walker asks once, after `begin_frame` and before the row loop, and
  stamps the answer on every row of that frame. Why: the sink already held a
  colour description, so passing a selector beside it meant two doors onto
  one fact — name one matrix, build the sink from another, and the picture
  is quietly wrong with nothing to fail on. Migration is deleting the
  argument; a sink that wants a specific matrix or gamut overrides the
  method. `full_range` stays a parameter — it is a quantisation fact about
  the frame, not a colour intent the sink owns.

  The kernels are untouched: rows still carry `matrix()` / `target_gamut()`,
  and the closed `KernelMatrix` / `KernelGamut` selectors are unchanged. Only
  *where the value enters* moved.
- **FFmpeg pin `n8.1` → `n9.0`** (`xtask/src/main.rs`), vendored tables
  regenerated by `cargo xtask sync` and `mediaframe::codec` by
  `cargo xtask gen-codec`. `AVPixelFormat` did not move between the two
  releases — `libavutil/pixfmt.h` is byte-identical — so the 254 vendored
  slugs, the 56 colour code points and `PixelFormat` itself need **no new
  variants**; the same holds for `AV_DISPOSITION_*` (19 flags),
  `AVSampleFormat` (12) and `AV_CODEC_PROP_*` (8). Only `codec_desc.c` moved.
- Provenance labels across the crate now read `n9.0`, each re-verified
  against the n9.0 headers rather than relabelled on faith. The generated
  codec module interpolates the pin and its own variant counts instead of
  carrying them as hand-typed strings — both had gone stale (`(281)` /
  `(221)` for enums that are now 279 / 222). Historical `CHANGELOG` entries
  and the "new in n8.1" note on the 96/128-bit packed RGB formats keep their
  tags: those are statements about the past, and still true.
- `"unknown"` still names nothing any colour enum renders, but on the four
  enums whose FFmpeg table spells `UNSPECIFIED` that way it now parses to
  `Unspecified` instead of riding the escape. `ChromaLocation` and
  `DcpTargetGamut` are unaffected — FFmpeg already agrees with them.

### Fixed

- **`ChannelLayout`'s 5.x slugs were inverted against FFmpeg** — breaking, and
  the wire form moves with them. FFmpeg's `channel_layout_map[]` gives the
  unqualified name to the **back**-speaker layouts (`"5.0"` →
  `AV_CH_LAYOUT_5POINT0_BACK` = `SURROUND|BACK_LEFT|BACK_RIGHT`, `"5.1"` →
  `AV_CH_LAYOUT_5POINT1_BACK`) and qualifies the side ones (`"5.0(side)"` →
  `AV_CH_LAYOUT_5POINT0` = `SURROUND|SIDE_LEFT|SIDE_RIGHT`, `"5.1(side)"` →
  `AV_CH_LAYOUT_5POINT1`). This crate had the four the other way round, so an
  FFmpeg- or `ffprobe`-sourced `"5.1"` parsed to `N5Point1`, whose docs promise
  side speakers, when FFmpeg meant back. The strings round-tripped, so nothing
  caught it; anything keying off the variant's documented speaker set was
  quietly wrong. The four slugs are swapped to match:
  `N5Point0` → `"5.0(side)"`, `N5Point0Back` → `"5.0"`,
  `N5Point1` → `"5.1(side)"`, `N5Point1Back` → `"5.1"`. `as_str`, `Display`,
  `FromStr` and the serde wire form move together. A transcribed
  `channel_layout_map[]` table now pins every named layout, so the next
  inversion fails a test instead of shipping.
- `ChannelLayout::Quad`'s doc claimed it was "L+R+SL+SR **or** L+R+BL+BR". It
  is `AV_CH_LAYOUT_QUAD` = `STEREO|BACK_LEFT|BACK_RIGHT` — back only. The side
  four-channel layout is FFmpeg's `AV_CH_LAYOUT_2_2`, named `"quad(side)"`,
  which this vocabulary does not enumerate and which rides `Other`. Doc only;
  the slug was already right.
- The hardware-exclusion roster no longer lies: `xvmc` had outlived
  `AV_PIX_FMT_XVMC` (already gone at n8.1) and excluded nothing.
  `cargo xtask sync` now proves every roster entry against the pinned header
  and refuses to write a table built from a stale one.

## [0.3.0]

**Breaking**, on three counts: the numeric escape (`Unknown(u32)`) is struck
from every coded vocabulary and `Other(SmolStr)` becomes the one extension
idiom; the YUV/RGB kernel door takes a closed selector instead of the open
`Matrix`; and the public dependency `mediatime` crosses 0.2 → 0.3 (its rescale
ladder was renamed and its rounding corrected — see mediatime's own notes;
`mediatime::Timestamp` is in this crate's public API).

### Added

- `FromStr` for the pixel-format, bayer, subtitle, audio and colour
  vocabularies — eighteen parse twins, each generated from its own `as_str`
  table and each with its **own** parse error type (the shared
  `parse::ParseError` is gone).
- The five RAW types (`BayerPattern`, `BayerDemosaic`, `WbChannel`,
  `WhiteBalance`, `ColorCorrectionMatrix`) join serde, arbitrary and
  quickcheck; the two float carriers deserialize through `try_new` and refuse
  invalid values.
- `Unwrap`/`TryUnwrap` reach `SampleFormat`, `ChannelLayout` and
  `SubtitleCodec`; the size threshold that keeps the two 200-plus codec enums
  out is now written down.
- `KernelMatrix` — a closed `Copy` selector of the ten matrices the
  conversion kernels actually have coefficients for — and `KernelGamut`
  (which deletes `xyz12_to`'s documented panic). Kernel entries take them
  directly; the other eight named matrices now refuse loudly
  (`UnsupportedKernelMatrixError`) where they used to convert silently as
  BT.709. `Unspecified` keeps its documented BT.709 default.
- Geometry projections: `Dimensions::aspect_ratio`, `Rect::aspect_ratio`
  (`Option<Rational>` — zero extents are ordinary), and
  `Dimensions::display_size(SampleAspectRatio)` with FFmpeg's
  `AV_ROUND_NEAR_INF` rounding; `Dimensions::contains(&Rect)` for crop
  validation.
- `FieldOrder::Unknown` and `PixelFormat::None` as **named** members (FFmpeg's
  own `AV_FIELD_UNKNOWN` / `AV_PIX_FMT_NONE` code points — a file saying
  "unknown" is a value, not an escape).

### Changed

- **Breaking:** `Unknown(u32)` struck from eleven types. `from_u32` returns
  `Option<Self>` and `to_u32` returns `Option<u32>` (the FFmpeg-interop
  boundary); serde **and** buffa wire shapes move from number to slug for the
  coded enums; `Copy` leaves the ten enums, `color::Info` and
  `frame::VideoFrame` (the per-row walker types get it back through
  `KernelMatrix`); `as_str` returns `&str` and is no longer `const` on the
  ten.
- **Breaking:** the canonical text form is lowercase (`"Bilinear"` →
  `"bilinear"`), every name door ASCII-case-folds its input (`FromStr` and the
  `other()` constructors), and folding is allocation-free — the parse tables
  compare bytes, which also made the biggest table ~2.5× faster.
- **Breaking:** at the no-alloc tier the coded enums are closed vocabularies —
  `Other` lives behind `any(feature = "alloc", feature = "std")`, and the tier
  law is documented: no name available means an error at the boundary, never a
  wrong value.
- **Breaking:** public dependency `mediatime` 0.2 → 0.3.
- Unit tests moved beside their modules (`foo/mod.rs` + `foo/tests.rs`) across
  the crate; test counts verified identical by name. Internal only.

**Breaking**, on two independent counts. `frame::Rational` widens to
`i64`/`NonZeroI64` and its constructor becomes checked (see **Changed**
below). And two public dependencies cross a major: `mediatime` 0.1 → 0.2
(`mediatime::Timestamp` appears in `frame::TimestampedFrame`'s public
signatures, so a caller holding a `mediatime 0.1` value no longer type-checks)
and `buffa` 0.8 → 0.9 (`Message` is implemented for public types, so a
downstream on 0.8 no longer sees those impls). **No wire byte changes** — every
entry below carries its own proof.

### Changed

- **`frame::Rational` is now `i64` / `NonZeroI64`** (was `u32` /
  `NonZeroU32`), and [`Rational::new`] is checked rather than total:
  it panics on `num < 0` or `den < 0`, with a new
  `Rational::try_new -> Option<Self>` as the fallible form.
  `SampleAspectRatio` (a newtype over `Rational`) and `FrameRate`
  (which composes it) follow automatically and carry no width of
  their own; `SampleAspectRatio::new` panics the same way, and its
  fallible route is the existing
  `Rational::try_new(..).map(SampleAspectRatio::from)`.

  *Why `i64`, and why `mediatime::Timebase` stays `i32`.* `mediaframe`
  is a pure **receiver** — nothing here is handed back to a decoder
  SDK — so "must round-trip into an `AVRational`", the reason
  `Timebase` went to `i32`, does not apply. What does apply is
  storage (`sqlx` has no `Type<Postgres>` for `u32`, so a `u32`
  widens to `i64` to be stored regardless) and ingest (R3D metadata
  returns `unsigned int`, ISO BMFF `pasp` is `unsigned int(32)` —
  values `i32` would have to *reject*). `Timebase` is additionally an
  arithmetic operand whose rescale overflow proofs need `num < 2^32`;
  `Rational` never multiplies against a PTS and carries no such
  proof. **The two types differ deliberately** — this is not an
  inconsistency to reconcile.

  The four setters (`with_num`/`with_den`/`set_num`/`set_den`) now
  route through `new`, so the sign invariants have exactly one
  enforcement site rather than a mutator hole. `Deserialize` was the
  other unguarded construction path — the derive assigns fields
  directly, and the field types no longer carry the invariant — so
  each field gained a `deserialize_with` guard; `{"num": -5}` is now
  a deserialization error instead of a value the constructor would
  refuse. The constructor deliberately does **not** reduce to lowest
  terms: a stream declaring `2/4` reads back as `2/4`.

  **The wire format does not change.** `SampleAspectRatio` and
  `Rational` move from `uint32 num/den` to `int64 num/den`, which is
  the same plain non-ZigZag varint over every value the old
  representation could hold. Proven, not inferred: 680 payloads
  across `Rational`, `SampleAspectRatio` and `FrameRate` — spanning
  every varint continuation boundary and `u32::MAX` — encode to
  identical bytes under both representations, and the `i64` build
  cross-decodes all 680 `uint32`-era payloads back to the same values
  and the same bytes. (`sint32`/`sint64` would have been the silent
  break, since ZigZag re-encodes every value; this crate uses neither.)
  Decode stays total in the newly reachable directions: a negative
  numerator clamps to `0` and a zero-or-negative denominator to `1`,
  matching `mediatime::Timebase`'s decode policy.

- **`xtask`: `syn` 2 → 3, `prettyplease` 0.2 → 0.3** — a coupled bump
  (`prettyplease` 0.3 requires `syn ^3`, so neither moves alone).
  Dev-only: `xtask` is `publish = false`, so nothing here reaches the
  published `mediaframe` artifact. `syn` 3's breaking change is
  `Signature::unsafety: Option<Token![unsafe]>` → the tri-state
  `Signature::safety: Safety` (Rust 2024 `unsafe extern`); `xtask`
  names only `syn::Ident` and `syn::parse2::<syn::File>` and never
  inspects a signature, so it compiles unchanged. `prettyplease` 0.3
  emits **byte-identical** output to 0.2 for the generated
  `mediaframe/src/codec.rs` (89,303 bytes pre-`rustfmt`), so
  `cargo xtask check`'s byte-for-byte freshness diff stays green and
  the committed file needs no regeneration.
- **`quickcheck-richderive` 0.3 → 0.4** (`quickcheck` feature) — upstream
  is a dependency-only release (its own `syn` 2 → 3 migration); the
  derive, the accepted attribute keys, and the emitted impls are
  unchanged. Re-verified against *this* crate rather than inherited:
  `-Zunpretty=expanded` over `--features quickcheck,frame,buffa,serde,arbitrary`
  is byte-identical across the bump (263,301 lines). All 40 derive sites
  keep their `#[quickcheck(arbitrary = "…")]` attributes as-is —
  that key names a **function**, and every value here points at a
  `pub(crate) fn(&mut Gen) -> T` in `quickcheck_helpers`, so none of
  them is the sibling `with = "…"` key (which names a *module* supplying
  both `arbitrary` and `shrink`). No consumer-visible change.
- **`buffa` 0.8 → 0.9** (`buffa` feature) — `Message::write_to` now takes
  `&mut impl EncodeSink` in place of `&mut impl BufMut`, so all 26
  `write_to` signatures in `src/buffa.rs` move (the trait method's
  parameter type is what changed, so keeping `BufMut` is an `E0276`
  "impl has stricter requirements"). Nothing else in the module changes:
  no body touches a `BufMut` method directly — every byte goes through
  `buffa`'s `encode_*` helpers, whose bodies are unchanged — and
  `buffa` carries a blanket `impl<T: BufMut + ?Sized> EncodeSink for T`,
  so every existing caller still passes a `Vec<u8>` / `BytesMut`.
  **The wire format does not change.** Established on this crate's own
  types rather than inherited: all 37 `Message` impls were driven over
  400 deterministic `arbitrary` values each (14,800 encodings) under
  0.8.1 and 0.9.1, and the encoded bytes are identical in every case —
  so bytes written by a 0.8-linked peer still decode here. The 112
  non-identity round-trips are `audio::SampleFormat` only, are present
  identically in both runs, and are the documented `Other(SmolStr)` →
  `Unknown(u32::MAX)` collapse, not a regression.
  `EncodeSink`'s segmented `Rope` sink is **not** adopted here.
- **`mediatime` 0.1 → 0.2** — `mediatime::Timebase`'s `num`/`den` became
  `i32`/`NonZeroI32` (matching ffmpeg's `AVRational`, which is
  `{int num; int den;}`), `Timebase::new` now panics on a negative
  numerator or denominator with `try_new` returning `Option`, and its
  `Deserialize` gained a range guard. The surface this crate touches is
  small: `mediatime::Timestamp` — not `Timebase` — is what
  `frame::TimestampedFrame` carries, and `Timestamp::new(i64, Timebase)`
  is unchanged, so the single site that moves is one test's
  `NonZeroU32` → `NonZeroI32` denominator literal. Every other
  `Timebase` mention in this crate is prose, and each statement it
  makes (non-proto-zero `1/1` default; a frame rate is deliberately not
  a PTS timebase) is still true of 0.2.
  Also collapses the transient duplicate from the previous commit:
  `mediatime` 0.2 requires `buffa` 0.9, so the graph carries one
  `buffa` again.

## [0.1.7]

### Added

- **`Primaries::chromaticities()` / `Primaries::white_point()`** —
  `const fn`s exposing the per-standard CIE 1931 `xy` reference data for
  each defined `Primaries` variant: the R, G, B primaries as
  `Option<[ChromaCoord; 3]>` (index `0` = red, `1` = green, `2` = blue)
  and the reference white point as `Option<ChromaCoord>`, both in
  `ChromaCoord`'s SMPTE ST 2086 fixed-point units (floating value =
  `raw / 50000.0`, so BT.709 red `(0.640, 0.330)` is `(32000, 16500)`).
  Values track FFmpeg's `av_csp_primaries_desc` (`libavutil/csp.c`)
  across BT.709 / sRGB, BT.470 M/BG, SMPTE 170M/240M, Film, BT.2020,
  SMPTE ST 428, DCI-P3 (RP 431-2), Display-P3 (EG 432-1), and EBU
  3213-E, with white points D65 / CIE C / DCI / equal-energy E as each
  standard dictates. `Unknown` and `Unspecified` return `None` (no
  defined primaries); the within-crate match is exhaustive without a
  wildcard, so a future primaries variant cannot silently fall through.
  Puts the colorimetric reference data in the format authority so
  downstream crates (e.g. `colconv`) consume one table instead of
  re-hardcoding chromaticities, and unblocks chromaticity-derived matrix
  work. Note that SMPTE ST 428 mirrors FFmpeg's tabulated D-Cinema
  primaries (white point E), **not** the CIE XYZ identity that ITU-T
  H.273 Table 2 lists for ST 428-1. Additive and non-breaking.

## [0.1.6]

### Added

- **`PixelFormat::V410Be`** — first-class big-endian counterpart of
  `V410Le` for the packed YUV 4:4:4 10-bit `V410` layout (one 32-bit
  word per sample). The big-endian decode path already existed — the
  `V410Frame<'a, true>` / `V410BeFrame` borrow view, the `V410<true>`
  source marker, and the endian-generic `v410_to::<true>` walker — and
  is now exposed as a wire-stable enum variant (`as_str()` slug
  `"v410be"`, discriminant `435`). Additive and non-breaking.
- **`PixelFormat::canonical()`** — `const fn` resolving a deprecated /
  aliased pixel format to `(canonical_format, Option<DynamicRange>)`:
  the non-deprecated format describing the same bytes, plus the dynamic
  range the alias *pins* (or `None` when the range is stream-driven).
  Centralises the alias table in the format authority so downstream
  crates (e.g. `colconv`) consume one mapping instead of each
  re-deriving it. Resolves the `yuvj{411,420,422,440,444}p` full-range
  aliases → their `yuv*p` base + `DynamicRange::Full`, `Gray8a` /
  `Y400a` → `Ya8`, and the `XV30` byte-order pair onto its matching
  `V410` variant — `Xv30Le` → `V410Le` and `Xv30Be` → `V410Be` (`XV30`
  is the FFmpeg rename of the identical-bit-pattern `V410`; both endians
  resolve while preserving byte order). The match is exhaustive without
  a wildcard, so a future alias variant cannot silently fall through.
  Additive and non-breaking — every other format (including `Unknown`)
  returns `(self, None)`.

## [0.1.5]

### Added

- **Pixel-format source coverage** — frame types, source markers, and
  `{fmt}_to` walkers for a large batch of additional formats, each wired
  through its per-family feature flag:
  - **NV20** (`yuv-semi-planar`) — 10-bit low-bit-packed semi-planar
    4:2:2; the low-bit-aligned twin of `P210` (one `u16` per sample with
    the 10 active bits in the low positions).
  - **Gray family** (`gray`) — `Gray32` (32-bit), `Grayf16` (`f16`),
    `Yaf16` / `Yaf32` (`f16` / `f32` gray + alpha).
  - **GBR family** (`gbr`) — `Gbrap32` (32-bit GBRA), `Gbrp10Msb` /
    `Gbrp12Msb` (MSB-packed — samples in the high bits).
  - **RGB family** (`rgb` / `rgb-float`) — `Rgb96` / `Rgba128`
    (32-bit-per-channel integer), `Rgbaf16` / `Rgbaf32` (`f16` / `f32`
    RGBA).
  - **YUV 4:4:4 MSB** (`yuv-planar`) — `Yuv444p10Msb` / `Yuv444p12Msb`
    (MSB-packed planar 4:4:4).
  - **Packed 4:4:4** (`yuv-444-packed`) — `Ayuv`, `Uyva`, `Vyu444`.
  - **Legacy bit-packed RGB** (`rgb-legacy`) — `Rgb4` / `Rgb4Byte` /
    `Rgb8` and `Bgr4` / `Bgr4Byte` / `Bgr8`.
  - **`Xv48`** (`yuv-444-packed`) — 16-bit packed YUV 4:4:4 (FFmpeg
    `AV_PIX_FMT_XV48LE` / `BE`); the full-16-bit sibling of `Xv36`.
  - **`Yuva420p12`** (`yuva`) — 12-bit low-bit-packed planar YUVA 4:2:0;
    a mediaframe extension (no FFmpeg pixel format) that non-FFmpeg
    decoders / WebCodecs emit.

### Changed

- **High-bit Bayer is now endian-aware** (`bayer`) — the Bayer source
  marker gains a trailing `const BE: bool = false` (source-compatible
  default), mirroring the `Y2xx` family, so the 10 / 12 / 14 / 16-bit
  Bayer formats (all four CFA patterns) support both little- and
  big-endian planes. The `&[u16]` plane is interpreted as wire bytes
  (LE for `BE = false`, BE for `BE = true`); FFmpeg defines the Bayer
  LE/BE split only at 16-bit, so the 10 / 12 / 14-bit forms are
  mediaframe extensions. Little-endian behavior is byte-identical on
  little-endian hosts.

## [0.1.4]

### Added

- **`audio::ReplayGain`** — value object for container-tagged loudness-
  normalization recommendations (FFmpeg `AV_PKT_DATA_REPLAYGAIN` side
  data or the `REPLAYGAIN_TRACK_*` / `REPLAYGAIN_ALBUM_*` `AVDictionary`
  keys). Carries `track_gain_db`, `track_peak`, and the optional
  album-level `album_gain_db` / `album_peak`. Distinct from
  [`audio::Loudness`]: `Loudness` is the EBU R128 measurement of the
  signal; `ReplayGain` is the normalization recommendation a tagger
  wrote into the container (the delta from a −18 LUFS reference).
  Album-level numbers cannot be computed from a single track's loudness
  alone, so both are independently useful and not redundant. Buffa wire
  bridge: `{ float track_gain_db = 1; float track_peak = 2; optional
  float album_gain_db = 3; optional float album_peak = 4; }`. Test
  helpers wired through `quickcheck_helpers::composite::replay_gain` +
  `arbitrary_impls::composite`.

## [0.1.1] May 21, 2026

### Added

- **`serde` feature** — optional `serde::{Serialize, Deserialize}` for the
  whole descriptor vocabulary, gated behind `--features serde` (off by
  default). The wire shape mirrors what storage backends already use, so a
  serde-`json` value matches their representation:
  - **Open** codec / format enums (`codec::{Video,Audio,Subtitle}Codec`,
    `container::Format`, `subtitle::Format`,
    `audio::{ChannelLayout, ContainerFormat}`) serialize as their canonical
    `as_str()` slug — `VideoCodec::H264` ⇄ `"h264"`, `Other("x265")` ⇄
    `"x265"` (no `{"Other": …}` wrapper).
  - **`audio::SampleFormat`** — has BOTH an `Unknown(u32)` numeric escape
    AND an `Other(SmolStr)` string escape, so it gets a bespoke impl rather
    than the slug-only path. On **human-readable** formats (JSON / YAML /
    …): named + `Other` values serialize as their `as_str()` string,
    `Unknown(v)` as the bare numeric code `v`. On **non-human-readable**
    binary formats (bincode / postcard / …): an explicit tagged
    `{Code(u32), Slug(Cow<str>)}` wire enum, since `deserialize_any` is
    unavailable there. All three arms round-trip losslessly on both.
  - **Closed FFmpeg-coded enums with a lossless `Unknown(u32)` escape**
    (`color::{Matrix, Primaries, Transfer, DynamicRange, ChromaLocation,
    DcpTargetGamut}`, `pixel_format::PixelFormat`,
    `frame::{Rotation, FieldOrder, StereoMode}`) and
    `disposition::TrackDisposition` serialize as their `to_u32()` integer.
    Round-trip is total: an unrecognised *code* deserializes to `Unknown(v)`.
    These accept only integers — there is no slug form.
  - **Strictly-closed coded enums (no `Unknown` arm)** —
    `subtitle::TrackOrigin` (`Embedded`/`Sidecar`/`External`) and
    `audio::BitRateMode` (`Cbr`/`Vbr`/`Abr`) — serialize as their `to_u32()`
    integer but **reject unrecognised wire codes** as serde errors instead
    of silently collapsing them to the default variant. Both expose a
    `try_from_u32(v: u32) -> Option<Self>` method backing this behavior.
  - **Plain structs** (`color::Info` and its HDR/mastering sub-structs,
    `frame::{Dimensions, Rect, Rational, SampleAspectRatio, FrameRate}`,
    `audio::{Loudness, Tags, Device}`… ) derive serde directly.
  - **Validated structs** (`capture::GeoLocation`, `audio::Fingerprint`,
    `audio::CoverArt`) route deserialize through their checking
    constructors, so out-of-range / invariant-violating values are rejected
    rather than materialised.
  - **`lang::Language`** serializes as its canonical BCP-47 string
    (`"en-US"`, `"zh-Hant-TW"`, `"und"`).
  - Works at every capability tier: the no-alloc Copy types gain serde
    under bare `--features serde`; the heap-tier types (codecs, formats,
    audio metadata, capture, language) when paired with `alloc` / `std`
    (forwarding `serde` to `smol_str` / `bytes`).

## [0.1.0] May 19, 2026

Initial `mediaframe` release — this crate is a **rename** of the
`videoframe` crate. It was previously published as `videoframe`
(version line `0.1.x`–`0.3.x`); those `videoframe` crates.io versions
are being **yanked** and superseded by `mediaframe 0.1.0` (fresh crate
identity).

### Added

- **`audio` module** — first cut of the audio-stream descriptor
  vocabulary (audio + container cluster of the `0.1.0` stream-vocab
  expansion):
  - `audio::ChannelLayout` — `#[non_exhaustive]` closed enum of
    common FFmpeg `AV_CH_LAYOUT_*` shapes (`Mono`, `Stereo`,
    `_2_1` through `_7_1` with `*Back` side-vs-back variants,
    `Hexagonal`, `Octagonal`, `Ambisonic1`/`2`/`3`) plus
    `Other(SmolStr)` lossless escape; `as_str()` returns the
    FFmpeg-canonical slug, `FromStr` is total.
  - `audio::BitRateMode` — closed `Cbr` / `Vbr` / `Abr` trichotomy
    (default `Cbr`), `to_u32`/`from_u32` for the wire codec.
  - `audio::SampleFormat` — sample-format vocabulary mirroring
    FFmpeg `AVSampleFormat` (`U8`/`S16`/`S32`/`S64`/`Flt`/`Dbl`
    packed + their `*p` planar twins), lossless `Unknown(u32)` +
    `Other(SmolStr)` escapes, `to_u32`/`from_u32` per FFmpeg
    `AV_SAMPLE_FMT_*` enum indices, `is_planar()` predicate.
  - `audio::ContainerFormat` — audio-only container vocab
    (`Mp3`, `Aac`, `Flac`, `Ogg`, `Opus`, `Wav`, `Aiff`, `Alac`,
    `Wma`, `Ape`, `Wv`, `Mka`, `M4a`, `Caf`) plus `Other(SmolStr)`.
  - `audio::Loudness` — EBU R128 / ITU-R BS.1770 measurement
    value object (`integrated_lufs`, `range_lu`, `true_peak_dbtp`,
    `sample_peak_dbfs` — all `f32`; no `Eq`/`Hash`).
  - `audio::Fingerprint` — algorithm-tagged opaque bytes
    (`{ algorithm: SmolStr, value: bytes::Bytes }` — O(1) clone),
    `try_new` rejects empty algorithm.
  - `audio::CoverArt` — embedded picture
    (`{ mime: SmolStr, data: bytes::Bytes }` — O(1) clone), `try_new`
    rejects empty mime / empty data.
  - `audio::Tags` — FFmpeg / Vorbis-Comment / iTunes-atom
    metadata: title, artist, album_artist, album, composer,
    genre, comment (`SmolStr`, `""` = absent) + year, track / disc
    number + total (`Option<u16>`) + language (`Option<SmolStr>`,
    TODO(lang) — swap to `Option<crate::Language>` after the
    capture-lang cluster lands).
- **`container::Format`** — top-level multimedia container
  vocabulary (`Mov`, `Mp4`, `Mkv`, `Webm`, `Avi`, `Flv`, `MpegTs`,
  `Ogg`, `Asf`, `Rm`, `Wmv`, `Mxf`, `Gxf`, `Threegp` — `.3gp` digit-
  prefix-renamed) plus `Other(SmolStr)`; audio-only containers live
  on [`audio::ContainerFormat`].
- **`subtitle` module** — `Format` (file / demuxer-tag axis,
  `#[non_exhaustive]` + `Other(SmolStr)`; named variants for the
  common text- and image-based formats — `Srt` / `WebVtt` / `Ass` /
  `Ssa` / `Sub` (MicroDVD) / `Mpl2` / `Lrc` / `Smi` / `Stl` / `Sbv` /
  `Ttml` / `MovText` / `DvdSub` / `PgsSub` / `HdmvPgs` / `DvbSub` /
  `XSub`; `as_str` / total `FromStr` round-trip; `is_image_based`
  helper for mediaschema's `REQUIRES_OCR` derivation) and
  `TrackOrigin` (closed unit-only enum — `Embedded` /
  `Sidecar` / `External`; stable `to_u32` / `from_u32` ids
  `0` / `1` / `2`; `Default == Embedded`). The module is gated on
  the `alloc` feature for the `Other(SmolStr)` escape.
- **`disposition::TrackDisposition`** — FFmpeg `AV_DISPOSITION_*`
  bitflags from `libavformat/avformat.h` n8.1 (`u32` backing).
  Shared across video / audio / subtitle tracks; ports the
  placeholder that used to live in `mediaschema::domain::bitflags`.
  `to_u32` / `from_u32` aliases for `bits` / `from_bits_retain` so
  unknown bits round-trip losslessly.
- **`capture` module** (alloc-gated) — EXIF / capture-metadata
  vocabulary.
  - `Device { make, model }` (private `SmolStr` fields; empty string
    means absent, never `Option<SmolStr>`; builders / setters /
    `is_empty`).
  - `GeoLocation { lat: f64, lon: f64, altitude: Option<f32> }` with
    range-validating `try_new`, ISO-6709 degrees-only
    parse/format (`from_iso6709` + `to_iso6709`, `FromStr` +
    `Display`, hand-rolled <200-line parser — no regex / no chrono).
    `(0, 0)` "Null Island" is accepted (it is a real, legal
    coordinate); only out-of-range lat/lon and structurally bad
    strings are rejected via `GeoLocationError::{LatOutOfRange,
    LonOutOfRange, Iso6709Malformed}`.
- **`lang::Language`** (alloc-gated) — validated BCP-47 language tag
  wrapping `icu_locale_core` `Language`/`Script`/`Region` subtags (`Copy`,
  heap-free in-rust representation; the `to_bcp47() -> String` /
  `Display` surface needs the allocator). `try_new(lang, script,
  region)` + `from_bcp47` / `Default = "und"` (ISO 639-3
  undetermined) + `is_undetermined` + `FromStr`.
  `LanguageError::{InvalidLanguage, InvalidScript, InvalidRegion,
  MalformedBcp47}`.
- **`buffa`** — hand-written `Message` / `DefaultInstance` wire
  support for every new type (see the `## Audio + container types`,
  `## Subtitle + disposition`, and `## Capture + language` sub-
  sections of the `buffa.rs` module doc). `GeoLocation` always-encodes
  `lat`/`lon` (the `(0, 0)` "Null Island" default is a real
  coordinate — proto3 zero-elision would be unsound, same defensive
  stance as `SampleAspectRatio`); `altitude` is presence-encoded
  (field emitted iff `Some`, including for `Some(0.0)`). The `buffa`
  feature now implies `alloc` (string-bearing wire codecs pull in
  `smol_str`).
- **Deps** — adds `icu_locale_core = "2"` and `bytes = "1"` (both
  optional, gated on the `alloc` feature; both `no_std`-friendly).
  `bytes::Bytes` backs the `audio::CoverArt` / `audio::Fingerprint`
  payloads so large blobs clone in O(1).

### Changes

- **Crate rename** — `videoframe` → `mediaframe`, version reset to
  `0.1.0`. The contents are carried over **verbatim**: the
  pixel-format / colour / frame vocabulary plus `Rational`,
  `FrameRate`, `FieldOrder`, `StereoMode`, `DolbyVisionConfig`, and
  `SampleAspectRatio` represented via `Rational`. No types, logic, or
  API changed other than the crate name (and the `buffa` proto
  package identifier `videoframe.v1` → `mediaframe.v1`).
- **Charter broadened** — the crate is now a *media-stream descriptor
  vocabulary* for video **+ audio + subtitle**, not video-only. Only
  the existing video vocabulary ships in `0.1.0`; audio/subtitle
  descriptor types will be added incrementally in later releases.

---

— the following entries are from the crate's `videoframe` history —

## videoframe 0.3.1 — May 19, 2026

### Added

- **`frame`** — `Rational` (generic exact `num/den` ratio,
  `NonZeroU32` denominator, `1/1` default), `FrameRate` (exact fps
  `Rational` + `is_vfr` marker; deliberately not
  `mediatime::Timebase`), `FieldOrder` (FFmpeg `AVFieldOrder`,
  lossless `Unknown(u32)`, `Unknown(0)` default), `StereoMode`
  (FFmpeg `AVStereo3DType`, lossless `Unknown(u32)`, `Mono` default).
- **`color`** — `DolbyVisionConfig` (FFmpeg
  `AVDOVIDecoderConfigurationRecord`; distinct from the HDR10 static
  `HdrStaticMetadata`).
- **`buffa`** — hand-written `Message`/`DefaultInstance` wire support
  for `Rational`, `FrameRate`, `FieldOrder`, `StereoMode`,
  `DolbyVisionConfig`.
- **`frame`** — `SampleAspectRatio` → `Rational` interop
  (`SampleAspectRatio::rational`/`as_rational`,
  `From<SampleAspectRatio> for Rational`, `From<Rational> for
  SampleAspectRatio`).

### Breakage

- **`frame::SampleAspectRatio`** — now represented as a newtype over
  `Rational` (`pub struct SampleAspectRatio(Rational)`) instead of
  its own `{ num, den }` fields, making `Rational` the single source
  of truth for "exact ratio with a non-zero denominator". The public
  *method* API (`new`/`num`/`den`/`is_square`/`with_*`/`set_*`/
  `Default`/`Display`/derives) and the `buffa` wire format are
  **byte-for-byte unchanged**; only the internal representation and
  the `From` surface (added `From<Rational> for SampleAspectRatio`,
  added `rational()` alongside `as_rational()`) changed.

## videoframe 0.3.0 — May 19, 2026

### Added

- **`buffa`** — optional `buffa` wire serialization for the colour /
  frame / HDR vocabulary (hand-written `Message`/`DefaultInstance`,
  no codegen); lets downstream proto schemas extern-map
  `.videoframe.v1` → `::videoframe`.
- **`color`/`frame`** — lossless `Unknown(u32)` catch-all on every
  colour enum, `Rotation`, and `DcpTargetGamut`: unrecognised /
  future / corrupt wire ids round-trip verbatim instead of collapsing
  to a default.
- **`color`** — `DOMAIN_EXT_BASE` + `Matrix::Bt601`
  (videoframe-domain superset id, disjoint from FFmpeg/H.273 codes).
- **`color`/`frame`** — `ContentLightLevel`, `ChromaCoord`,
  `MasteringDisplay`, `HdrStaticMetadata` (SMPTE ST 2086 / FFmpeg
  HDR10 static side-data); `Rotation`; `SampleAspectRatio`.
- **xtask** — `check` verifies colour-enum numbering against the
  pinned FFmpeg n8.1 header (vendored `ffmpeg-color.txt`).

### Breakage

- **`color`** — `Primaries`/`Transfer`/`Matrix`/
  `DynamicRange`/`ChromaLocation` renumbered to exact FFmpeg n8.1 /
  ITU-T H.273 code points; `to_u32`/`from_u32` now lossless.
- **`color::Transfer`** — `Bt470M`/`Bt470Bg` renamed to
  `Gamma22`/`Gamma28` (FFmpeg-canonical names for the identical
  transfer code 4/5; slugs / `Display` unchanged).
- **`color::Matrix`** — `Default` changed `Bt709` →
  `Unspecified` (FFmpeg `AVCOL_SPC_UNSPECIFIED`); `Info`
  default/`UNSPECIFIED` `matrix` likewise.
- **`color::ChromaCoord`** — `x`/`y` widened `u16` → `u32` so
  out-of-range wire values are preserved losslessly (no saturation).
- **`frame::Rotation`** — no longer `#[repr(u32)]`; gains
  `Unknown(u32)`.

### Changes

- **`buffa`** — standalone-enum codec elides on the type's `Default`
  (FFmpeg `UNSPECIFIED`), not proto3 wire-zero, so code `0` (e.g.
  `Matrix::Rgb`) is no longer conflated with "absent".
- **`source::xyz12`** — `xyz12_to` requires a concrete
  `DcpTargetGamut`; passing `Unknown(_)` panics with a descriptive
  message instead of silently decoding as DCI-P3.

## videoframe 0.2.0 — May 12, 2026

### Added

- Add bayer structures

### Breakage

- **`cfa`** - remove cfa mod

### Changes

- Make all error enums follows tuple enum errors

## videoframe 0.1.0 — May 11, 2026

This is the first release line. Nothing has been published to
crates.io yet; everything below describes the shape of the
forthcoming `0.1.0`.

### Added

- **`color`** — ITU-T H.273 enums (`Matrix`, `Primaries`,
  `Transfer`, `DynamicRange`, `ChromaLocation`) bundled into
  `Info`. Plus `DcpTargetGamut` for DCI-XYZ target-gamut
  selection. Each enum exposes `pub const fn as_str() -> &'static
  str` returning the FFmpeg-style wire slug, and a
  `derive_more::Display` impl routes through `as_str()` so the two
  cannot drift.
- **`cfa`** — Bayer mosaic descriptor (`BayerPattern`).
- **`pixel_format`** — single `PixelFormat` enum covering **every**
  pixel format in FFmpeg `n8.1`'s `AVPixelFormat` (254 variants
  excluding GPU-resident HW formats) plus cinema-RAW additions.
  ~270 variants total. `Unknown(u32)` preserves the raw wire value
  so `from_u32(to_u32(x)) == x` for every `x: u32`.
- **`frame::Dimensions`**, **`frame::Rect`**, **`frame::Plane<B>`** —
  structural primitives (always available).
- **`frame::VideoFrame<P, B>`** — runtime-tagged frame: dimensions,
  pixel format `P`, up to 4 `Plane<B>`, optional visible-rect crop,
  `Info`. **No timestamp**, no backend extras — pure pixel
  data. Generic over `P` (typically `PixelFormat`) and `B` (buffer
  type — `&'a [u8]` / `Vec<u8>` / `Bytes` / refcounted FFmpeg buffer).
- **`frame::TimestampedFrame<F>`** — orthogonal time-carrying wrapper
  bundling `Option<mediatime::Timestamp>` PTS + duration around any
  inner `F`. Composition over inheritance: pixel data stays
  independent of any timekeeping convention. Use with
  `VideoFrame<P, B>` for runtime-tagged decoder output or with
  typed `*Frame<'a, BE>` borrow views for conversion pipelines.
- **Typed `*Frame<'a, BE>` borrow types** (per-family feature-gated)
  — ~70 zero-copy validated borrow views covering planar YUV
  (4:2:0 / 4:2:2 / 4:4:4 / 4:4:0 / 4:1:1 / 4:1:0 at 8 / 9 / 10 / 12
  / 14 / 16-bit), planar YUVA (same matrix), semi-planar YUV (NV12
  / 16 / 21 / 24 / 42 + P010 / 210 / 410 families), packed YUV
  (YUYV422 / UYVY422 / YVYU422 / UYYVYY411 / V210 / V410 / XV30 /
  XV36 / AYUV64 / VUYA / VUYX / Y210 / Y212 / Y216), packed RGB
  (Rgb24 / Bgr24 / Rgba / Bgra / Argb / Abgr / Xrgb / Rgbx / Xbgr /
  Bgrx / Rgb48 / Bgr48 / Rgba64 / Bgra64 / X2Rgb10 / X2Bgr10),
  packed RGB float (Rgbf32 / Rgbf16), packed legacy RGB (Rgb444 /
  555 / 565 + Bgr counterparts), planar GBR / GBRA at 8 / 9-16 /
  float, grayscale (Gray8 / 9-16 / f32 / Ya8 / Ya16), Bayer 8 /
  10 / 12 / 14 / 16-bit × 4 patterns, Xyz12, and Pal8 / Monoblack /
  Monowhite. Each `*Frame<'a, BE>` carries a `<const BE: bool =
  false>` parameter selecting endianness; row kernels handle the
  byte-swap under the hood.
- **`source`** — per-format marker ZSTs (`Yuv420p`, `Nv12`,
  `Rgb24`, …), `*Row<'a>` borrow types, `*Sink` subtraits, and
  `*_to` walker fns that iterate Frame → Row → `PixelSink`. The
  `walker!` macro generates the marker / Row / Sink / walker
  quartet uniformly per format. The companion `marker!` macro
  generates the canonical marker shape (`pub struct Foo(())` with
  `pub const fn new()` constructor — private `()` field locks
  shape evolution to additive changes only).
- **`PixelSink`** + **`SourceFormat`** sealed traits re-exported at
  the crate root.
- **`xtask`** — dev-only Cargo subcommand. `cargo xtask sync`
  fetches FFmpeg's `libavutil/pixfmt.h` from the pinned release tag
  (currently `n8.1`) and writes the lowercase slug list to
  `xtask/vendor/ffmpeg-pixfmts.txt`. `cargo xtask check` diffs the
  vendored list against `PixelFormat::as_str()` and fails on any
  missing variant. Vendoring only the slug list (not the LGPL
  header verbatim) sidesteps the license question.

### Conventions

- **No public fields anywhere.** Every struct exposes private fields
  via `pub const fn` getters + `pub const fn new(...)` constructors
  + `#[must_use]` `with_*` consuming builders + `set_*` in-place
  setters. Applies to color types, frame primitives, all error
  payloads, and marker ZSTs.
- **Sealed-trait pattern** on `SourceFormat`: external crates can
  introspect but not extend the format set.
- **Single-source-of-truth display strings**: every enum's `Display`
  impl is derived through its `pub const fn as_str()` — no risk of
  drift between the two surfaces.
- **`derive_more::IsVariant`** on every enum (color, cfa,
  pixel_format, every `*FrameError`). Callers get `is_<variant>()`
  predicates for free.

### `*FrameError` shape

All 65 `*FrameError` enums use **newtype-tuple variants** wrapping
private-field payload structs (no struct-style variants). Pattern:

```rust
pub enum FooFrameError {
    Bar(Bar),
    Baz(Baz),
}
```

#### Shared error payloads

Common shapes live at the top of `videoframe::frame` and are reused
across every error enum that has the matching shape — variant
names carry plane / unit semantics, payload carries shape-only data:

- `ZeroDimension { width, height }`
- `DimensionOverflow { width, height }`
- `InsufficientStride { stride, min }` — wraps every
  `Insufficient*Stride` variant across the Y / U / V / A / G / B / R /
  Uv / Vu plane axes. Variant name conveys per-plane / per-unit
  semantics.
- `InsufficientPlane { expected, actual }` — wraps every
  `Insufficient*Plane` variant.
- `GeometryOverflow { stride, rows }`
- `OddWidth { width }`
- `WidthNotMultipleOf4 { width }`
- `WidthOverflow { width }`
- `UnsupportedBits { bits }`

Naming follows the **`Insufficient*` family** rather than the
historical `*TooShort` / `*TooSmall` style (e.g.
`InsufficientYPlane`, `InsufficientYStride`).

Rare / unique shapes get local payload structs adjacent to their
consumer enum: `Yuv420pFrame16SampleOutOfRange`,
`Yuva420pFrame16SampleOutOfRange`, `Yuva422pFrame16SampleOutOfRange`,
`Yuva444pFrame16SampleOutOfRange`, `BayerSampleOutOfRange`,
`PnSampleLowBitsSet`, `Xv36SampleLowBitsSetAt`, `PnUvStrideOdd`.

#### `Display` impls

Each payload struct derives `thiserror::Error` and owns its own
`#[error("...")]` message. Enum variants delegate via
`#[error(transparent)]` — display routes through the payload's
own `Display` impl. Trade-off: per-enum format-identifying
prefixes (e.g. "V210Frame: zero dimension width=X height=Y")
drop in favor of canonical payload-owned messages; format
identity lives on the typed enum (`V210FrameError`) itself.

#### Generated accessors

Every `*FrameError` derives `derive_more::{IsVariant, TryUnwrap,
Unwrap}` with `#[unwrap(ref, ref_mut)]` + `#[try_unwrap(ref,
ref_mut)]` modifiers. Each variant gets:

- `is_<variant>() -> bool`
- `unwrap_<variant>(self) -> Payload`
- `unwrap_<variant>_ref(&self) -> &Payload`
- `unwrap_<variant>_mut(&mut self) -> &mut Payload`
- `try_unwrap_<variant>(self) -> Result<Payload, Self>`
- `try_unwrap_<variant>_ref(&self) -> Result<&Payload, &Self>`
- `try_unwrap_<variant>_mut(&mut self) -> Result<&mut Payload, &mut Self>`

### Feature flags

- `default = ["std"]` — `std` and `alloc` features, mediatime,
  derive_more (`is_variant` + `display`), thiserror always pulled
  in (small, no_std-friendly).
- **Per-family feature flags** gate the typed `*Frame<'a, BE>`
  validators and the matching `source::*` walker quartet so
  consumers compile only the formats they actually use:

  | Feature           | Formats                                                  |
  |-------------------|----------------------------------------------------------|
  | `yuv-planar`      | Yuv420p / 422p / 444p / 440p / 411p / 410p + 9-16 bit    |
  | `yuv-semi-planar` | NV12 / 16 / 21 / 24 / 42, P010 / 210 / 410 family        |
  | `yuva`            | YUVA planar 8-bit + 9-16 bit                             |
  | `yuv-packed`      | YUYV422, UYVY422, YVYU422, UYYVYY411                     |
  | `yuv-444-packed`  | V410, XV30, XV36, AYUV64, VUYA, VUYX, V30X               |
  | `y2xx`            | Y210 / Y212 / Y216                                       |
  | `v210`            | V210                                                     |
  | `rgb`             | Rgb24/Bgr24/Rgba/Bgra + 10-bit + 16-bit                  |
  | `rgb-float`       | Rgbf32 / Rgbf16 + Rgbaf16/f32                            |
  | `rgb-legacy`      | Rgb444 / 555 / 565 + Bgr counterparts                    |
  | `gbr`             | Gbrp / Gbrap + 9-16 bit + float                          |
  | `gray`            | Gray8 / 9-16 bit / f32, Ya8 / Ya16                       |
  | `bayer`           | Bayer 8 / 10 / 12 / 14 / 16-bit × 4 patterns             |
  | `xyz`             | Xyz12 (DCI-XYZ)                                          |
  | `mono`            | Monoblack / Monowhite / Pal8                             |
  | `frame`           | umbrella — enables every sub-feature above               |

  Deps pulled by family features:
  - `half` — `rgb-float`, `gbr`, `gray` (for `half::f16`)
  - `derive_more` `try_unwrap` / `unwrap` features — every
    per-family feature (so all `*FrameError` enums get the full
    unwrap accessor surface).

### `no_std`

Default-feature `std` is on. `--no-default-features` builds pure
no_std (enums + `Copy` types + marker ZSTs + frame primitives).
Add `alloc` for the small set of `Vec` / `String` helpers used
under `no_std + alloc`. The `extern crate alloc as std` aliasing
pattern keeps `std::vec::Vec` / `std::format!` resolving uniformly
across feature combos.

### Verification matrix

- Default features: 36 tests
- `--no-default-features --features alloc`: 32 tests
- `--features frame`: 656 tests
- All 15 individual per-family standalone builds compile
- `cargo xtask check` validates `PixelFormat` exhaustiveness
  against vendored FFmpeg `n8.1` slugs
