mod packing;
mod image;
mod ffi;
mod error;
mod io;

pub use packing::*;
pub use image::*;
pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::QuatImageError;
    use crate::packing::{pack_quats, unpack_quats};
    use crate::image::QuatImage;

    // ==========================================
    // 1. Osnovni testovi (Najlakši - Happy Path)
    // ==========================================

    // Test 1: Kreiranje slike i provera dimenzija
    #[test]
    fn test_01_basic_creation() {
        let img = QuatImage::new(10, 20).unwrap();
        assert_eq!(img.width(), 10);
        assert_eq!(img.height(), 20);
    }

    // Test 2: Upisivanje i čitanje validnog piksela (0-3)
    #[test]
    fn test_02_set_and_get_pixel() {
        let mut img = QuatImage::new(4, 4).unwrap();
        img.set_pixel(1, 2, 3).unwrap();
        assert_eq!(img.get_pixel(1, 2).unwrap(), 3);
        
        img.set_pixel(0, 0, 0).unwrap();
        assert_eq!(img.get_pixel(0, 0).unwrap(), 0);
    }

    // Test 3: Resetovanje/čišćenje cele slike
    #[test]
    fn test_03_clear_image() {
        let mut img = QuatImage::new(2, 2).unwrap();
        img.set_pixel(0, 0, 2).unwrap();
        img.set_pixel(1, 1, 3).unwrap();
        
        img.clear();
        
        assert_eq!(img.get_pixel(0, 0).unwrap(), 0);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 0);
    }

    // Test 4: Direktno pakovanje i raspakivanje kvatova (unit test za packing.rs)
    #[test]
    fn test_04_pack_unpack_unit() {
        let packed = pack_quats(1, 2, 3, 0).unwrap();
        let (q0, q1, q2, q3) = unpack_quats(packed);
        assert_eq!(q0, 1);
        assert_eq!(q1, 2);
        assert_eq!(q2, 3);
        assert_eq!(q3, 0);
    }

    // ==========================================
    // 2. Testovi grešaka i rubnih slučajeva
    // ==========================================

    // Test 5: Greška pri upisu nevalidne vrednosti (> 3)
    #[test]
    fn test_05_invalid_pixel_value() {
        let mut img = QuatImage::new(4, 4).unwrap();
        let res = img.set_pixel(0, 0, 4); // Dozvoljeno je samo 0, 1, 2, 3
        assert!(matches!(res, Err(QuatImageError::InvalidQuatValue(4))));

        let pack_res = pack_quats(5, 0, 0, 0);
        assert!(matches!(pack_res, Err(QuatImageError::InvalidQuatValue(5))));
    }

    // Test 6: Greška van granica (OutOfBounds)
    #[test]
    fn test_06_out_of_bounds() {
        let mut img = QuatImage::new(5, 5).unwrap();
        assert!(matches!(img.get_pixel(5, 0), Err(QuatImageError::OutOfBounds)));
        assert!(matches!(img.get_pixel(0, 5), Err(QuatImageError::OutOfBounds)));
        assert!(matches!(img.set_pixel(10, 10, 1), Err(QuatImageError::OutOfBounds)));
    }

    // Test 7: Nevalidne dimenzije slike (nula širina ili visina)
    #[test]
    fn test_07_invalid_dimensions() {
        assert!(matches!(QuatImage::new(0, 10), Err(QuatImageError::InvalidDimensions)));
        assert!(matches!(QuatImage::new(10, 0), Err(QuatImageError::InvalidDimensions)));
        assert!(matches!(QuatImage::new(0, 0), Err(QuatImageError::InvalidDimensions)));
    }

    // ==========================================
    // 3. Kompleksniji i stresni testovi
    // ==========================================

    // Test 8: Transformacije slike (Horizontalni i Vertikalni flip)
    #[test]
    fn test_08_transformations_flip() {
        let mut img = QuatImage::new(2, 2).unwrap();
        // Red 0: [1, 2]
        // Red 1: [0, 3]
        img.set_pixel(0, 0, 1).unwrap();
        img.set_pixel(1, 0, 2).unwrap();
        img.set_pixel(0, 1, 0).unwrap();
        img.set_pixel(1, 1, 3).unwrap();

        // Horizontalni flip -> Red 0 postaje [2, 1], Red 1 postaje [3, 0]
        img.flip_horizontal().unwrap();
        assert_eq!(img.get_pixel(0, 0).unwrap(), 2);
        assert_eq!(img.get_pixel(1, 0).unwrap(), 1);
        assert_eq!(img.get_pixel(0, 1).unwrap(), 3);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 0);

        // Vertikalni flip nazad
        img.flip_vertical().unwrap();
        assert_eq!(img.get_pixel(0, 0).unwrap(), 3);
        assert_eq!(img.get_pixel(1, 0).unwrap(), 0);
    }

    // Test 9: Zaštita od prekoračenja memorije (Integer Overflow)
    #[test]
    fn test_09_integer_overflow_protection() {
        let res = QuatImage::new(usize::MAX, usize::MAX);
        assert!(matches!(res, Err(QuatImageError::IntegerOverflow)));
    }

    // Test 10: Najstresniji test izdržljivosti (Velika mreža od 1000x1000 piksela, masivan upis/čitanje i kaskadne transformacije)
    #[test]
    fn test_10_stress_endurance_massive_grid() {
        const WIDTH: usize = 1000;
        const HEIGHT: usize = 1000;
        let mut img = QuatImage::new(WIDTH, HEIGHT).unwrap();

        // Masivno upisivanje preko 1.000.000 piksela sa kalkulisanim vrednostima (0-3)
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let val = ((x + y) % 4) as u8;
                img.set_pixel(x, y, val).unwrap();
            }
        }

        // Provera integriteta za svaki pojedinačni piksel posle pakovanja u bajtove
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let expected = ((x + y) % 4) as u8;
                let actual = img.get_pixel(x, y).unwrap();
                assert_eq!(actual, expected, "Data mismatch at coordinate ({}, {})", x, y);
            }
        }

        // Stres testiranje rotacija/flipovanja na ogromnom nizu
        img.flip_horizontal().unwrap();
        img.flip_vertical().unwrap();

        // Provera tačnosti ugla nakon višestrukih transformacija
        let corner_val = img.get_pixel(0, 0).unwrap();
        let expected_corner = (((WIDTH - 1) + (HEIGHT - 1)) % 4) as u8;
        assert_eq!(corner_val, expected_corner);
    }
    #[test]
fn test_rotate_90_cw() {
    let mut img = QuatImage::new(2, 1).unwrap();
    img.set_pixel(0, 0, 1).unwrap();
    img.set_pixel(1, 0, 2).unwrap();

    img.rotate_90_cw().unwrap();

    assert_eq!(img.width(), 1);
    assert_eq!(img.height(), 2);
    assert_eq!(img.get_pixel(0, 0).unwrap(), 2);
    assert_eq!(img.get_pixel(0, 1).unwrap(), 1);
}

// Test for calculating histogram
#[test]
fn test_histogram() {
    let mut img = QuatImage::new(2, 2).unwrap();
    img.set_pixel(0, 0, 1).unwrap();
    img.set_pixel(1, 0, 1).unwrap();
    img.set_pixel(0, 1, 3).unwrap();
    // (1, 1) remains 0 by default

    let hist = img.histogram();
    assert_eq!(hist[0], 1); // 1 pixel with value 0
    assert_eq!(hist[1], 2); // 2 pixels with value 1
    assert_eq!(hist[2], 0); // 0 pixels with value 2
    assert_eq!(hist[3], 1); // 1 pixel with value 3
  }
  #[test]
    fn test_pack_quats_slice_batch() {
        let quats = vec![0, 1, 2, 3, 3, 2, 1, 0];
        let mut packed = vec![0u8; 2];
        
        // Test uspešnog grupnog pakovanja
        let res = pack_quats_slice(&quats, &mut packed);
        assert!(res.is_ok());
        
        // Provera ispravnosti bitovskog pakovanja
        assert_eq!(packed[0], (3 << 6) | (2 << 4) | (1 << 2) | 0);
        assert_eq!(packed[1], (0 << 6) | (1 << 4) | (2 << 2) | 3);

        // Test greške pri neadekvatnoj veličini niza (nije deljiv sa 4)
        let invalid_quats = vec![0, 1, 2];
        let mut invalid_packed = vec![0u8; 1];
        assert!(matches!(
            pack_quats_slice(&invalid_quats, &mut invalid_packed),
            Err(QuatImageError::InvalidDimensions)
        ));
    }

    #[test]
    fn test_morphological_dilation() {
        let mut img = QuatImage::new(3, 3).unwrap();
        // Postavljamo jedan piksel sa maksimalnom vrednošću 3 u sredinu
        img.set_pixel(1, 1, 3).unwrap();
        
        // Dilatacija (max filter u 3x3 okolini) treba da proširi vrednost 3 na sve okolne piksele
        let dilated = img.dilate().unwrap();
        
        assert_eq!(dilated.get_pixel(0, 0).unwrap(), 3);
        assert_eq!(dilated.get_pixel(1, 0).unwrap(), 3);
        assert_eq!(dilated.get_pixel(2, 2).unwrap(), 3);
    }

    #[test]
    fn test_replace_color_parallel() {
        let mut img = QuatImage::new(3, 3).unwrap();
        img.set_pixel(0, 0, 1).unwrap();
        img.set_pixel(2, 2, 1).unwrap();
        
        // Paralelna zamena vrednosti 1 sa vrednošću 2 preko Rayon-a
        img.replace_color_par(1, 2).unwrap();
        
        assert_eq!(img.get_pixel(0, 0).unwrap(), 2);
        assert_eq!(img.get_pixel(2, 2).unwrap(), 2);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 0); // Ostalo nevažno/podrazumevano
    }

    #[test]
    fn test_rgb8_quantization_conversion() {
        // Simuliramo 4 piksela standardnog RGB niza (tamno, srednje-nisko, srednje-visoko, svetlo)
        let rgb_data = vec![
            10, 10, 10,     // Treba da se mapira u kvart 0
            80, 80, 80,     // Treba da se mapira u kvart 1
            150, 150, 150,  // Treba da se mapira u kvart 2
            240, 240, 240,  // Treba da se mapira u kvart 3
        ];
        
        let img = QuatImage::from_rgb8(2, 2, &rgb_data).unwrap();
        
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.get_pixel(0, 0).unwrap(), 0);
        assert_eq!(img.get_pixel(1, 0).unwrap(), 1);
        assert_eq!(img.get_pixel(0, 1).unwrap(), 2);
        assert_eq!(img.get_pixel(1, 1).unwrap(), 3);
    }
}