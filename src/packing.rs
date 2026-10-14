use crate::error::QuatImageError;

#[inline]
pub fn pack_quats(q0: u8, q1: u8, q2: u8, q3: u8) -> Result<u8, QuatImageError> {
    if q0 > 3 { return Err(QuatImageError::InvalidQuatValue(q0)); }
    if q1 > 3 { return Err(QuatImageError::InvalidQuatValue(q1)); }
    if q2 > 3 { return Err(QuatImageError::InvalidQuatValue(q2)); }
    if q3 > 3 { return Err(QuatImageError::InvalidQuatValue(q3)); }
    
    Ok((q3 << 6) | (q2 << 4) | (q1 << 2) | (q0 & 0x03))
}

#[inline]
pub fn unpack_quats(byte: u8) -> (u8, u8, u8, u8) {
    let q0 = byte & 0x03;
    let (b1, _) = byte.overflowing_shr(2);
    let q1 = b1 & 0x03;
    let (b2, _) = byte.overflowing_shr(4);
    let q2 = b2 & 0x03;
    let (b3, _) = byte.overflowing_shr(6);
    let q3 = b3 & 0x03;
    (q0, q1, q2, q3)
}