use std::{error::Error, fmt::Display};

mod emulator;
mod macho;
pub mod nac;

#[derive(Debug)]
pub struct AbsintheError(i32);

impl Display for AbsintheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AbsintheError {}

impl From<unicorn_engine::unicorn_const::uc_error> for AbsintheError {
    fn from(error: unicorn_engine::unicorn_const::uc_error) -> Self {
        eprintln!("NAC emulator: {error:?}");
        Self(-10000 - error as i32)
    }
}

pub(crate) fn failure(message: impl Display) -> AbsintheError {
    eprintln!("NAC: {message}");
    AbsintheError(-10000)
}
