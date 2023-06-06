use std::{fmt, str};

use thiserror::Error;

pub const DIGIT_COUNT: usize = 5;
const ASCII_OFFSET: u8 = b' ';
pub const VERSION: Version = Version(1);

pub struct Version(u16);
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{:0digi$}", self.0, digi = DIGIT_COUNT)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "The version string is too short, it should have {DIGIT_COUNT} \
        digits, yet it has {0} characters."
    )]
    TooShort(usize),
    #[error("The version string contains non-ascii characters.")]
    NotUtf8,
    #[error("The version string doesn't end with {DIGIT_COUNT} digits, it ended with '{0}'")]
    NotDigit(String),
}

const fn digits_represents(version: u16, ascii: &[u8]) -> bool {
    let least_significant = version % 10;
    let prefix = (version - least_significant) / 10;

    let Some((&ascii_last_digit, ascii)) = ascii.split_last() else { return false; };
    let last_digit = ascii_last_digit - ASCII_OFFSET;

    last_digit == least_significant as u8 && digits_represents(prefix, ascii)
}

impl Version {
    pub fn get_version_slice(ascii: &[u8]) -> Result<u16, Error> {
        let too_short = Error::TooShort(ascii.len());

        let ascii = ascii.rchunks(DIGIT_COUNT).next().ok_or(too_short)?;
        let encoded = str::from_utf8(ascii).map_err(|_| Error::NotUtf8)?;

        let not_digit = |_| Error::NotDigit(encoded.to_string());
        encoded.parse().map_err(not_digit)
    }
    /// True if `ascii`'s trailing `DIGIT_COUNT` characters represent in ASCII
    /// this version number.
    pub fn digits_represents(self, ascii: &[u8]) -> bool {
        let len = ascii.len();
        digits_represents(self.0, &ascii[len - DIGIT_COUNT..])
    }
}
