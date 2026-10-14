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