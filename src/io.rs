use crate::error::QuatImageError;
use crate::image::QuatImage;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const MAGIC_HEADER: &[u8; 4] = b"QIMG"; // Magic identifier for binary file
const MAGIC_V1: &[u8; 4] = b"QIMG";
const FORMAT_VERSION_V1: u8 = 1;
const HEADER_SIZE_V1: usize = 36;

/// IEEE 802.3 CRC-32 without external dependencies.
fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn build_v1_frame(img: &QuatImage) -> Result<Vec<u8>, QuatImageError> {
    let payload = img.as_bytes();
    let width_u64 = u64::try_from(img.width()).map_err(|_| QuatImageError::IntegerOverflow)?;
    let height_u64 = u64::try_from(img.height()).map_err(|_| QuatImageError::IntegerOverflow)?;
    let payload_len_u64 = u64::try_from(payload.len()).map_err(|_| QuatImageError::IntegerOverflow)?;

    let total_len = HEADER_SIZE_V1
        .checked_add(payload.len())
        .ok_or(QuatImageError::IntegerOverflow)?;
    let mut buf = Vec::with_capacity(total_len);

    // 0..4: Magic
    buf.extend_from_slice(MAGIC_V1);
    // 4: Version, 5: Flags, 6..8: Reserved
    buf.push(FORMAT_VERSION_V1);
    buf.push(0u8);
    buf.extend_from_slice(&[0u8; 2]);
    // 8..16: Width, 16..24: Height, 24..32: Payload length
    buf.extend_from_slice(&width_u64.to_le_bytes());
    buf.extend_from_slice(&height_u64.to_le_bytes());
    buf.extend_from_slice(&payload_len_u64.to_le_bytes());

    // Compute checksum over header (0..32) + payload
    let mut crc = crc32_ieee(&buf[0..32]);
    let mut crc_full = !crc;
    for &byte in payload {
        crc_full ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc_full & 1).wrapping_neg();
            crc_full = (crc_full >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    crc = !crc_full;

    // 32..36: CRC32
    buf.extend_from_slice(&crc.to_le_bytes());
    // 36..end: Payload
    buf.extend_from_slice(payload);

    Ok(buf)
}

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

/// Loads a `QuatImage` saved with `save_to_file_rle`.
    pub fn load_from_file_rle<P: AsRef<Path>>(path: P) -> Result<Self, QuatImageError> {
        let mut file = File::open(path).map_err(|_| QuatImageError::OutOfBounds)?;

        let mut header = [0u8; 4];
        file.read_exact(&mut header)
            .map_err(|_| QuatImageError::OutOfBounds)?;
        if &header != b"QRLE" {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut width_bytes = [0u8; 8];
        let mut height_bytes = [0u8; 8];
        let mut encoded_len_bytes = [0u8; 8];
        file.read_exact(&mut width_bytes)
            .map_err(|_| QuatImageError::OutOfBounds)?;
        file.read_exact(&mut height_bytes)
            .map_err(|_| QuatImageError::OutOfBounds)?;
        file.read_exact(&mut encoded_len_bytes)
            .map_err(|_| QuatImageError::OutOfBounds)?;

        let width = u64::from_le_bytes(width_bytes) as usize;
        let height = u64::from_le_bytes(height_bytes) as usize;
        let encoded_len = u64::from_le_bytes(encoded_len_bytes) as usize;

        if encoded_len % 2 != 0 {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut img = Self::new(width, height)?;
        let expected_bytes = img.as_bytes().len();

        let mut encoded = vec![0u8; encoded_len];
        file.read_exact(&mut encoded)
            .map_err(|_| QuatImageError::OutOfBounds)?;

        let mut decoded = Vec::with_capacity(expected_bytes);
        for pair in encoded.chunks_exact(2) {
            let count = pair[0] as usize;
            let val = pair[1];
            if count == 0 || decoded.len() + count > expected_bytes {
                return Err(QuatImageError::InvalidDimensions);
            }
            decoded.extend(std::iter::repeat_n(val, count));
        }

        if decoded.len() != expected_bytes {
            return Err(QuatImageError::InvalidDimensions);
        }

        img.as_mut_bytes().copy_from_slice(&decoded);
        Ok(img)
    }

    /// Saves the `QuatImage` in binary format v1 (magic + version + dimensions + CRC32 + payload).
    pub fn save_to_file_v1<P: AsRef<Path>>(&self, path: P) -> Result<(), QuatImageError> {
        let frame = build_v1_frame(self)?;
        let mut file = File::create(path).map_err(|_| QuatImageError::OutOfBounds)?;
        file.write_all(&frame)
            .map_err(|_| QuatImageError::OutOfBounds)?;
        Ok(())
    }

    /// Loads and validates a `QuatImage` from binary format v1, verifying magic, version, and CRC32.
    pub fn load_from_file_v1<P: AsRef<Path>>(path: P) -> Result<Self, QuatImageError> {
        let mut file = File::open(path).map_err(|_| QuatImageError::OutOfBounds)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|_| QuatImageError::OutOfBounds)?;

        if bytes.len() < HEADER_SIZE_V1 {
            return Err(QuatImageError::InvalidDimensions);
        }

        if &bytes[0..4] != MAGIC_V1 {
            return Err(QuatImageError::InvalidDimensions);
        }

        let version = bytes[4];
        if version != FORMAT_VERSION_V1 {
            return Err(QuatImageError::InvalidDimensions);
        }

        let width = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let height = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
        let payload_len = u64::from_le_bytes(bytes[24..32].try_into().unwrap()) as usize;
        let expected_crc = u32::from_le_bytes(bytes[32..36].try_into().unwrap());

        let expected_total = HEADER_SIZE_V1
            .checked_add(payload_len)
            .ok_or(QuatImageError::IntegerOverflow)?;
        if bytes.len() != expected_total {
            return Err(QuatImageError::InvalidDimensions);
        }

        let img = Self::new(width, height)?;
        if img.as_bytes().len() != payload_len {
            return Err(QuatImageError::InvalidDimensions);
        }

        // Verify CRC32 across header (0..32) and payload (36..end)
        let mut state = 0xFFFF_FFFFu32;
        for &byte in &bytes[0..32] {
            state ^= byte as u32;
            for _ in 0..8 {
                let mask = (state & 1).wrapping_neg();
                state = (state >> 1) ^ (0xEDB8_8320 & mask);
            }
        }
        for &byte in &bytes[36..] {
            state ^= byte as u32;
            for _ in 0..8 {
                let mask = (state & 1).wrapping_neg();
                state = (state >> 1) ^ (0xEDB8_8320 & mask);
            }
        }
        let actual_crc = !state;

        if actual_crc != expected_crc {
            return Err(QuatImageError::InvalidDimensions);
        }

        let mut out = img;
        out.as_mut_bytes().copy_from_slice(&bytes[36..]);
        Ok(out)
    }

}