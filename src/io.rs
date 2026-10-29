use crate::error::QuatImageError;
use crate::image::QuatImage;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const MAGIC_HEADER: &[u8; 4] = b"QIMG"; // Magic identifier for binary file

impl QuatImage {
    /// Saves the compressed QuatImage to a binary file with header.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let mut file = File::create(path)?;
        
        // Write Magic bytes
        file.write_all(MAGIC_HEADER)?;
        
        // Write Width and Height as u64 Little Endian
        file.write_all(&(self.width() as u64).to_le_bytes())?;
        file.write_all(&(self.height() as u64).to_le_bytes())?;
        
        // Write packed binary data
        file.write_all(self.as_bytes())?;
        Ok(())
    }

    /// Loads a QuatImage from a binary file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, QuatImageError> {
        let mut file = File::open(path).map_err(|_| QuatImageError::OutOfBounds)?;
        
        let mut header = [0u8; 4];
        file.read_exact(&mut header).map_err(|_| QuatImageError::OutOfBounds)?;
        if &header != MAGIC_HEADER {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut width_bytes = [0u8; 8];
        let mut height_bytes = [0u8; 8];
        file.read_exact(&mut width_bytes).map_err(|_| QuatImageError::OutOfBounds)?;
        file.read_exact(&mut height_bytes).map_err(|_| QuatImageError::OutOfBounds)?;

        let width = u64::from_le_bytes(width_bytes) as usize;
        let height = u64::from_le_bytes(height_bytes) as usize;

        // 1. Kreiramo sliku preko javne metode new()
        let mut img = Self::new(width, height)?;

        // 2. Učitavamo bajtove direktno u njen mutabilni bafer
        file.read_exact(img.as_mut_bytes()).map_err(|_| QuatImageError::OutOfBounds)?;

        Ok(img)
    }
    pub fn save_to_file_rle<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    file.write_all(b"QRLE")?; // Special RLE Magic Header
    file.write_all(&(self.width() as u64).to_le_bytes())?;
    file.write_all(&(self.height() as u64).to_le_bytes())?;

    let data = self.as_bytes();
    let mut encoded: Vec<u8> = Vec::new();
    
    if !data.is_empty() {
        let mut current_byte = data[0];
        let mut count: u8 = 1;

        for &b in &data[1..] {
            if b == current_byte && count < 255 {
                count += 1;
            } else {
                encoded.push(count);
                encoded.push(current_byte);
                current_byte = b;
                count = 1;
            }
        }
        encoded.push(count);
        encoded.push(current_byte);
    }

    file.write_all(&(encoded.len() as u64).to_le_bytes())?;
    file.write_all(&encoded)?;
    Ok(())
}
}