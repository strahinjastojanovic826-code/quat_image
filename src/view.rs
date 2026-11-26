//! # Safety Notes on `unsafe` Blocks in `view.rs`
//!
//! This module uses `unsafe` blocks when reading or materializing zero-copy
//! `QuatImageView` sub-regions to avoid repeated bounds checking inside inner
//! pixel loops.
//!
//! ## Safety Invariants Assumed Across `unsafe` Blocks
//! - **Sub-region coordinate bounds:** `QuatImageView` only covers valid
//!   `[origin_x..origin_x + width)` and `[origin_y..origin_y + height)` ranges
//!   within `self.source`, so `self.origin_x + x` and `self.origin_y + y` for
//!   `x in 0..self.width` and `y in 0..self.height` stay strictly in bounds.
//! - **Materialized buffer bounds:** In `to_owned_image`, `out` is allocated with
//!   dimensions `(self.width, self.height)`, ensuring `out.set_pixel_unchecked(x, y, val)`
//!   writes within valid coordinates and `0..=3` quaternary values.

use crate::error::QuatImageError;
use crate::image::QuatImage;

/// Zero-copy read-only window over a rectangular region of a `QuatImage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuatImageView<'a> {
    source: &'a QuatImage,
    origin_x: usize,
    origin_y: usize,
    width: usize,
    height: usize,
}

impl<'a> QuatImageView<'a> {
    /// Creates a zero-copy view starting at `(origin_x, origin_y)` with size `(width, height)`.
    pub fn new(
        source: &'a QuatImage,
        origin_x: usize,
        origin_y: usize,
        width: usize,
        height: usize,
    ) -> Result<Self, QuatImageError> {
        if width == 0 || height == 0 {
            return Err(QuatImageError::InvalidDimensions);
        }
        let end_x = origin_x
            .checked_add(width)
            .ok_or(QuatImageError::IntegerOverflow)?;
        let end_y = origin_y
            .checked_add(height)
            .ok_or(QuatImageError::IntegerOverflow)?;

        if end_x > source.width() || end_y > source.height() {
            return Err(QuatImageError::OutOfBounds);
        }

        Ok(Self {
            source,
            origin_x,
            origin_y,
            width,
            height,
        })
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns a nested zero-copy view relative to this view.
    pub fn subview(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Self, QuatImageError> {
        if width == 0 || height == 0 {
            return Err(QuatImageError::InvalidDimensions);
        }
        let end_x = x.checked_add(width).ok_or(QuatImageError::IntegerOverflow)?;
        let end_y = y.checked_add(height).ok_or(QuatImageError::IntegerOverflow)?;
        if end_x > self.width || end_y > self.height {
            return Err(QuatImageError::OutOfBounds);
        }

        Self::new(
            self.source,
            self.origin_x + x,
            self.origin_y + y,
            width,
            height,
        )
    }

    /// Safely reads a quaternary pixel (`0..=3`) at view-local coordinates `(x, y)`.
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<u8, QuatImageError> {
        if x >= self.width || y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        self.source.get_pixel(self.origin_x + x, self.origin_y + y)
    }

    /// Computes histogram `[count_0, count_1, count_2, count_3]` over this view without copying.
    pub fn histogram(&self) -> [usize; 4] {
        let mut counts = [0usize; 4];
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe {
                    self.source
                        .get_pixel_unchecked(self.origin_x + x, self.origin_y + y)
                };
                if val < 4 {
                    counts[val as usize] += 1;
                }
            }
        }
        counts
    }

    /// Materializes this zero-copy view into a standalone owned `QuatImage`.
    pub fn to_owned_image(&self) -> Result<QuatImage, QuatImageError> {
        let mut out = QuatImage::new(self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe {
                    self.source
                        .get_pixel_unchecked(self.origin_x + x, self.origin_y + y)
                };
                unsafe { out.set_pixel_unchecked(x, y, val) };
            }
        }
        Ok(out)
    }
}

impl QuatImage {
    /// Creates a zero-copy view over the entire image.
    pub fn as_view(&self) -> Result<QuatImageView<'_>, QuatImageError> {
        QuatImageView::new(self, 0, 0, self.width, self.height)
    }

    /// Creates a zero-copy view over a sub-rectangle without allocating or copying.
    pub fn view(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<QuatImageView<'_>, QuatImageError> {
        QuatImageView::new(self, x, y, width, height)
    }
}