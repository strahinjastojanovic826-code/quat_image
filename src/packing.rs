//! # Safety Notes on `unsafe` SIMD Blocks in `packing.rs`
//!
//! This module dispatches vectorized packing and unpacking routines (`AVX2`, `SSSE3`,
//! and `NEON`) with runtime CPU feature detection and falls back to safe slice loops
//! when hardware extensions are unavailable or for trailing elements.
//!
//! ## Safety Invariants Assumed Across `unsafe` Blocks
//! - **Runtime feature availability:** `pack_quats_avx2` and `unpack_quats_avx2` are
//!   only invoked when `is_x86_feature_detected!("avx2")` returns `true`;
//!   `pack_quats_ssse3` only when `is_x86_feature_detected!("ssse3")` returns
//!   `true`; and `pack_quats_neon` only when `std::arch::is_aarch64_feature_detected!("neon")`
//!   returns `true` on `aarch64`. Each helper function is also annotated with the
//!   corresponding `#[target_feature(enable = "...")]` attribute.
//! - **Slice lengths & chunk alignment:** Vectorized loops iterate over `chunks_exact`
//!   iterators of fixed lane widths (`32` quats / `8` packed bytes for `AVX2`, `16`
//!   quats / `4` packed bytes for `SSSE3` and `NEON`, and `8` packed bytes / `32`
//!   quats for `unpack_quats_avx2`), while any remainder is handled safely via
//!   `pack_quats_slice` (or safe unpacking). Raw pointer loads (`_mm256_loadu_si256`,
//!   `_mm_loadu_si128`, `vld1q_u8`, `vld4_u8`) operate over valid `chunks_exact`
//!   slices of exact byte length without requiring prior alignment.
//! - **Input validation:** Before packing a SIMD chunk, input values are checked
//!   against the `0..=3` quaternary range using bitmasks (`0xFC` / `0x0FC00FC00FC00FC0`),
//!   returning `Err(QuatImageError::InvalidQuatValue(...))` if any element exceeds `3`.

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

pub fn pack_quats_slice(quats: &[u8], packed: &mut [u8]) -> Result<(), QuatImageError> {
    if quats.len() % 4 != 0 || packed.len() != quats.len() / 4 {
        return Err(QuatImageError::InvalidDimensions);
    }
    
    for (q_chunk, p_byte) in quats.chunks_exact(4).zip(packed.iter_mut()) {
        if q_chunk[0] > 3 || q_chunk[1] > 3 || q_chunk[2] > 3 || q_chunk[3] > 3 {
            let invalid = *q_chunk.iter().find(|&&x| x > 3).unwrap();
            return Err(QuatImageError::InvalidQuatValue(invalid));
        }
        *p_byte = (q_chunk[3] << 6) | (q_chunk[2] << 4) | (q_chunk[1] << 2) | q_chunk[0];
    }
    Ok(())
}


/// Packs a slice of quaternary values (each in `0..=3`) into 2-bit packed bytes
/// using SIMD acceleration (AVX2 / SSSE3 on x86_64, NEON on aarch64) with scalar fallback.
pub fn pack_quats_simd(quats: &[u8], packed: &mut [u8]) -> Result<(), QuatImageError> {
    if quats.len() % 4 != 0 || packed.len() != quats.len() / 4 {
        return Err(QuatImageError::InvalidDimensions);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { pack_quats_avx2(quats, packed) };
        }
        if is_x86_feature_detected!("ssse3") {
            return unsafe { pack_quats_ssse3(quats, packed) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return unsafe { pack_quats_neon(quats, packed) };
        }
    }

    pack_quats_slice(quats, packed)
}

/// Unpacks 2-bit packed bytes into a slice of quaternary values (`0..=3`) using SIMD.
pub fn unpack_quats_simd(packed: &[u8], quats: &mut [u8]) -> Result<(), QuatImageError> {
    if quats.len() != packed.len() * 4 {
        return Err(QuatImageError::InvalidDimensions);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { unpack_quats_avx2(packed, quats) };
            return Ok(());
        }
    }

    for (p_byte, q_chunk) in packed.iter().zip(quats.chunks_exact_mut(4)) {
        let (q0, q1, q2, q3) = unpack_quats(*p_byte);
        q_chunk[0] = q0;
        q_chunk[1] = q1;
        q_chunk[2] = q2;
        q_chunk[3] = q3;
    }
    Ok(())
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn pack_quats_avx2(quats: &[u8], packed: &mut [u8]) -> Result<(), QuatImageError> { unsafe {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut q_chunks = quats.chunks_exact(32);
    let mut p_chunks = packed.chunks_exact_mut(8);

    let invalid_mask = _mm256_set1_epi8(0xFCu8 as i8);
    // Multiply adjacent pairs: (q0 * 1 + q1 * 4) -> lower nibble has [q1:q0]
    let mul_pairs = _mm256_set1_epi16(0x0401);
    // Multiply adjacent 16-bit words: (pair0 * 1 + pair1 * 16) -> lower byte has [q3:q2:q1:q0]
    let mul_quads = _mm256_set1_epi32(0x0010_0001);
    let pack_shuffle = _mm256_setr_epi8(
        0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );

    for (q_in, p_out) in (&mut q_chunks).zip(&mut p_chunks) {
        let v = _mm256_loadu_si256(q_in.as_ptr() as *const __m256i);

        // Validate all 32 bytes are in 0..=3
        let bad = _mm256_and_si256(v, invalid_mask);
        if _mm256_testz_si256(bad, bad) == 0 {
            let &invalid = q_in.iter().find(|&&x| x > 3).unwrap();
            return Err(QuatImageError::InvalidQuatValue(invalid));
        }

        let pairs = _mm256_maddubs_epi16(v, mul_pairs);
        let quads = _mm256_madd_epi16(pairs, mul_quads);
        let shuffled = _mm256_shuffle_epi8(quads, pack_shuffle);

        let lo = _mm256_extract_epi32(shuffled, 0) as u32;
        let hi = _mm256_extract_epi32(shuffled, 4) as u32;
        p_out[0..4].copy_from_slice(&lo.to_le_bytes());
        p_out[4..8].copy_from_slice(&hi.to_le_bytes());
    }

    // Tail (< 32 elements)
    let rem_q = q_chunks.remainder();
    let rem_p = p_chunks.into_remainder();
    if !rem_q.is_empty() {
        pack_quats_slice(rem_q, rem_p)?;
    }

    Ok(())
}}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "ssse3")]
unsafe fn pack_quats_ssse3(quats: &[u8], packed: &mut [u8]) -> Result<(), QuatImageError> { unsafe {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut q_chunks = quats.chunks_exact(16);
    let mut p_chunks = packed.chunks_exact_mut(4);

    let invalid_mask = _mm_set1_epi8(0xFCu8 as i8);
    let mul_pairs = _mm_set1_epi16(0x0401);
    let mul_quads = _mm_set1_epi32(0x0010_0001);
    let pack_shuffle = _mm_setr_epi8(
        0, 4, 8, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    );

    for (q_in, p_out) in (&mut q_chunks).zip(&mut p_chunks) {
        let v = _mm_loadu_si128(q_in.as_ptr() as *const __m128i);
        let bad = _mm_and_si128(v, invalid_mask);
        if _mm_movemask_epi8(_mm_cmpeq_epi8(bad, _mm_setzero_si128())) != 0xFFFF {
            let &invalid = q_in.iter().find(|&&x| x > 3).unwrap();
            return Err(QuatImageError::InvalidQuatValue(invalid));
        }

        let pairs = _mm_maddubs_epi16(v, mul_pairs);
        let quads = _mm_madd_epi16(pairs, mul_quads);
        let shuffled = _mm_shuffle_epi8(quads, pack_shuffle);
        let packed_u32 = _mm_cvtsi128_si32(shuffled) as u32;
        p_out.copy_from_slice(&packed_u32.to_le_bytes());
    }

    let rem_q = q_chunks.remainder();
    let rem_p = p_chunks.into_remainder();
    if !rem_q.is_empty() {
        pack_quats_slice(rem_q, rem_p)?;
    }

    Ok(())
}}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn pack_quats_neon(quats: &[u8], packed: &mut [u8]) -> Result<(), QuatImageError> {
    use std::arch::aarch64::*;

    let mut q_chunks = quats.chunks_exact(16);
    let mut p_chunks = packed.chunks_exact_mut(4);
    let mask_hi = vdupq_n_u8(0xFC);

    for (q_in, p_out) in (&mut q_chunks).zip(&mut p_chunks) {
        let v = vld1q_u8(q_in.as_ptr());
        let bad = vandq_u8(v, mask_hi);
        if vmaxvq_u8(bad) != 0 {
            let &invalid = q_in.iter().find(|&&x| x > 3).unwrap();
            return Err(QuatImageError::InvalidQuatValue(invalid));
        }

        // Deinterleave into 4 lanes: q0, q1, q2, q3 (4 bytes each)
        let ld = vld4_u8(q_in.as_ptr());
        let p01 = vorr_u8(ld.0, vshl_n_u8(ld.1, 2));
        let p23 = vorr_u8(vshl_n_u8(ld.2, 4), vshl_n_u8(ld.3, 6));
        let res = vorr_u8(p01, p23);
        let val = vget_lane_u32(vreinterpret_u32_u8(res), 0);
        p_out.copy_from_slice(&val.to_le_bytes());
    }

    let rem_q = q_chunks.remainder();
    let rem_p = p_chunks.into_remainder();
    if !rem_q.is_empty() {
        pack_quats_slice(rem_q, rem_p)?;
    }

    Ok(())
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn unpack_quats_avx2(packed: &[u8], quats: &mut [u8]) { unsafe {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut p_chunks = packed.chunks_exact(8);
    let mut q_chunks = quats.chunks_exact_mut(32);

    // Replicate each input byte 4 times across 32 bytes
    let dup_shuffle = _mm256_setr_epi8(
        0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
        4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7,
    );
    // Bit masks per lane position: q0=0x03, q1=0x0C, q2=0x30, q3=0xC0
    let bit_masks = _mm256_set1_epi32(0xC030_0C03u32 as i32);
    // Normalize shifted bits down to 0..=3 via high-byte multiply on 16-bit pairs:
    // lane0 * 64, lane1 * 16 -> >> 6; lane2 * 4, lane3 * 1 -> >> 6
    let norm_mul = _mm256_set1_epi32(0x0104_1040u32 as i32);

    for (p_in, q_out) in (&mut p_chunks).zip(&mut q_chunks) {
        let raw = u64::from_le_bytes(p_in.try_into().unwrap()) as i64;
        let v = _mm256_set1_epi64x(raw);
        let rep = _mm256_shuffle_epi8(v, dup_shuffle);
        let isolated = _mm256_and_si256(rep, bit_masks);

        // Multiply each u8 by factor and shift right by 6 bits (per 16-bit lane)
        let lo_u16 = _mm256_and_si256(isolated, _mm256_set1_epi16(0x00FF));
        let hi_u16 = _mm256_srli_epi16(isolated, 8);
        let lo_mul = _mm256_srli_epi16(_mm256_mullo_epi16(lo_u16, _mm256_and_si256(norm_mul, _mm256_set1_epi16(0x00FF))), 6);
        let hi_mul = _mm256_slli_epi16(_mm256_srli_epi16(_mm256_mullo_epi16(hi_u16, _mm256_srli_epi16(norm_mul, 8)), 6), 8);

        let unpacked = _mm256_or_si256(lo_mul, hi_mul);
        _mm256_storeu_si256(q_out.as_mut_ptr() as *mut __m256i, unpacked);
    }

    for (p_byte, q_chunk) in p_chunks.remainder().iter().zip(q_chunks.into_remainder().chunks_exact_mut(4)) {
        let (q0, q1, q2, q3) = unpack_quats(*p_byte);
        q_chunk[0] = q0;
        q_chunk[1] = q1;
        q_chunk[2] = q2;
        q_chunk[3] = q3;
    }
}}