/// BMM350 register addresses and constant values
pub struct Core_Register;
impl Core_Register {
    /// Chip ID register address
    pub const CHIPID: u8 = 0x00;
    /// Error register address
    pub const ERR_REG: u8 = 0x02;
    /// Command register address
    pub const CMD: u8 = 0x7E;
    /// Expected chip ID for BMI323
    pub const BMM350_CHIP_ID: u8 = 0x33;
    /// Soft reset command value
    pub const CMD_SOFT_RESET: u8 = 0xB6;
}

pub struct Data_Register;
impl Data_Register {
    pub const BMM350_REG_MAG_X_XLSB: u8 = 0x31;
    pub const BMM350_REG_MAG_X_LSB: u8 = 0x32;
    pub const BMM350_REG_MAG_X_MSB: u8 = 0x33;
    pub const BMM350_REG_MAG_Y_XLSB: u8 = 0x34;
    pub const BMM350_REG_MAG_Y_LSB: u8 = 0x35;
    pub const BMM350_REG_MAG_Y_MSB: u8 = 0x36;
    pub const BMM350_REG_MAG_Z_XLSB: u8 = 0x37;
    pub const BMM350_REG_MAG_Z_LSB: u8 = 0x38;
    pub const BMM350_REG_MAG_Z_MSB: u8 = 0x39;
    pub const BMM350_REG_TEMP_XLSB: u8 = 0x3A;
    pub const BMM350_REG_TEMP_LSB: u8 = 0x3B;
    pub const BMM350_REG_TEMP_MSB: u8 = 0x3C;
    pub const BMM350_REG_SENSORTIME_XLSB: u8 = 0x3D;
    pub const BMM350_REG_SENSORTIME_LSB: u8 = 0x3E;
    pub const BMM350_REG_SENSORTIME_MSB: u8 = 0x3F;
}
