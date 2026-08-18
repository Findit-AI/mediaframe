//! Packed **RGBAF16** source (FFmpeg `AV_PIX_FMT_RGBAF16{LE,BE}`) — 16-bit
//! half-precision float per channel, byte order `R, G, B, A` per pixel
//! (8 bytes / 4 × `half::f16` per pixel).
//!
//! The alpha-carrying twin of [`super::Rgbf16`] (4 components vs 3). Like the
//! 8-bit packed-RGB family the input is already RGB — there is no chroma matrix
//! work. Outputs map to the sink's standard channels (with a saturating cast
//! back to integer for u8 / u16 / luma / HSV outputs):
//! - `with_rgb` — clamp `[0, 1]` × 255 → packed `R, G, B` u8 (alpha dropped).
//! - `with_rgba` — same RGB conversion + source alpha.
//! - `with_rgb_u16` — clamp `[0, 1]` × 65535 → packed `R, G, B` u16 (alpha dropped).
//! - `with_rgba_u16` — same RGB conversion + source alpha.
//! - `with_luma` / `with_luma_u16` — derives Y' from R/G/B (after the
//!   clamp + cast to u8) using the existing `rgb_to_luma_row` /
//!   `rgb_to_luma_u16_row` kernels (alpha ignored).
//! - `with_hsv` — clamp + cast to u8 staging followed by the existing
//!   `rgb_to_hsv_row` kernel (alpha ignored).
//! - `with_rgb_f16` — **lossless** half-float pass-through of R, G, B (HDR
//!   values > 1.0 are preserved).
//! - `with_rgb_f32` — lossless widening: each `f16` element is widened
//!   to `f32` (HDR values > 1.0 are preserved).
//!
//! HDR values > 1.0 in the source saturate to the output range for
//! every integer output. No tone mapping is applied.

use crate::frame::Rgbaf16Frame;

walker! {
  packed_be {
    /// Zero-sized marker for the packed **RGBAF16** source format.
    /// `<const BE: bool = false>` mirrors the parent
    /// [`Rgbaf16Frame`](crate::frame::Rgbaf16Frame)'s endian flag — `false` (default) selects
    /// `AV_PIX_FMT_RGBAF16LE`, `true` selects `AV_PIX_FMT_RGBAF16BE`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Rgbaf16,
    frame: Rgbaf16Frame,
    row: Rgbaf16Row,
    sink: Rgbaf16Sink,
    walker: rgbaf16_to,
    walker_endian: rgbaf16_to_endian,
    buf_field: rgba,
    elem_type: half::f16,
    row_elems: |w| w * 4,
    row_doc: "One row of an [`Rgbaf16`] source — `width * 4` packed\n\
              `half::f16` samples (`R, G, B, A` per pixel; alpha is real).\n\
              The Row type is **not** parameterized on `BE` — it just\n\
              borrows the underlying byte slice; the kernel's BE-aware\n\
              byte-swap is monomorphized via the parent `Rgbaf16<BE>` marker.",
    walker_doc: "Walks an [`Rgbaf16Frame`](crate::frame::Rgbaf16Frame) row by row into the sink.\n\
                 The `<const BE>` parameter is propagated from the\n\
                 frame to the sink-trait bound (`S: Rgbaf16Sink<BE>`)\n\
                 so the row-kernel call inside `process` monomorphizes\n\
                 against the same byte order.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
