//! Packed **RGBAF32** source (FFmpeg `AV_PIX_FMT_RGBAF32{LE,BE}`) — 32-bit
//! float per channel, byte order `R, G, B, A` per pixel (16 bytes /
//! 4 × `f32` per pixel).
//!
//! The alpha-carrying twin of [`super::Rgbf32`] (4 components vs 3). Like the
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
//! - `with_rgb_f32` — **lossless** float pass-through of R, G, B (HDR values
//!   > 1.0 are preserved).
//!
//! HDR values > 1.0 in the source saturate to the output range for
//! every integer output. No tone mapping is applied.

use crate::frame::Rgbaf32Frame;

walker! {
  packed_be {
    /// Zero-sized marker for the packed **RGBAF32** source format.
    /// `<const BE: bool = false>` mirrors the parent
    /// [`Rgbaf32Frame`](crate::frame::Rgbaf32Frame)'s endian flag — `false` (default) selects
    /// `AV_PIX_FMT_RGBAF32LE`, `true` selects `AV_PIX_FMT_RGBAF32BE`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    marker: Rgbaf32,
    frame: Rgbaf32Frame,
    row: Rgbaf32Row,
    sink: Rgbaf32Sink,
    walker: rgbaf32_to,
    walker_endian: rgbaf32_to_endian,
    buf_field: rgba,
    elem_type: f32,
    row_elems: |w| w * 4,
    row_doc: "One row of an [`Rgbaf32`] source — `width * 4` packed\n\
              `f32` samples (`R, G, B, A` per pixel; alpha is real). The\n\
              Row type is **not** parameterized on `BE` — it just borrows\n\
              the underlying byte slice; the kernel's BE-aware byte-swap\n\
              is monomorphized via the parent `Rgbaf32<BE>` marker.",
    walker_doc: "Walks an [`Rgbaf32Frame`](crate::frame::Rgbaf32Frame) row by row into the sink.\n\
                 The `<const BE>` parameter is propagated from the\n\
                 frame to the sink-trait bound (`S: Rgbaf32Sink<BE>`)\n\
                 so the row-kernel call inside `process` monomorphizes\n\
                 against the same byte order.",
  }
}

#[cfg(all(test, feature = "std"))]
mod tests;
