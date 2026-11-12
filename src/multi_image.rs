use crate::error::QuatImageError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiQuatImage {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>, // 1 byte per pixel: 4 channels packed (2 bits each -> c3:c2:c1:c0)
}

impl MultiQuatImage {
    /// Creates a new multi-channel quaternary image with specified dimensions.
    pub fn new(width: usize, height: usize) -> Result<Self, QuatImageError> {
        if width == 0 || height == 0 {
            return Err(QuatImageError::InvalidDimensions);
        }
        let total_pixels = width
            .checked_mul(height)
            .ok_or(QuatImageError::IntegerOverflow)?;

        Ok(Self {
            width,
            height,
            data: vec![0; total_pixels],
        })
    }

    /// Returns the width of the image.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns a slice containing the raw byte data of the image.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable slice containing the raw byte data of the image.
    pub fn as_mut_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Safely gets all 4 channels for a pixel at (x, y).
    pub fn get_pixel_channels(&self, x: usize, y: usize) -> Result<(u8, u8, u8, u8), QuatImageError> {
        if x >= self.width || y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        let index = y * self.width + x;
        let pixel = self.data[index];

        let c0 = pixel & 0x03;
        let c1 = (pixel >> 2) & 0x03;
        let c2 = (pixel >> 4) & 0x03;
        let c3 = (pixel >> 6) & 0x03;

        Ok((c0, c1, c2, c3))
    }

    /// Safely sets all 4 channels for a pixel at (x, y).
    pub fn set_pixel_channels(
        &mut self,
        x: usize,
        y: usize,
        c0: u8,
        c1: u8,
        c2: u8,
        c3: u8,
    ) -> Result<(), QuatImageError> {
        if x >= self.width || y >= self.height {
            return Err(QuatImageError::OutOfBounds);
        }
        if c0 > 3 || c1 > 3 || c2 > 3 || c3 > 3 {
            let max_val = std::cmp::max(std::cmp::max(c0, c1), std::cmp::max(c2, c3));
            return Err(QuatImageError::InvalidQuatValue(max_val));
        }

        let index = y * self.width + x;
        self.data[index] = (c3 << 6) | (c2 << 4) | (c1 << 2) | (c0 & 0x03);
        Ok(())
    }
}