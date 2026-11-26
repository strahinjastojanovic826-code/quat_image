//! # Safety Notes on `unsafe` Blocks in `image.rs`
//!
//! This module uses `unsafe` blocks primarily to avoid redundant bounds checks and
//! repeated bit-unpacking/packing overhead on hot loops over packed 2-bit quaternary
//! pixel buffers (`Vec<u8>`, 4 pixels per byte).
//!
//! ## Safety Invariants Assumed Across `unsafe` Blocks
//! - **Bounds safety:** Whenever `get_pixel_unchecked` or `set_pixel_unchecked` is
//!   called, coordinates `(x, y)` have already been validated or constrained within
//!   `0..width` and `0..height` (for example via prior bounds checks, loop ranges
//!   over `0..width` / `0..height`, clamped `isize` neighborhood checks in `dilate` and
//!   `erode`, clamped `copy_w` / `copy_h` bounds in `blit` and `blit_masked`, or
//!   explicit boundary guards in `flood_fill` and `connected_components`).
//! - **Buffer capacity:** `self.data` (and destination/temporary `QuatImage` buffers)
//!   is allocated with length `(width * height + 3) / 4` (or `width * height` in
//!   `from_grayscale8`), ensuring `pixel_index / 4` stays within `self.data` for
//!   valid coordinates.
//! - **Quaternary value range:** Pixel values written via `set_pixel_unchecked` are
//!   expected to be in `0..=3` (validated in `set_pixel`, `flood_fill`, or produced
//!   by operations clamped/quantized to `0..=3`). Note: inside `set_pixel_unchecked`,
//!   `pack_quats(...).unwrap_unchecked()` assumes all four unpacked/assigned values are
//!   `<= 3`; replacing it with a safe fallback (`unwrap_or(0)`) is recommended to
//!   avoid undefined behavior if an out-of-range value is ever passed directly.

use crate::packing::{pack_quats, unpack_quats};
use crate::error::QuatImageError;
use rayon::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuatImage {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) data: Vec<u8>,
}

impl QuatImage {
    /// Kreira novu sliku sa zadatim dimenzijama.
    pub fn new(width: usize, height: usize) -> Result<Self, QuatImageError> {
        if width == 0 || height == 0 {
            return Err(QuatImageError::InvalidDimensions);
        }
        let total_pixels = width.checked_mul(height).ok_or(QuatImageError::IntegerOverflow)?;
        let data_len = (total_pixels + 3) / 4;
        Ok(Self {
            width,
            height,
            data: vec![0; data_len],
        })
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn as_bytes(&self) -> &[u8] { &self.data }
    pub fn as_mut_bytes(&mut self) -> &mut [u8] { &mut self.data }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Bezbedno čitanje piksela sa proverom granica
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<u8, QuatImageError> {
        if x >= self.width || y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        Ok(unsafe { self.get_pixel_unchecked(x, y) })
    }

    /// Brzo čitanje bez provere granica (internal performance boost)
    #[inline]
    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> u8 { unsafe {
        let pixel_index = y * self.width + x;
        let byte_index = pixel_index / 4;
        let sub_index = pixel_index % 4;
        let byte = *self.data.get_unchecked(byte_index);
        let (q0, q1, q2, q3) = unpack_quats(byte);
        match sub_index {
            0 => q0,
            1 => q1,
            2 => q2,
            _ => q3,
        }
    }}

    pub fn set_pixel(&mut self, x: usize, y: usize, val: u8) -> Result<(), QuatImageError> {
        if x >= self.width || y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        if val > 3 {
            return Err(QuatImageError::InvalidQuatValue(val));
        }
        unsafe { self.set_pixel_unchecked(x, y, val); }
        Ok(())
    }

    pub fn dilate(&self) -> Result<Self, QuatImageError> {
    let mut result = Self::new(self.width, self.height)?;
    for y in 0..self.height {
        for x in 0..self.width {
            let mut max_val = 0;
            for dy in -1..=1 {
                for dy_u in [dy].iter() {
                    // Safe neighborhood boundary checking
                    let ny = y as isize + *dy_u;
                    for dx in -1..=1 {
                        let nx = x as isize + dx;
                        if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                            let val = unsafe { self.get_pixel_unchecked(nx as usize, ny as usize) };
                            if val > max_val {
                                max_val = val;
                            }
                        }
                    }
                }
            }
            unsafe { result.set_pixel_unchecked(x, y, max_val); }
        }
    }
    Ok(result)
}

    pub fn crop(&self, x: usize, y: usize, crop_width: usize, crop_height: usize) -> Result<Self, QuatImageError> {
        if x + crop_width > self.width || y + crop_height > self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        if crop_width == 0 || crop_height == 0 {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut cropped = Self::new(crop_width, crop_height)?;
        for cy in 0..crop_height {
            for cx in 0..crop_width {
                let val = unsafe { self.get_pixel_unchecked(x + cx, y + cy) };
                unsafe {
                    cropped.set_pixel_unchecked(cx, cy, val);
                }
            }
        }
        Ok(cropped)
    }

    pub fn erode(&self) -> Result<QuatImage, QuatImageError> {
        let mut result = QuatImage::new(self.width, self.height)?;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let mut min_val: u8 = 3;
                
                for dy in -1..=1 {
                    let ny = y as isize + dy;
                    if ny >= 0 && ny < self.height as isize {
                        for dx in -1..=1 {
                            let nx = x as isize + dx;
                            if nx >= 0 && nx < self.width as isize {
                                let val = unsafe { self.get_pixel_unchecked(nx as usize, ny as usize) };
                                if val < min_val {
                                    min_val = val;
                                }
                            }
                        }
                    }
                }
                unsafe {
                    result.set_pixel_unchecked(x, y, min_val);
                }
            }
        }
        Ok(result)
    }

    pub fn opening(&self) -> Result<QuatImage, QuatImageError> {
        let eroded = self.erode()?;
        eroded.dilate()
    }

    pub fn closing(&self) -> Result<QuatImage, QuatImageError> {
        let dilated = self.dilate()?;
        dilated.erode()
    }

    pub fn replace_color_par(&mut self, target: u8, replacement: u8) -> Result<(), QuatImageError> {
    if target > 3 || replacement > 3 {
        return Err(QuatImageError::InvalidQuatValue(if target > 3 { target } else { replacement }));
    }
    if target == replacement {
        return Ok(());
    }

    self.data.par_iter_mut().try_for_each(|byte| -> Result<(), QuatImageError> {
        let (q0, q1, q2, q3) = unpack_quats(*byte);
        let mut modified = false;
        let mut nq0 = q0;
        let mut nq1 = q1;
        let mut nq2 = q2;
        let mut nq3 = q3;

        if q0 == target { nq0 = replacement; modified = true; }
        if q1 == target { nq1 = replacement; modified = true; }
        if q2 == target { nq2 = replacement; modified = true; }
        if q3 == target { nq3 = replacement; modified = true; }

        if modified {
            *byte = pack_quats(nq0, nq1, nq2, nq3)?;
        }
        Ok(())
    })?;

    Ok(())
}

    pub fn histogram(&self) -> [usize; 4] {
        let mut counts = [0; 4];
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe { self.get_pixel_unchecked(x, y) } as usize;
                if val < 4 {
                    counts[val] += 1;
                }
            }
        }
        counts
    }

    pub fn from_rgb8(width: usize, height: usize, rgb_data: &[u8]) -> Result<Self, QuatImageError> {
    if rgb_data.len() != width * height * 3 {
        return Err(QuatImageError::InvalidDimensions);
    }
    
    let mut img = Self::new(width, height)?;
    for (i, chunk) in rgb_data.chunks_exact(3).enumerate() {
        let x = i % width;
        let y = i / width;
        
        // Standard luminance calculation (Rec. 709)
        let gray = (0.2126 * chunk[0] as f32 + 0.7152 * chunk[1] as f32 + 0.0722 * chunk[2] as f32) as u8;
        
        // Quantize 0..255 range into 4 quaternary levels (0, 1, 2, 3)
        let quat_val = match gray {
            0..=63 => 0,
            64..=127 => 1,
            128..=191 => 2,
            _ => 3,
        };
        
        img.set_pixel(x, y, quat_val)?;
    }
    
    Ok(img)
}

/// Quantizes an 8-bit grayscale buffer into a `QuatImage`.
    pub fn from_grayscale8(
        width: usize,
        height: usize,
        gray_data: &[u8],
    ) -> Result<Self, QuatImageError> {
        if width == 0 || height == 0 {
            return Err(QuatImageError::InvalidDimensions);
        }
        let expected = width
            .checked_mul(height)
            .ok_or(QuatImageError::IntegerOverflow)?;
        if gray_data.len() != expected {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut img = Self::new(width, height)?;
        for (i, &g) in gray_data.iter().enumerate() {
            let x = i % width;
            let y = i / width;
            let quat_val = g >> 6; // Maps 0..=255 cleanly into 0..=3
            unsafe { img.set_pixel_unchecked(x, y, quat_val) };
        }
        Ok(img)
    }

    /// Expands quaternary values (0..=3) into an 8-bit grayscale buffer (0, 85, 170, 255).
    pub fn to_grayscale8(&self) -> Vec<u8> {
        const LEVELS: [u8; 4] = [0, 85, 170, 255];
        let mut out = Vec::with_capacity(self.width * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe { self.get_pixel_unchecked(x, y) } as usize;
                out.push(LEVELS[val]);
            }
        }
        out
    }

    /// Expands quaternary values into an RGB8 buffer using a 4-color `[R, G, B]` palette.
    pub fn to_rgb8(&self, palette: [[u8; 3]; 4]) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.width * self.height * 3);
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe { self.get_pixel_unchecked(x, y) } as usize;
                out.extend_from_slice(&palette[val]);
            }
        }
        out
    }

    /// Counts the number of pixels that differ between two images of equal dimensions.
    pub fn diff_count(&self, other: &Self) -> Result<usize, QuatImageError> {
        if self.width != other.width || self.height != other.height {
            return Err(QuatImageError::InvalidDimensions);
        }
        let mut diffs = 0usize;
        for y in 0..self.height {
            for x in 0..self.width {
                let a = unsafe { self.get_pixel_unchecked(x, y) };
                let b = unsafe { other.get_pixel_unchecked(x, y) };
                if a != b {
                    diffs += 1;
                }
            }
        }
        Ok(diffs)
    }

    /// Computes Peak Signal-to-Noise Ratio (PSNR in dB) for quaternary signal (MAX = 3).
    pub fn psnr(&self, other: &Self) -> Result<f64, QuatImageError> {
        if self.width != other.width || self.height != other.height {
            return Err(QuatImageError::InvalidDimensions);
        }
        let mut sq_err_sum = 0u64;
        for y in 0..self.height {
            for x in 0..self.width {
                let a = unsafe { self.get_pixel_unchecked(x, y) } as i64;
                let b = unsafe { other.get_pixel_unchecked(x, y) } as i64;
                let diff = a - b;
                sq_err_sum += (diff * diff) as u64;
            }
        }
        if sq_err_sum == 0 {
            return Ok(f64::INFINITY);
        }
        let total_pixels = (self.width * self.height) as f64;
        let mse = sq_err_sum as f64 / total_pixels;
        let max_val = 3.0_f64;
        Ok(10.0 * ((max_val * max_val) / mse).log10())
    }

    /// Computes Intersection over Union (IoU) for a specific quaternary class `target_val` (0..=3).
    pub fn iou(&self, other: &Self, target_val: u8) -> Result<f64, QuatImageError> {
        if target_val > 3 {
            return Err(QuatImageError::InvalidQuatValue(target_val));
        }
        if self.width != other.width || self.height != other.height {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut intersection = 0usize;
        let mut union = 0usize;
        for y in 0..self.height {
            for x in 0..self.width {
                let a = unsafe { self.get_pixel_unchecked(x, y) } == target_val;
                let b = unsafe { other.get_pixel_unchecked(x, y) } == target_val;
                if a && b {
                    intersection += 1;
                }
                if a || b {
                    union += 1;
                }
            }
        }

        if union == 0 {
            Ok(1.0)
        } else {
            Ok(intersection as f64 / union as f64)
        }
    }

    pub fn rotate_90_cw(&mut self) -> Result<(), QuatImageError> {
        let mut new_img = Self::new(self.height, self.width)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe { self.get_pixel_unchecked(x, y) };
                let new_x = y;
                let new_y = self.width - 1 - x;
                unsafe {
                    new_img.set_pixel_unchecked(new_x, new_y, val);
                }
            }
        }
        self.width = new_img.width;
        self.height = new_img.height;
        self.data = new_img.data;
        Ok(())
    }

    pub fn replace_color(&mut self, target: u8, replacement: u8) -> Result<(), QuatImageError> {
        if target > 3 || replacement > 3 {
            return Err(QuatImageError::InvalidQuatValue(if target > 3 { target } else { replacement }));
        }
        if target == replacement {
            return Ok(());
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe { self.get_pixel_unchecked(x, y) };
                if val == target {
                    unsafe {
                        self.set_pixel_unchecked(x, y, replacement);
                    }
                }
            }
        }
        Ok(())
    }

    #[inline]
    pub unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, val: u8) { unsafe {
        let pixel_index = y * self.width + x;
        let byte_index = pixel_index / 4;
        let sub_index = pixel_index % 4;
        let byte = *self.data.get_unchecked(byte_index);
        let (mut q0, mut q1, mut q2, mut q3) = unpack_quats(byte);
        match sub_index {
            0 => q0 = val,
            1 => q1 = val,
            2 => q2 = val,
            _ => q3 = val,
        }
        let packed = pack_quats(q0, q1, q2, q3).unwrap_unchecked();
        *self.data.get_unchecked_mut(byte_index) = packed;
    }}

    /// Optimizovan horizontalni flip korišćenjem unutrašnjih brzih metoda
    pub fn flip_horizontal(&mut self) -> Result<(), QuatImageError> {
        let mut new_img = Self::new(self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe { self.get_pixel_unchecked(x, y) };
                unsafe { new_img.set_pixel_unchecked(self.width - 1 - x, y, val); }
            }
        }
        self.data = new_img.data;
        Ok(())
    }

    pub fn flip_vertical(&mut self) -> Result<(), QuatImageError> {
        let mut new_img = Self::new(self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let val = unsafe { self.get_pixel_unchecked(x, y) };
                unsafe { new_img.set_pixel_unchecked(x, self.height - 1 - y, val); }
            }
        }
        self.data = new_img.data;
        Ok(())
    }

    /// Copies `src` into `self` at top-left offset `(dst_x, dst_y)`, clipping at boundaries.
    pub fn blit(&mut self, src: &Self, dst_x: usize, dst_y: usize) -> Result<(), QuatImageError> {
        if dst_x >= self.width || dst_y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        let copy_w = src.width.min(self.width - dst_x);
        let copy_h = src.height.min(self.height - dst_y);

        for y in 0..copy_h {
            for x in 0..copy_w {
                let val = unsafe { src.get_pixel_unchecked(x, y) };
                unsafe { self.set_pixel_unchecked(dst_x + x, dst_y + y, val) };
            }
        }
        Ok(())
    }

    /// Copies `src` into `self` at `(dst_x, dst_y)`, skipping pixels equal to `transparent_val`.
    pub fn blit_masked(
        &mut self,
        src: &Self,
        dst_x: usize,
        dst_y: usize,
        transparent_val: u8,
    ) -> Result<(), QuatImageError> {
        if transparent_val > 3 {
            return Err(QuatImageError::InvalidQuatValue(transparent_val));
        }
        if dst_x >= self.width || dst_y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        let copy_w = src.width.min(self.width - dst_x);
        let copy_h = src.height.min(self.height - dst_y);

        for y in 0..copy_h {
            for x in 0..copy_w {
                let val = unsafe { src.get_pixel_unchecked(x, y) };
                if val != transparent_val {
                    unsafe { self.set_pixel_unchecked(dst_x + x, dst_y + y, val) };
                }
            }
        }
        Ok(())
    }

    /// 4-connected flood fill starting at `(x, y)` with `new_val`. Returns number of modified pixels.
    pub fn flood_fill(
        &mut self,
        start_x: usize,
        start_y: usize,
        new_val: u8,
    ) -> Result<usize, QuatImageError> {
        if start_x >= self.width || start_y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        if new_val > 3 {
            return Err(QuatImageError::InvalidQuatValue(new_val));
        }

        let target_val = unsafe { self.get_pixel_unchecked(start_x, start_y) };
        if target_val == new_val {
            return Ok(0);
        }

        let mut queue = VecDeque::new();
        queue.push_back((start_x, start_y));
        unsafe { self.set_pixel_unchecked(start_x, start_y, new_val) };
        let mut filled = 1;

        while let Some((x, y)) = queue.pop_front() {
            if x > 0 && unsafe { self.get_pixel_unchecked(x - 1, y) } == target_val {
                unsafe { self.set_pixel_unchecked(x - 1, y, new_val) };
                filled += 1;
                queue.push_back((x - 1, y));
            }
            if x + 1 < self.width && unsafe { self.get_pixel_unchecked(x + 1, y) } == target_val {
                unsafe { self.set_pixel_unchecked(x + 1, y, new_val) };
                filled += 1;
                queue.push_back((x + 1, y));
            }
            if y > 0 && unsafe { self.get_pixel_unchecked(x, y - 1) } == target_val {
                unsafe { self.set_pixel_unchecked(x, y - 1, new_val) };
                filled += 1;
                queue.push_back((x, y - 1));
            }
            if y + 1 < self.height && unsafe { self.get_pixel_unchecked(x, y + 1) } == target_val {
                unsafe { self.set_pixel_unchecked(x, y + 1, new_val) };
                filled += 1;
                queue.push_back((x, y + 1));
            }
        }

        Ok(filled)
    }

    /// Labels 4-connected foreground components (pixels > 0).
    /// Returns `(label_map, component_count)` where `0` is background.
    pub fn connected_components(&self) -> (Vec<u32>, u32) {
        let total = self.width * self.height;
        let mut labels = vec![0u32; total];
        let mut current_label = 0u32;
        let mut queue = VecDeque::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let val = unsafe { self.get_pixel_unchecked(x, y) };
                if val > 0 && labels[idx] == 0 {
                    current_label += 1;
                    labels[idx] = current_label;
                    queue.push_back((x, y));

                    while let Some((cx, cy)) = queue.pop_front() {
                        if cx > 0 {
                            let n_idx = cy * self.width + (cx - 1);
                            if labels[n_idx] == 0 && unsafe { self.get_pixel_unchecked(cx - 1, cy) } > 0 {
                                labels[n_idx] = current_label;
                                queue.push_back((cx - 1, cy));
                            }
                        }
                        if cx + 1 < self.width {
                            let n_idx = cy * self.width + (cx + 1);
                            if labels[n_idx] == 0 && unsafe { self.get_pixel_unchecked(cx + 1, cy) } > 0 {
                                labels[n_idx] = current_label;
                                queue.push_back((cx + 1, cy));
                            }
                        }
                        if cy > 0 {
                            let n_idx = (cy - 1) * self.width + cx;
                            if labels[n_idx] == 0 && unsafe { self.get_pixel_unchecked(cx, cy - 1) } > 0 {
                                labels[n_idx] = current_label;
                                queue.push_back((cx, cy - 1));
                            }
                        }
                        if cy + 1 < self.height {
                            let n_idx = (cy + 1) * self.width + cx;
                            if labels[n_idx] == 0 && unsafe { self.get_pixel_unchecked(cx, cy + 1) } > 0 {
                                labels[n_idx] = current_label;
                                queue.push_back((cx, cy + 1));
                            }
                        }
                    }
                }
            }
        }

        (labels, current_label)
    }

    /// Inverts quaternary values in-place (0 <-> 3, 1 <-> 2).
    pub fn invert(&mut self) {
        for byte in &mut self.data {
            *byte = !*byte;
        }
        self.mask_trailing_bits();
    }

    /// Parallel in-place quaternary inversion using Rayon.
    pub fn invert_par(&mut self) {
        self.data.par_iter_mut().for_each(|byte| {
            *byte = !*byte;
        });
        self.mask_trailing_bits();
    }

    /// Applies a 4-entry lookup table (LUT) to remap every quaternary value (0..=3).
    pub fn remap(&mut self, lut: [u8; 4]) -> Result<(), QuatImageError> {
        if let Some(&invalid) = lut.iter().find(|&&v| v > 3) {
            return Err(QuatImageError::InvalidQuatValue(invalid));
        }

        // Precompute a 256-entry byte-level LUT so 4 pixels are mapped in 1 memory lookup
        let mut byte_lut = [0u8; 256];
        for (b, entry) in byte_lut.iter_mut().enumerate() {
            let (q0, q1, q2, q3) = unpack_quats(b as u8);
            *entry = (lut[q3 as usize] << 6)
                | (lut[q2 as usize] << 4)
                | (lut[q1 as usize] << 2)
                | lut[q0 as usize];
        }

        for byte in &mut self.data {
            *byte = byte_lut[*byte as usize];
        }
        self.mask_trailing_bits();
        Ok(())
    }

    /// Element-wise quaternary bitwise AND with another image of the same dimensions.
    pub fn bitwise_and(&self, other: &Self) -> Result<Self, QuatImageError> {
        if self.width != other.width || self.height != other.height {
            return Err(QuatImageError::InvalidDimensions);
        }
        let data = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(&a, &b)| a & b)
            .collect();
        Ok(Self {
            width: self.width,
            height: self.height,
            data,
        })
    }

    /// Element-wise quaternary bitwise OR with another image of the same dimensions.
    pub fn bitwise_or(&self, other: &Self) -> Result<Self, QuatImageError> {
        if self.width != other.width || self.height != other.height {
            return Err(QuatImageError::InvalidDimensions);
        }
        let data = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(&a, &b)| a | b)
            .collect();
        Ok(Self {
            width: self.width,
            height: self.height,
            data,
        })
    }

    /// Element-wise quaternary bitwise XOR with another image of the same dimensions.
    pub fn bitwise_xor(&self, other: &Self) -> Result<Self, QuatImageError> {
        if self.width != other.width || self.height != other.height {
            return Err(QuatImageError::InvalidDimensions);
        }
        let data = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(&a, &b)| a ^ b)
            .collect();
        Ok(Self {
            width: self.width,
            height: self.height,
            data,
        })
    }

    /// Thresholds quaternary pixels: values >= threshold become `high`, others become `low`.
    pub fn threshold(&mut self, threshold: u8, low: u8, high: u8) -> Result<(), QuatImageError> {
        if threshold > 3 {
            return Err(QuatImageError::InvalidQuatValue(threshold));
        }
        if low > 3 {
            return Err(QuatImageError::InvalidQuatValue(low));
        }
        if high > 3 {
            return Err(QuatImageError::InvalidQuatValue(high));
        }
        let lut = [
            if 0 >= threshold { high } else { low },
            if 1 >= threshold { high } else { low },
            if 2 >= threshold { high } else { low },
            if 3 >= threshold { high } else { low },
        ];
        self.remap(lut)
    }

    /// Ensures unused padding bits in the last byte remain zeroed when total_pixels % 4 != 0.
    #[inline]
    fn mask_trailing_bits(&mut self) {
        let rem = (self.width * self.height) % 4;
        if rem != 0 {
            if let Some(last) = self.data.last_mut() {
                let valid_bits = (rem * 2) as u32;
                let mask = (1u8 << valid_bits) - 1;
                *last &= mask;
            }
        }
    }

}

impl AsRef<[u8]> for QuatImage {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl AsMut<[u8]> for QuatImage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}