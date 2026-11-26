pub mod packing;
pub mod image;
pub mod error;
pub mod multi_image;
pub mod view;

#[cfg(feature = "io")]
pub mod io;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "parallel")]
pub mod ops;

pub use packing::*;
pub use image::*;
pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::QuatImageError;
    use crate::packing::{pack_quats, unpack_quats};
    use crate::image::QuatImage;
    use std::time::Instant;
    use crate::view::QuatImageView;

    // ==========================================
    // 1. Osnovni testovi (Najlakši - Happy Path)
    // ==========================================

    // Test 1: Image creation and dimension verification
    #[test]
    fn test_01_basic_creation() {
        let start = Instant::now();
        let img = QuatImage::new(10, 20).unwrap();
        println!("[BENCHMARK] test_01_basic_creation: {:?}", start.elapsed());

        assert_eq!(img.width(), 10);
        assert_eq!(img.height(), 20);
    }

    // Test 2: Writing and reading valid pixels (0-3)
    #[test]
    fn test_02_set_and_get_pixel() {
        let mut img = QuatImage::new(4, 4).unwrap();

        let start = Instant::now();
        img.set_pixel(1, 2, 3).unwrap();
        let val1 = img.get_pixel(1, 2).unwrap();
        img.set_pixel(0, 0, 0).unwrap();
        let val2 = img.get_pixel(0, 0).unwrap();
        println!("[BENCHMARK] test_02_set_and_get_pixel: {:?}", start.elapsed());

        assert_eq!(val1, 3);
        assert_eq!(val2, 0);
    }

    // Test 3: Resetting/clearing the entire image
    #[test]
    fn test_03_clear_image() {
        let mut img = QuatImage::new(2, 2).unwrap();
        img.set_pixel(0, 0, 2).unwrap();
        img.set_pixel(1, 1, 3).unwrap();

        let start = Instant::now();
        img.clear();
        println!("[BENCHMARK] test_03_clear_image: {:?}", start.elapsed());

        assert_eq!(img.get_pixel(0, 0).unwrap(), 0);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 0);
    }

    // Test 4: Direct packing and unpacking of quats (unit test for packing.rs)
    #[test]
    fn test_04_pack_unpack_unit() {
        let start = Instant::now();
        let packed = pack_quats(1, 2, 3, 0).unwrap();
        let (q0, q1, q2, q3) = unpack_quats(packed);
        println!("[BENCHMARK] test_04_pack_unpack_unit: {:?}", start.elapsed());

        assert_eq!(q0, 1);
        assert_eq!(q1, 2);
        assert_eq!(q2, 3);
        assert_eq!(q3, 0);
    }

    // ==========================================
    // 2. Error and Edge Case Tests
    // ==========================================

    // Test 5: Error when writing an invalid pixel value (> 3)
    #[test]
    fn test_05_invalid_pixel_value() {
        let mut img = QuatImage::new(4, 4).unwrap();

        let start = Instant::now();
        let res = img.set_pixel(0, 0, 4); // Only 0, 1, 2, 3 allowed
        let pack_res = pack_quats(5, 0, 0, 0);
        println!("[BENCHMARK] test_05_invalid_pixel_value: {:?}", start.elapsed());

        assert!(matches!(res, Err(QuatImageError::InvalidQuatValue(4))));
        assert!(matches!(pack_res, Err(QuatImageError::InvalidQuatValue(5))));
    }

    // Test 6: Out of bounds error (OutOfBounds)
    #[test]
    fn test_06_out_of_bounds() {
        let mut img = QuatImage::new(5, 5).unwrap();

        let start = Instant::now();
        let r1 = img.get_pixel(5, 0);
        let r2 = img.get_pixel(0, 5);
        let r3 = img.set_pixel(10, 10, 1);
        println!("[BENCHMARK] test_06_out_of_bounds: {:?}", start.elapsed());

        assert!(matches!(r1, Err(QuatImageError::OutOfBounds)));
        assert!(matches!(r2, Err(QuatImageError::OutOfBounds)));
        assert!(matches!(r3, Err(QuatImageError::OutOfBounds)));
    }

    // Test 7: Invalid image dimensions (zero width or height)
    #[test]
    fn test_07_invalid_dimensions() {
        let start = Instant::now();
        let r1 = QuatImage::new(0, 10);
        let r2 = QuatImage::new(10, 0);
        let r3 = QuatImage::new(0, 0);
        println!("[BENCHMARK] test_07_invalid_dimensions: {:?}", start.elapsed());

        assert!(matches!(r1, Err(QuatImageError::InvalidDimensions)));
        assert!(matches!(r2, Err(QuatImageError::InvalidDimensions)));
        assert!(matches!(r3, Err(QuatImageError::InvalidDimensions)));
    }

    // ==========================================
    // 3. Complex and Stress Tests
    // ==========================================

    // Test 8: Image transformations (Horizontal and Vertical flip)
    #[test]
    fn test_08_transformations_flip() {
        let mut img = QuatImage::new(2, 2).unwrap();
        // Row 0: [1, 2]
        // Row 1: [0, 3]
        img.set_pixel(0, 0, 1).unwrap();
        img.set_pixel(1, 0, 2).unwrap();
        img.set_pixel(0, 1, 0).unwrap();
        img.set_pixel(1, 1, 3).unwrap();

        // Horizontal flip -> Row 0 becomes [2, 1], Row 1 becomes [3, 0]
        let start_h = Instant::now();
        img.flip_horizontal().unwrap();
        println!("[BENCHMARK] test_08_flip_horizontal: {:?}", start_h.elapsed());

        assert_eq!(img.get_pixel(0, 0).unwrap(), 2);
        assert_eq!(img.get_pixel(1, 0).unwrap(), 1);
        assert_eq!(img.get_pixel(0, 1).unwrap(), 3);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 0);

        // Vertical flip
        let start_v = Instant::now();
        img.flip_vertical().unwrap();
        println!("[BENCHMARK] test_08_flip_vertical: {:?}", start_v.elapsed());

        assert_eq!(img.get_pixel(0, 0).unwrap(), 3);
        assert_eq!(img.get_pixel(1, 0).unwrap(), 0);
    }

    // Test 9: Memory overflow protection (Integer Overflow)
    #[test]
    fn test_09_integer_overflow_protection() {
        let start = Instant::now();
        let res = QuatImage::new(usize::MAX, usize::MAX);
        println!("[BENCHMARK] test_09_integer_overflow_protection: {:?}", start.elapsed());

        assert!(matches!(res, Err(QuatImageError::IntegerOverflow)));
    }

    // Test 10: Heavy endurance stress test (1000x1000 grid, massive read/write, and transformations)
    #[test]
    fn test_10_stress_endurance_massive_grid() {
        const WIDTH: usize = 1000;
        const HEIGHT: usize = 1000;
        let mut img = QuatImage::new(WIDTH, HEIGHT).unwrap();

        // Massive writing over 1,000,000 pixels with calculated values (0-3)
        let start_write = Instant::now();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let val = ((x + y) % 4) as u8;
                img.set_pixel(x, y, val).unwrap();
            }
        }
        println!("[BENCHMARK] test_10_write_1M_pixels: {:?}", start_write.elapsed());

        // Data integrity check for every single pixel after byte packing
        let start_read = Instant::now();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let expected = ((x + y) % 4) as u8;
                let actual = img.get_pixel(x, y).unwrap();
                assert_eq!(actual, expected, "Data mismatch at coordinate ({}, {})", x, y);
            }
        }
        println!("[BENCHMARK] test_10_read_1M_pixels: {:?}", start_read.elapsed());

        // Stress testing flips on massive array
        let start_flips = Instant::now();
        img.flip_horizontal().unwrap();
        img.flip_vertical().unwrap();
        println!("[BENCHMARK] test_10_flips_1M_pixels: {:?}", start_flips.elapsed());

        // Corner accuracy check after multiple transformations
        let corner_val = img.get_pixel(0, 0).unwrap();
        let expected_corner = (((WIDTH - 1) + (HEIGHT - 1)) % 4) as u8;
        assert_eq!(corner_val, expected_corner);
    }

    // Test 11: 90-degree clockwise rotation
    #[test]
    fn test_rotate_90_cw() {
        let mut img = QuatImage::new(2, 1).unwrap();
        img.set_pixel(0, 0, 1).unwrap();
        img.set_pixel(1, 0, 2).unwrap();

        let start = Instant::now();
        img.rotate_90_cw().unwrap();
        println!("[BENCHMARK] test_rotate_90_cw: {:?}", start.elapsed());

        assert_eq!(img.width(), 1);
        assert_eq!(img.height(), 2);
        assert_eq!(img.get_pixel(0, 0).unwrap(), 2);
        assert_eq!(img.get_pixel(0, 1).unwrap(), 1);
    }

    // Test 12: Calculating histogram
    #[test]
    fn test_histogram() {
        let mut img = QuatImage::new(2, 2).unwrap();
        img.set_pixel(0, 0, 1).unwrap();
        img.set_pixel(1, 0, 1).unwrap();
        img.set_pixel(0, 1, 3).unwrap();
        // (1, 1) remains 0 by default

        let start = Instant::now();
        let hist = img.histogram();
        println!("[BENCHMARK] test_histogram: {:?}", start.elapsed());

        assert_eq!(hist[0], 1); // 1 pixel with value 0
        assert_eq!(hist[1], 2); // 2 pixels with value 1
        assert_eq!(hist[2], 0); // 0 pixels with value 2
        assert_eq!(hist[3], 1); // 1 pixel with value 3
    }

    // Test 13: Batch slice packing
    #[test]
    fn test_pack_quats_slice_batch() {
        let quats = vec![0, 1, 2, 3, 3, 2, 1, 0];
        let mut packed = vec![0u8; 2];

        // Test successful batch packing
        let start = Instant::now();
        let res = pack_quats_slice(&quats, &mut packed);
        println!("[BENCHMARK] test_pack_quats_slice_batch: {:?}", start.elapsed());

        assert!(res.is_ok());

        // Verify bit-packing correctness
        assert_eq!(packed[0], (3 << 6) | (2 << 4) | (1 << 2) | 0);
        assert_eq!(packed[1], (0 << 6) | (1 << 4) | (2 << 2) | 3);

        // Test error on invalid slice length (not divisible by 4)
        let invalid_quats = vec![0, 1, 2];
        let mut invalid_packed = vec![0u8; 1];
        assert!(matches!(
            pack_quats_slice(&invalid_quats, &mut invalid_packed),
            Err(QuatImageError::InvalidDimensions)
        ));
    }

    // Test 14: Morphological dilation
    #[test]
    fn test_morphological_dilation() {
        let mut img = QuatImage::new(3, 3).unwrap();
        // Set a single pixel with maximum value 3 in the center
        img.set_pixel(1, 1, 3).unwrap();

        // Dilation (max filter in 3x3 neighborhood) should expand value 3 to all surrounding pixels
        let start = Instant::now();
        let dilated = img.dilate().unwrap();
        println!("[BENCHMARK] test_morphological_dilation: {:?}", start.elapsed());

        assert_eq!(dilated.get_pixel(0, 0).unwrap(), 3);
        assert_eq!(dilated.get_pixel(1, 0).unwrap(), 3);
        assert_eq!(dilated.get_pixel(2, 2).unwrap(), 3);
    }

    // Test 15: Parallel color replacement using Rayon
    #[test]
    fn test_replace_color_parallel() {
        let mut img = QuatImage::new(3, 3).unwrap();
        img.set_pixel(0, 0, 1).unwrap();
        img.set_pixel(2, 2, 1).unwrap();

        // Parallel replacement of value 1 with value 2 via Rayon
        let start = Instant::now();
        img.replace_color_par(1, 2).unwrap();
        println!("[BENCHMARK] test_replace_color_parallel: {:?}", start.elapsed());

        assert_eq!(img.get_pixel(0, 0).unwrap(), 2);
        assert_eq!(img.get_pixel(2, 2).unwrap(), 2);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 0); // Remained default
    }

    // Test 16: RGB8 quantization conversion
    #[test]
    fn test_rgb8_quantization_conversion() {
        // Simulate 4 pixels of a standard RGB array (dark, mid-low, mid-high, bright)
        let rgb_data = vec![
            10, 10, 10,     // Should map to quat 0
            80, 80, 80,     // Should map to quat 1
            150, 150, 150,  // Should map to quat 2
            240, 240, 240,  // Should map to quat 3
        ];

        let start = Instant::now();
        let img = QuatImage::from_rgb8(2, 2, &rgb_data).unwrap();
        println!("[BENCHMARK] test_rgb8_quantization_conversion: {:?}", start.elapsed());

        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.get_pixel(0, 0).unwrap(), 0);
        assert_eq!(img.get_pixel(1, 0).unwrap(), 1);
        assert_eq!(img.get_pixel(0, 1).unwrap(), 2);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 3);
    }

    #[test]
fn test_morphological_erosion_benchmark() {
    let mut img = QuatImage::new(10, 10).unwrap();
    // Ako postavimo sve na 1, a samo centar na 0:
    for y in 0..10 {
        for x in 0..10 {
            img.set_pixel(x, y, 1).unwrap();
        }
    }
    img.set_pixel(5, 5, 0).unwrap();

    let start = std::time::Instant::now();
    let eroded = img.erode().unwrap();
    let duration = start.elapsed();
    
    println!("[BENCHMARK] Erosion (10x10): {:?}", duration);

    // Centar i njegova okolina bi trebalo da osete uticaj nule
    assert_eq!(eroded.get_pixel(5, 5).unwrap(), 0);
    // Ugao je daleko od nule, pa ostaje 1
    assert_eq!(eroded.get_pixel(0, 0).unwrap(), 1);
}

// Test 13: Morphological opening and closing with performance benchmark
#[test]
fn test_opening_and_closing_benchmark() {
    let mut img = QuatImage::new(100, 100).unwrap();
    img.set_pixel(50, 50, 2).unwrap();

    let start_open = std::time::Instant::now();
    let opened = img.opening().unwrap();
    let duration_open = start_open.elapsed();
    println!("[BENCHMARK] Opening (100x100): {:?}", duration_open);

    let start_close = std::time::Instant::now();
    let closed = img.closing().unwrap();
    let duration_close = start_close.elapsed();
    println!("[BENCHMARK] Closing (100x100): {:?}", duration_close);

    assert_eq!(opened.width(), 100);
    assert_eq!(closed.height(), 100);
}

// Test 14: Image cropping with performance benchmark
#[test]
fn test_image_crop_benchmark() {
    let mut img = QuatImage::new(200, 200).unwrap();
    img.set_pixel(100, 100, 3).unwrap();

    let start = std::time::Instant::now();
    let cropped = img.crop(50, 50, 100, 100).unwrap();
    let duration = start.elapsed();
    
    println!("[BENCHMARK] Crop (50..150): {:?}", duration);

    assert_eq!(cropped.width(), 100);
    assert_eq!(cropped.height(), 100);
    assert_eq!(cropped.get_pixel(50, 50).unwrap(), 3);
}

#[test]
fn test_15_quaternary_bitwise_and_invert() {
    let mut a = QuatImage::new(4, 2).unwrap();
    let mut b = QuatImage::new(4, 2).unwrap();
    a.set_pixel(0, 0, 1).unwrap(); // 01
    a.set_pixel(1, 0, 2).unwrap(); // 10
    a.set_pixel(2, 0, 3).unwrap(); // 11
    b.set_pixel(0, 0, 3).unwrap(); // 11
    b.set_pixel(1, 0, 1).unwrap(); // 01
    b.set_pixel(2, 0, 2).unwrap(); // 10

    let start = Instant::now();
    let and_img = a.bitwise_and(&b).unwrap();
    let or_img = a.bitwise_or(&b).unwrap();
    let xor_img = a.bitwise_xor(&b).unwrap();
    a.invert();
    println!(
        "[BENCHMARK] test_15_quaternary_bitwise_and_invert: {:?}",
        start.elapsed()
    );

    assert_eq!(and_img.get_pixel(0, 0).unwrap(), 1);
    assert_eq!(and_img.get_pixel(1, 0).unwrap(), 0);
    assert_eq!(and_img.get_pixel(2, 0).unwrap(), 2);
    assert_eq!(or_img.get_pixel(1, 0).unwrap(), 3);
    assert_eq!(xor_img.get_pixel(2, 0).unwrap(), 1);
    assert_eq!(a.get_pixel(0, 0).unwrap(), 2); // 3 - 1 = 2
}

#[test]
fn test_16_remap_and_threshold() {
    let mut img = QuatImage::new(4, 1).unwrap();
    img.set_pixel(0, 0, 0).unwrap();
    img.set_pixel(1, 0, 1).unwrap();
    img.set_pixel(2, 0, 2).unwrap();
    img.set_pixel(3, 0, 3).unwrap();

    let start = Instant::now();
    img.remap([3, 1, 2, 0]).unwrap();
    img.threshold(2, 0, 3).unwrap();
    println!(
        "[BENCHMARK] test_16_remap_and_threshold: {:?}",
        start.elapsed()
    );

    // After remap([3,1,2,0]): pixels become [3, 1, 2, 0]
    // After threshold(2, 0, 3): values >= 2 -> 3, values < 2 -> 0 => [3, 0, 3, 0]
    assert_eq!(img.get_pixel(0, 0).unwrap(), 3);
    assert_eq!(img.get_pixel(1, 0).unwrap(), 0);
    assert_eq!(img.get_pixel(2, 0).unwrap(), 3);
    assert_eq!(img.get_pixel(3, 0).unwrap(), 0);
}

#[test]
fn test_17_blit_flood_fill_and_connected_components() {
    let mut canvas = QuatImage::new(6, 6).unwrap();
    let mut patch = QuatImage::new(2, 2).unwrap();
    patch.set_pixel(0, 0, 2).unwrap();
    patch.set_pixel(1, 0, 2).unwrap();
    patch.set_pixel(0, 1, 2).unwrap();
    patch.set_pixel(1, 1, 2).unwrap();

    let start = Instant::now();
    canvas.blit(&patch, 1, 1).unwrap();
    let filled = canvas.flood_fill(1, 1, 3).unwrap();
    let (labels, count) = canvas.connected_components();
    println!(
        "[BENCHMARK] test_17_blit_flood_fill_and_connected_components: {:?}",
        start.elapsed()
    );

    assert_eq!(filled, 4);
    assert_eq!(canvas.get_pixel(2, 2).unwrap(), 3);
    assert_eq!(count, 1);
    assert_eq!(labels.len(), 36);
}

#[test]
fn test_18_grayscale_rgb_and_metrics() {
    let gray = vec![0u8, 85, 170, 255];

    let start = Instant::now();
    let mut img = QuatImage::from_grayscale8(2, 2, &gray).unwrap();
    let expanded_gray = img.to_grayscale8();
    let rgb = img.to_rgb8([[0, 0, 0], [80, 80, 80], [160, 160, 160], [255, 255, 255]]);
    let diffs = img.diff_count(&img).unwrap();
    let psnr_val = img.psnr(&img).unwrap();
    let iou_val = img.iou(&img, 3).unwrap();
    img.set_pixel(0, 0, 1).unwrap();
    let diffs_after = img.diff_count(&QuatImage::from_grayscale8(2, 2, &gray).unwrap()).unwrap();
    println!(
        "[BENCHMARK] test_18_grayscale_rgb_and_metrics: {:?}",
        start.elapsed()
    );

    assert_eq!(expanded_gray, vec![0, 85, 170, 255]);
    assert_eq!(rgb.len(), 12);
    assert_eq!(diffs, 0);
    assert!(psnr_val.is_infinite());
    assert!((iou_val - 1.0).abs() < f64::EPSILON);
    assert_eq!(diffs_after, 1);
}

#[test]
fn test_19_zero_copy_view_and_subview() {
    let mut img = QuatImage::new(8, 8).unwrap();
    img.set_pixel(3, 3, 2).unwrap();
    img.set_pixel(4, 4, 3).unwrap();

    let start = Instant::now();
    let view = img.view(2, 2, 4, 4).unwrap();
    let sub: QuatImageView<'_> = view.subview(1, 1, 2, 2).unwrap();
    let hist = sub.histogram();
    let owned = sub.to_owned_image().unwrap();
    println!(
        "[BENCHMARK] test_19_zero_copy_view_and_subview: {:?}",
        start.elapsed()
    );

    assert_eq!(view.width(), 4);
    assert_eq!(sub.get_pixel(0, 0).unwrap(), 2);
    assert_eq!(sub.get_pixel(1, 1).unwrap(), 3);
    assert_eq!(hist[2], 1);
    assert_eq!(hist[3], 1);
    assert_eq!(owned.get_pixel(1, 1).unwrap(), 3);
}

#[test]
fn test_20_simd_pack_unpack_roundtrip() {
    let quats: Vec<u8> = (0..128).map(|i| (i % 4) as u8).collect();
    let mut packed = vec![0u8; 32];
    let mut unpacked = vec![0u8; 128];

    let start = Instant::now();
    pack_quats_simd(&quats, &mut packed).unwrap();
    unpack_quats_simd(&packed, &mut unpacked).unwrap();
    let invalid_res = pack_quats_simd(&[0, 1, 4, 0], &mut [0u8; 1]);
    println!(
        "[BENCHMARK] test_20_simd_pack_unpack_roundtrip: {:?}",
        start.elapsed()
    );

    assert_eq!(unpacked, quats);
    assert!(matches!(invalid_res, Err(QuatImageError::InvalidQuatValue(4))));
}

#[cfg(feature = "parallel")]
#[test]
fn test_21_tiled_rotate_and_parallel_ops() {
    let mut img = QuatImage::new(64, 64).unwrap();
    for y in 0..64 {
        for x in 0..64 {
            img.set_pixel(x, y, ((x + y) % 4) as u8).unwrap();
        }
    }

    let start = Instant::now();
    let mut rotated = img.clone();
    rotated.rotate_90_cw_tiled().unwrap();
    let dilated = img.dilate_par().unwrap();
    let eroded = img.erode_par().unwrap();
    let hist = img.histogram_par();
    println!(
        "[BENCHMARK] test_21_tiled_rotate_and_parallel_ops: {:?}",
        start.elapsed()
    );

    assert_eq!(rotated.width(), 64);
    assert_eq!(rotated.height(), 64);
    assert_eq!(dilated.width(), 64);
    assert_eq!(eroded.height(), 64);
    assert_eq!(hist.iter().sum::<usize>(), 64 * 64);
}

#[cfg(feature = "io")]
#[test]
fn test_22_binary_v1_and_rle_roundtrip() {
    let mut img = QuatImage::new(8, 8).unwrap();
    img.set_pixel(1, 2, 1).unwrap();
    img.set_pixel(5, 6, 3).unwrap();

    let v1_path = std::env::temp_dir().join("quat_image_test_v1.qimg");
    let rle_path = std::env::temp_dir().join("quat_image_test_v1.qrle");

    let start = Instant::now();
    img.save_to_file_v1(&v1_path).unwrap();
    let loaded_v1 = QuatImage::load_from_file_v1(&v1_path).unwrap();

    img.save_to_file_rle(&rle_path).unwrap();
    let loaded_rle = QuatImage::load_from_file_rle(&rle_path).unwrap();
    println!(
        "[BENCHMARK] test_22_binary_v1_and_rle_roundtrip: {:?}",
        start.elapsed()
    );

    assert_eq!(loaded_v1, img);
    assert_eq!(loaded_rle, img);

    let _ = std::fs::remove_file(v1_path);
    let _ = std::fs::remove_file(rle_path);
}

}