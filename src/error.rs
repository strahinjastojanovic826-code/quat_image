use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuatImageError {
    OutOfBounds,
    InvalidQuatValue(u8),
    InvalidDimensions,
    IntegerOverflow,
}

impl fmt::Display for QuatImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuatImageError::OutOfBounds => write!(f, "Koordinate su van granica slike"),
            QuatImageError::InvalidQuatValue(val) => write!(f, "Kvatarna vrednost mora biti između 0 i 3, uneto je: {}", val),
            QuatImageError::InvalidDimensions => write!(f, "Širina i visina moraju biti veće od 0"),
            QuatImageError::IntegerOverflow => write!(f, "Došlo je do prekoračenja memorije pri proračunu dimenzija"),
        }
    }
}

impl std::error::Error for QuatImageError {}