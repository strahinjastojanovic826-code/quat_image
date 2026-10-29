use crate::packing::{pack_quats, unpack_quats};
use crate::error::QuatImageError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuatImage {
    width: usize,
    height: usize,
    data: Vec<u8>,
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

    pub fn replace_color_par(&mut self, target: u8, replacement: u8) -> Result<(), QuatImageError> {
    if target > 3 || replacement > 3 {
        return Err(QuatImageError::InvalidQuatValue(if target > 3 { target } else { replacement }));
    }
    if target == replacement {
        return Ok(());
    }

    use rayon::prelude::*;

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