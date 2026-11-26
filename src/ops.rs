//! # Safety Notes on `unsafe` Blocks in `ops.rs`
//!
//! This module uses `unsafe` blocks in cache-tiled rotation (`rotate_90_cw_tiled`)
//! and parallel morphological filters (`dilate_par`, `erode_par`) to avoid redundant
//! bounds checks over validated coordinates.
//!
//! ## Safety Invariants Assumed Across `unsafe` Blocks
//! - **Tiled rotation coordinates:** In `rotate_90_cw_tiled`, `by` and `bx` are
//!   clamped by `y_end` and `x_end` against `self.height` and `self.width`, so
//!   source coordinates `(x, y)` and rotated destination coordinates
//!   `(new_x, new_y)` with `new_x = y` and `new_y = self.width - 1 - x` always
//!   stay within `new_img` (`self.height` x `self.width`).
//! - **Clamped 3x3 neighborhoods in parallel morphology:** In `dilate_par` and
//!   `erode_par`, `y_start`/`y_end` are bounded within `0..height` and
//!   `x_start`/`x_end` within `0..width`, guaranteeing that `(nx, ny)` in
//!   `get_pixel_unchecked(nx, ny)` is always valid across parallel row workers.

use crate::error::QuatImageError;
use crate::image::QuatImage;
use crate::packing::{pack_quats, unpack_quats};
use rayon::prelude::*;

impl QuatImage {
    /// Cache-friendly 90-degree clockwise rotation using 16x16 L1 cache tiles.
    pub fn rotate_90_cw_tiled(&mut self) -> Result<(), QuatImageError> {
        const TILE: usize = 16;
        let mut new_img = Self::new(self.height, self.width)?;

        for by in (0..self.height).step_by(TILE) {
            let y_end = (by + TILE).min(self.height);
            for bx in (0..self.width).step_by(TILE) {
                let x_end = (bx + TILE).min(self.width);

                for y in by..y_end {
                    let new_x = y;
                    for x in bx..x_end {
                        let val = unsafe { self.get_pixel_unchecked(x, y) };
                        let new_y = self.width - 1 - x;
                        unsafe { new_img.set_pixel_unchecked(new_x, new_y, val) };
                    }
                }
            }
        }

        self.width = new_img.width;
        self.height = new_img.height;
        self.data = new_img.data;
        Ok(())
    }

    /// Parallel, cache-friendly 3x3 morphological dilation (row-parallel, lock-free).
    pub fn dilate_par(&self) -> Result<Self, QuatImageError> {
        let mut result = Self::new(self.width, self.height)?;
        let width = self.width;
        let height = self.height;

        // Compute row-by-row in parallel; each thread owns its output row buffer
        let rows: Vec<Vec<u8>> = (0..height)
            .into_par_iter()
            .map(|y| {
                let mut row_vals = vec![0u8; width];
                let y_start = y.saturating_sub(1);
                let y_end = (y + 1).min(height - 1);

                for (x, out_val) in row_vals.iter_mut().enumerate() {
                    let x_start = x.saturating_sub(1);
                    let x_end = (x + 1).min(width - 1);
                    let mut max_val = 0u8;

                    for ny in y_start..=y_end {
                        for nx in x_start..=x_end {
                            let val = unsafe { self.get_pixel_unchecked(nx, ny) };
                            if val > max_val {
                                max_val = val;
                                if max_val == 3 {
                                    break;
                                }
                            }
                        }
                        if max_val == 3 {
                            break;
                        }
                    }
                    *out_val = max_val;
                }
                row_vals
            })
            .collect();

        // Pack sequential rows linearly into result buffer
        let mut bit_cursor = 0usize;
        for row in rows {
            for val in row {
                let byte_idx = bit_cursor / 4;
                let shift = (bit_cursor % 4) * 2;
                result.data[byte_idx] |= (val & 0x03) << shift;
                bit_cursor += 1;
            }
        }

        Ok(result)
    }

    /// Parallel, cache-friendly 3x3 morphological erosion (row-parallel, lock-free).
    pub fn erode_par(&self) -> Result<Self, QuatImageError> {
        let mut result = Self::new(self.width, self.height)?;
        let width = self.width;
        let height = self.height;

        let rows: Vec<Vec<u8>> = (0..height)
            .into_par_iter()
            .map(|y| {
                let mut row_vals = vec![0u8; width];
                let y_start = y.saturating_sub(1);
                let y_end = (y + 1).min(height - 1);

                for (x, out_val) in row_vals.iter_mut().enumerate() {
                    let x_start = x.saturating_sub(1);
                    let x_end = (x + 1).min(width - 1);
                    let mut min_val = 3u8;

                    for ny in y_start..=y_end {
                        for nx in x_start..=x_end {
                            let val = unsafe { self.get_pixel_unchecked(nx, ny) };
                            if val < min_val {
                                min_val = val;
                                if min_val == 0 {
                                    break;
                                }
                            }
                        }
                        if min_val == 0 {
                            break;
                        }
                    }
                    *out_val = min_val;
                }
                row_vals
            })
            .collect();

        let mut bit_cursor = 0usize;
        for row in rows {
            for val in row {
                let byte_idx = bit_cursor / 4;
                let shift = (bit_cursor % 4) * 2;
                result.data[byte_idx] |= (val & 0x03) << shift;
                bit_cursor += 1;
            }
        }

        Ok(result)
    }

    /// Parallel morphological opening (parallel erosion followed by parallel dilation).
    pub fn opening_par(&self) -> Result<Self, QuatImageError> {
        self.erode_par()?.dilate_par()
    }

    /// Parallel morphological closing (parallel dilation followed by parallel erosion).
    pub fn closing_par(&self) -> Result<Self, QuatImageError> {
        self.dilate_par()?.erode_par()
    }

    /// Parallel byte-level histogram computation (4 pixels unpacked per iteration).
    pub fn histogram_par(&self) -> [usize; 4] {
        let total_pixels = self.width * self.height;
        let full_bytes = total_pixels / 4;
        let rem = total_pixels % 4;

        let mut counts = self.data[..full_bytes]
            .par_iter()
            .fold(
                || [0usize; 4],
                |mut acc, &byte| {
                    let (q0, q1, q2, q3) = unpack_quats(byte);
                    acc[(q0 & 0x03) as usize] += 1;
                    acc[(q1 & 0x03) as usize] += 1;
                    acc[(q2 & 0x03) as usize] += 1;
                    acc[(q3 & 0x03) as usize] += 1;
                    acc
                },
            )
            .reduce(
                || [0usize; 4],
                |a, b| [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]],
            );

        if rem != 0 {
            if let Some(&last_byte) = self.data.last() {
                let (q0, q1, q2, _) = unpack_quats(last_byte);
                let tail = [q0, q1, q2];
                for &q in &tail[..rem] {
                    counts[(q & 0x03) as usize] += 1;
                }
            }
        }

        counts
    }

    /// Parallel Rec. 709 RGB8 -> QuatImage quantization (4 RGB pixels -> 1 packed byte per task).
    pub fn from_rgb8_par(
        width: usize,
        height: usize,
        rgb_data: &[u8],
    ) -> Result<Self, QuatImageError> {
        if width == 0 || height == 0 {
            return Err(QuatImageError::InvalidDimensions);
        }
        let total_pixels = width
            .checked_mul(height)
            .ok_or(QuatImageError::IntegerOverflow)?;
        let expected_rgb = total_pixels
            .checked_mul(3)
            .ok_or(QuatImageError::IntegerOverflow)?;
        if rgb_data.len() != expected_rgb {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut img = Self::new(width, height)?;
        let full_quads = total_pixels / 4;
        let rem_pixels = total_pixels % 4;

        // Each 12 bytes of RGB (4 pixels) maps directly to 1 byte in img.data
        img.data[..full_quads]
            .par_iter_mut()
            .zip(rgb_data[..full_quads * 12].par_chunks_exact(12))
            .for_each(|(out_byte, rgb_quad)| {
                let q0 = rgb_to_quat(&rgb_quad[0..3]);
                let q1 = rgb_to_quat(&rgb_quad[3..6]);
                let q2 = rgb_to_quat(&rgb_quad[6..9]);
                let q3 = rgb_to_quat(&rgb_quad[9..12]);
                *out_byte = pack_quats(q0, q1, q2, q3).unwrap_or(0);
            });

        // Handle remaining 1..=3 trailing pixels
        if rem_pixels > 0 {
            let base_rgb = full_quads * 12;
            let mut tail_quats = [0u8; 4];
            for i in 0..rem_pixels {
                let offset = base_rgb + i * 3;
                tail_quats[i] = rgb_to_quat(&rgb_data[offset..offset + 3]);
            }
            if let Some(last_byte) = img.data.last_mut() {
                *last_byte =
                    pack_quats(tail_quats[0], tail_quats[1], tail_quats[2], tail_quats[3])
                        .unwrap_or(0);
            }
        }

        Ok(img)
    }
}

#[inline]
fn rgb_to_quat(rgb: &[u8]) -> u8 {
    // Fixed-point Rec. 709 approximation: (54*R + 183*G + 19*B) >> 8 -> 0..=255, then >> 6 -> 0..=3
    let lum = (54u32 * rgb[0] as u32 + 183u32 * rgb[1] as u32 + 19u32 * rgb[2] as u32) >> 8;
    (lum >> 6) as u8
}