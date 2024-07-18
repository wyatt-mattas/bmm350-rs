use crate::{
    interface::{I2cInterface, ReadData, SpiInterface, WriteData},
    Bmm350, CoreRegister, Error,
};
use embedded_hal::delay::DelayNs;

impl<I2C, D> Bmm350<I2cInterface<I2C>, D>
where
    D: DelayNs,
{
    /// Create a new BMI323 device instance
    ///
    /// # Arguments
    ///
    /// * `iface` - The communication interface
    /// * `delay` - A delay provider
    pub fn new_with_i2c(i2c: I2C, address: u8, delay: D) -> Self {
        Bmm350 {
            iface: I2cInterface { i2c, address },
            delay,
        }
    }
}

impl<SPI, D> Bmm350<SpiInterface<SPI>, D>
where
    D: DelayNs,
{
    /// Create a new BMI323 device instance
    ///
    /// # Arguments
    ///
    /// * `iface` - The communication interface
    /// * `delay` - A delay provider
    pub fn new_with_spi(spi: SPI, delay: D) -> Self {
        Bmm350 {
            iface: SpiInterface { spi },
            delay,
        }
    }
}

impl<DI, D, E> Bmm350<DI, D>
where
    DI: ReadData<Error = Error<E>> + WriteData<Error = Error<E>>,
    D: DelayNs,
{
    /// Initialize the device
    pub fn init(&mut self) -> Result<(), Error<E>> {
        self.write_register(CoreRegister::CMD, CoreRegister::CMD_SOFT_RESET)?;
        self.delay.delay_us(2000);

        //let mut reg_data = [0u8; 3];
        //reg_data[0] = 0x01; // sensor error conditins register
        let status = self.read_register(CoreRegister::ERR_REG)?;
        if (status & 0b0000_0001) != 0 {
            return Err(Error::InvalidDevice);
        }

        let result = self.read_register(CoreRegister::CHIPID)?;
        if result != CoreRegister::BMM350_CHIP_ID {
            return Err(Error::InvalidDevice);
        }

        Ok(())
    }

    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), Error<E>> {
        self.iface.write_data(&[reg, value])
    }

    fn read_register(&mut self, reg: u8) -> Result<u8, Error<E>> {
        self.iface.read_register(reg)
    }

    /*
    fn read_data<'a>(&mut self, data: &'a mut [u8]) -> Result<&'a [u8], Error<E>> {
        self.iface.read_data(data)
        } */
}
