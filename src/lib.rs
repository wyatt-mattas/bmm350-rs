mod interface;
mod registers;
mod types;
pub use registers::{CoreRegister, DataRegister};
pub mod device;

pub use types::Error;

/// Main struct representing the BMM350 device
pub struct Bmm350<DI, D> {
    /// Communication interface (I2C or SPI)
    iface: DI,
    /// Delay provider
    delay: D,
}
