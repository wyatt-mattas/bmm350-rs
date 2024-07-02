use core::fmt::Debug;

/// Possible errors that can occur when interacting with the BMI323
#[derive(Debug)]
pub enum Error<E> {
    /// Communication error
    Comm(E),
    /// Invalid device (wrong chip ID)
    InvalidDevice,
    /// Invalid configuration
    InvalidConfig,
}
