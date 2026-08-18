//! Walker for the `Gbrpf32` source format (`AV_PIX_FMT_GBRPF32{LE,BE}`) — three
//! full-resolution `f32` planes in **G, B, R** order.
//!
//! Nominal range `[0.0, 1.0]`; HDR values > 1.0 are permitted. Integer
//! outputs clamp to `[0.0, 1.0]` before scaling; float outputs are
//! lossless pass-through.
//!
//! The marker carries `<const BE: bool = false>`: `Gbrpf32`
//! (= `Gbrpf32<false>`) is the LE source; `Gbrpf32<true>` is the BE source.
//! The walker [`gbrpf32_to_endian`] propagates `BE` from
//! [`Gbrpf32Frame<'_, BE>`] into the sinker dispatch.

use crate::{
  PixelSink, SourceFormat,
  frame::{Gbrpf32Frame, Gbrpf32LeFrame},
  source::sealed::Sealed,
};

/// Zero-sized marker for the planar GBR float-32 source format
/// (`AV_PIX_FMT_GBRPF32{LE,BE}`). `<const BE: bool>` defaults to `false`
/// (LE).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Gbrpf32<const BE: bool = false>;

impl<const BE: bool> Sealed for Gbrpf32<BE> {}
impl<const BE: bool> SourceFormat for Gbrpf32<BE> {}

/// One output row from a [`Gbrpf32Frame`](crate::frame::Gbrpf32Frame) — three full-width `f32` slices
/// in G / B / R order. Use [`Self::g`] / [`Self::b`] / [`Self::r`].
///
/// The Row type is **not** parameterized on `BE` — Row is just borrowed
/// samples; the kernel monomorphization picks up `BE` from the sinker
/// type's `MixedSinker<Gbrpf32<BE>>` parameterization.
#[derive(Debug, Clone, Copy)]
pub struct Gbrpf32Row<'a> {
  g: &'a [f32],
  b: &'a [f32],
  r: &'a [f32],
  row: usize,
}

impl<'a> Gbrpf32Row<'a> {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) fn new(g: &'a [f32], b: &'a [f32], r: &'a [f32], row: usize) -> Self {
    Self { g, b, r, row }
  }

  /// Green plane row — `width` `f32` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn g(&self) -> &'a [f32] {
    self.g
  }
  /// Blue plane row — `width` `f32` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn b(&self) -> &'a [f32] {
    self.b
  }
  /// Red plane row — `width` `f32` elements.
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn r(&self) -> &'a [f32] {
    self.r
  }
  /// Output row index within the frame (0-based).
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn row(&self) -> usize {
    self.row
  }
}

/// Sinks that consume rows of a [`Gbrpf32`] source. The `<const BE>`
/// parameter encodes the source byte-order (LE bit pattern vs BE bit
/// pattern of the f32). Defaults to `false` (LE) for back-compat.
pub trait Gbrpf32Sink<const BE: bool = false>:
  for<'a> PixelSink<Input<'a> = Gbrpf32Row<'a>>
{
}

/// Walks a [`Gbrpf32Frame<'_, BE>`] row by row, dispatching each row to the
/// sink. Propagates `<const BE: bool>` from the frame into
/// [`Gbrpf32Sink<BE>`]. Use the LE-only [`gbrpf32_to`] wrapper for
/// pre-Phase-4 explicit-turbofish callers.
pub fn gbrpf32_to_endian<S, const BE: bool>(
  src: &Gbrpf32Frame<'_, BE>,
  sink: &mut S,
) -> Result<(), S::Error>
where
  S: Gbrpf32Sink<BE>,
{
  sink.begin_frame(src.width(), src.height())?;

  let w = src.width() as usize;
  let h = src.height() as usize;
  let g_plane = src.g();
  let b_plane = src.b();
  let r_plane = src.r();
  let g_stride = src.g_stride() as usize;
  let b_stride = src.b_stride() as usize;
  let r_stride = src.r_stride() as usize;

  for row in 0..h {
    let g = &g_plane[row * g_stride..row * g_stride + w];
    let b = &b_plane[row * b_stride..row * b_stride + w];
    let r = &r_plane[row * r_stride..row * r_stride + w];
    sink.process(Gbrpf32Row::new(g, b, r, row))?;
  }
  Ok(())
}

/// LE-only back-compat wrapper preserving the pre-Phase-4 walker
/// signature. Forwards to [`gbrpf32_to_endian`] with `BE = false`.
///
/// Rust forbids defaults on function-position const-generic parameters,
/// so an explicit-turbofish caller written before the Phase-4 BE
/// migration (`gbrpf32_to::<MySink>(...)`) would otherwise fail to
/// compile. Keeping this single-generic wrapper preserves source
/// compatibility for those call sites. BE-aware callers should use
/// [`gbrpf32_to_endian`] directly.
#[cfg_attr(not(tarpaulin), inline(always))]
pub fn gbrpf32_to<S>(src: &Gbrpf32LeFrame<'_>, sink: &mut S) -> Result<(), S::Error>
where
  S: Gbrpf32Sink<false>,
{
  gbrpf32_to_endian::<S, false>(src, sink)
}

#[cfg(all(test, feature = "std"))]
mod tests;
