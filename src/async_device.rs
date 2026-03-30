use crate::{
    // async_interface::{AsyncI2cInterface, ReadData, WriteData},
    types::{
        AxisEnableDisable, DataRate, MagCompensation, PerformanceMode, PmuCmdStatus0,
        PowerMode, Sensor3DData, Sensor3DDataScaled,
    },
    AverageNum, AsyncBmm350, InterruptDrive, InterruptEnableDisable, InterruptLatch, InterruptMap,
    InterruptPolarity, MagConfig, Register,
};
#[cfg(feature = "defmt")]
use defmt::error;
use embedded_hal_async::{delay::DelayNs, i2c::I2c};

#[cfg(feature = "defmt")]
fn log_bmm_error_code(err_value: u8) {
    error!("BMM Error: {}", err_value);
}

#[cfg(not(feature = "defmt"))]
fn log_bmm_error_code(_err_value: u8) {}

#[cfg(feature = "defmt")]
fn log_bmm_i2c_error<E>(_err: &E) {
    error!("BMM I2C communication error");
}

#[cfg(not(feature = "defmt"))]
fn log_bmm_i2c_error<E>(_err: &E) {}

impl<I2C: embedded_hal_async::i2c::I2c, D> AsyncBmm350<I2C, D> where I2C: I2c, D: DelayNs{
    /// Create a new BMM350 device instance
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C interface
    /// * `address` - The I2C address of the device
    /// * `delay` - A delay provider
    pub fn new_with_i2c(i2c: I2C, address: u8, delay: D) -> Self {
        AsyncBmm350 {
            i2c,
            address,
            delay,
            mag_range: 1000.0,
            var_id: 0,
            mag_comp: MagCompensation::default(), // Default range in uT
        }
    }
}

impl<I2C: embedded_hal_async::i2c::I2c, D> AsyncBmm350<I2C, D> where D: DelayNs{
    /// Initialize the device
    pub async fn init(&mut self) -> Result<i8, I2C::Error> {
        self.delay.delay_us(3_000).await;

        self.write_register_16bit(Register::CMD, Register::CMD_SOFT_RESET).await?;

        self.delay.delay_us(24_000).await;

        let err_result = self.read_register(Register::ERR_REG).await;
        let mut value_return = 0;
        match err_result{
            Ok(err_value)=>{
                if err_value != 0 {
                    log_bmm_error_code(err_value);
                    value_return = -1;
                }
                else{
                    value_return = 0;
                }
            }
            Err(i2c_error)=>{
                log_bmm_i2c_error(&i2c_error);
                return Err(i2c_error);
            }
        }

        let chip_id_result = self.read_register(Register::CHIPID).await;
        match chip_id_result{
            Ok(chip_id)=>{
                if chip_id != Register::BMM350_CHIP_ID {
                    value_return = -1;
                }
                else{
                    value_return = 0;
                }
            }
            Err(i2c_error)=>{
                value_return = -1;
                return Err(i2c_error);
            }
        }

        // Perform OTP dump after boot
        self.otp_dump_after_boot().await?;

        // Power off OTP
        self.write_register(Register::OTP_CMD_REG, Register::OTP_CMD_PWR_OFF_OTP).await?;

        self.magnetic_reset().await?;

        Ok(value_return)
    }

    async fn otp_dump_after_boot(&mut self) -> Result<(), I2C::Error> {
        let mut otp_data = [0u16; 32];

        for i in 0..32 {
            otp_data[i] = self.read_otp_word(i as u8).await?;
        }

        self.var_id = ((otp_data[30] & 0x7f00) >> 9) as u8;

        // Update magnetometer offset and sensitivity data
        self.update_mag_compensation(&otp_data).await?;

        Ok(())
    }

    async fn read_otp_word(&mut self, addr: u8) -> Result<u16, I2C::Error> {
        let otp_cmd = 0x20 | (addr & 0x1F); // OTP read command
        self.write_register(Register::OTP_CMD_REG, otp_cmd).await?;

        // Wait for OTP read to complete
        for _ in 0..10 {
            self.delay.delay_us(300);
            let status = self.read_register(Register::OTP_STATUS_REG).await?;
            if status & 0x01 != 0 {
                break;
            }
        }

        let msb = self.read_register(Register::OTP_DATA_MSB_REG).await?;
        let lsb = self.read_register(Register::OTP_DATA_LSB_REG).await?;

        Ok(((msb as u16) << 8) | (lsb as u16) & 0xFFFF)
    }

  


    async fn update_mag_compensation(&mut self, otp_data: &[u16; 32]) -> Result<(), I2C::Error> {
        // Implement the logic to update magnetometer compensation data
        // This is a simplified version and may need to be expanded based on the specific BMM350 requirements
        self.mag_comp = MagCompensation {
            offset_x: self.extract_signed_12bit(otp_data[0x0E] & 0x0FFF),
            offset_y: self.extract_signed_12bit(((otp_data[16] & 0xF000) >> 4) + (otp_data[0x0F] & 0x00FF)),
            offset_z: self.extract_signed_12bit((otp_data[17] & 0x0F00) + (otp_data[0x10] & 0x00FF)),
            // Add more fields as necessary
        };

        Ok(())
    }
    
    fn extract_signed_12bit(&self, value: u16) -> i16 {
        let value = value & 0x0FFF; // Ensure only 12 bits are used
        if value & 0x0800 != 0 {
            (value as i16) - 0x1000
        } else {
            value as i16
        }
    }
    fn get_lsb_to_ut_degc() -> [f32; 4] {
        let bxy_sens = 14.55;
        let bz_sens = 9.0;
        let temp_sens = 0.00204;

        let ina_xy_gain_trgt = 19.46;
        let ina_z_gain_trgt = 31.0;

        let adc_gain = 1.0 / 1.5;
        let lut_gain = 0.714607238769531;
        let power = 1_000_000.0 / 1_048_576.0;

        [
            power / (bxy_sens * ina_xy_gain_trgt * adc_gain * lut_gain), // X
            power / (bxy_sens * ina_xy_gain_trgt * adc_gain * lut_gain), // Y
            power / (bz_sens  * ina_z_gain_trgt * adc_gain * lut_gain),  // Z
            1.0 / (temp_sens * adc_gain * lut_gain * 1_048_576.0),       // Temp
        ]
    }

    /// Perform magnetic reset of the sensor.
    /// This is necessary after a field shock (400mT field applied to sensor).
    /// It performs both a bit reset and flux guide reset in suspend mode.
    pub async fn magnetic_reset(&mut self) -> Result<i8, I2C::Error> {
        let mut return_value = 0;

        // Check if we're in normal mode
        let mut restore_normal = false;
        let mut pmu_status = self.read_pmu_cmd_status_0().await?;

        // If we're in normal mode, we need to go to suspend first
        if pmu_status.power_mode_is_normal == 0x1 {
            restore_normal = true;
            self.set_power_mode(PowerMode::Suspend).await?;
        }

        // Set Bit Reset (BR) command
        // TODO set BitReset as register instead of PowerMode enum
        self.write_register(Register::PMU_CMD, PowerMode::BitReset as u8).await?;
        self.delay.delay_us(14_000); // BR_DELAY

        // Verify BR status
        pmu_status = self.read_pmu_cmd_status_0().await?;
        if pmu_status.pmu_cmd_value != Register::PMU_CMD_STATUS_0_BR {
            return_value = -1;
            return Ok(return_value);
        }

        // Set Flux Guide Reset (FGR) command
        // TODO set FluxGuideReset as register instead of PowerMode enum
        self.write_register(Register::PMU_CMD, PowerMode::FluxGuideReset as u8).await?;
        self.delay.delay_us(18_000); // FGR_DELAY

        // Verify FGR status
        let pmu_status = self.read_pmu_cmd_status_0().await?;
        if pmu_status.pmu_cmd_value != Register::PMU_CMD_STATUS_0_FGR {
            return_value = -1;
            return Ok(return_value);
        }

        // Restore normal mode if we were in it before
        if restore_normal {
            self.set_power_mode(PowerMode::Normal).await?;
        }

        Ok(return_value)
    }

    /// Read the PMU command status register 0
    async fn read_pmu_cmd_status_0(&mut self) -> Result<PmuCmdStatus0, I2C::Error> {
        let status = self.read_register(Register::PMU_CMD_STATUS_0).await?;

        Ok(PmuCmdStatus0 {
            pmu_cmd_busy: (status & 0x01),
            odr_overwrite: (status & 0x2) >> 0x1,
            avg_overwrite: (status & 0x4) >> 0x2,
            power_mode_is_normal: (status & 0x8) >> 0x3,
            cmd_is_illegal: (status & 0x10) >> 0x4,
            pmu_cmd_value: (status & 0xE0) >> 5,
        })
    }

    /// Set the magnetometer configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The magnetometer configuration
    pub async fn set_mag_config(&mut self, config: MagConfig) -> Result<(), I2C::Error> {
        let reg_data = u16::from(config);
        self.write_register_16bit(Register::PMU_CMD_AGGR_SET, reg_data).await?;

        // Wait for magnetometer data to be ready
        self.wait_for_data_ready().await;
        Ok(())
    }

    /// Set the power mode of the sensor
    ///
    /// # Arguments
    ///
    /// * `mode` - The power mode to set
    pub async fn set_power_mode(&mut self, mode: PowerMode) -> Result<i8, I2C::Error> {
        let value_return = 0;
        
        let last_pwr = self.read_register(Register::REG_PMU_CMD).await?;
        if last_pwr > Register::PMU_CMD_NM_TC {
            return Ok(-1);
        }
        if last_pwr == Register::PMU_CMD_NM || last_pwr == Register::PMU_CMD_UPD_OAE {
            self.write_register(Register::REG_PMU_CMD, Register::PMU_CMD_SUS).await?;
            self.delay.delay_us(6_000);
        }
        self.power_mode(mode).await?;
        Ok(0)
    }

    async fn power_mode(&mut self, mode: PowerMode) -> Result<(), I2C::Error> {
        let sus_to_forced_mode: [u32; 4] = [
            Register::SUS_TO_FORCEDMODE_NO_AVG_DELAY,
            Register::SUS_TO_FORCEDMODE_AVG_2_DELAY,
            Register::SUS_TO_FORCEDMODE_AVG_4_DELAY,
            Register::SUS_TO_FORCEDMODE_AVG_8_DELAY,
        ];

        /* Array to store suspend to forced mode fast delay */
        let sus_to_forced_mode_fast: [u32; 4] = [
            Register::SUS_TO_FORCEDMODE_FAST_NO_AVG_DELAY,
            Register::SUS_TO_FORCEDMODE_FAST_AVG_2_DELAY,
            Register::SUS_TO_FORCEDMODE_FAST_AVG_4_DELAY,
            Register::SUS_TO_FORCEDMODE_FAST_AVG_8_DELAY,
        ];

        self.write_register(Register::REG_PMU_CMD, mode as u8).await?;
        let get_avg: u8 = self.read_register(Register::REG_PMU_CMD_AGGR_SET).await?;
        let avg = (get_avg & Register::AVG_MASK) >> Register::AVG_POS;
        let mut delay_us = 0;
        match mode {
            PowerMode::Normal => {
                delay_us = 38_000;
            }
            PowerMode::Forced => {
                delay_us = sus_to_forced_mode[avg as usize];
            }
            PowerMode::ForcedFast => {
                delay_us = sus_to_forced_mode_fast[avg as usize];
            }
            _ => {}
        }

        self.delay.delay_us(delay_us).await;

        Ok(())
    }

    /// Enable or disable axes
    ///
    /// # Arguments
    ///
    /// * `x` - Enable or disable X axis
    /// * `y` - Enable or disable Y axis
    /// * `z` - Enable or disable Z axis
    pub async fn enable_axes(&mut self, x: AxisEnableDisable, y: AxisEnableDisable, z: AxisEnableDisable) -> Result<(), I2C::Error> {
        let mut reg_data: u8 = 0;
        reg_data = ((x as u8) & 0x01)
            | ((reg_data & 0x02) | ((y as u8) << 0x1) & 0x02)
            | ((reg_data & 0x04) | ((z as u8) << 0x2) & 0x04);
        let result = self.write_register(Register::PMU_CMD_AXIS_EN, reg_data).await;
        match result{
            Ok(_)=>{
                return Ok(());
            }
            Err(i2c_error)=>{
                return Err(i2c_error);
            }
        }
    }

    /// Read the raw magnetometer data
    pub async fn read_mag_data(&mut self) -> Result<Sensor3DData, I2C::Error> {
        // Prepare a buffer: 1 byte for start address + 9 bytes for data (X, Y, Z)
        const DATA_LEN: usize = 9;
        const BUFFER_LEN: usize = 1 + DATA_LEN;
        let mut buffer = [0u8; BUFFER_LEN]; // Size 10
        buffer[0] = Register::MAG_X_LSB; // Start address 0x31

        // read_data will return a slice referencing buffer[1..10] containing the 9 data bytes
        let sensor_data_slice = self.read_data(&mut buffer[0..BUFFER_LEN]).await?;

        // Helper function for 24-bit signed reconstruction (still needed!)
        fn reconstruct_signed_24bit(xlsb: u8, lsb: u8, msb: u8) -> i32 {
            let unsigned_val = (xlsb as u32) | ((lsb as u32) << 8) | ((msb as u32) << 16);
            if (msb & 0x80) != 0 {
                (unsigned_val | 0xFF000000) as i32 // Manual sign extension
            } else {
                unsigned_val as i32
            }
        }

        
        // Use indices relative to the returned slice
        Ok(Sensor3DData {
            x: reconstruct_signed_24bit(
                sensor_data_slice[0],
                sensor_data_slice[1],
                sensor_data_slice[2],
            ),
            y: reconstruct_signed_24bit(
                sensor_data_slice[3],
                sensor_data_slice[4],
                sensor_data_slice[5],
            ),
            z: reconstruct_signed_24bit(
                sensor_data_slice[6],
                sensor_data_slice[7],
                sensor_data_slice[8],
            ),
        })     
    }

    pub async fn read_mag_data_scaled(&mut self)->Result<Sensor3DDataScaled, I2C::Error>{
        let coefficients = Self::get_lsb_to_ut_degc();
        let mag_data_raw_result = self.read_mag_data().await;
        match mag_data_raw_result{
            Ok(mag_data_raw)=>{
                Ok(mag_data_raw.to_ut(coefficients))
            }
            Err(i2c_error)=>{
                Err(i2c_error)
            }
        }
    }

    /// Perform a self-test
    // TODO fix this
    async fn perform_self_test(&mut self) -> Result<bool, I2C::Error> {
        // Save current configuration
        let current_power_mode = self.read_register(Register::PMU_CMD).await?;
        let current_odr = self.read_register(Register::PMU_CMD_AGGR_SET).await?;

        // Set device to normal mode and 100Hz ODR
        self.set_power_mode(PowerMode::Normal).await?;
        self.set_mag_config(
            MagConfig::builder()
                .odr(DataRate::ODR100Hz)
                .performance(PerformanceMode::Regular)
                .build(),
        ).await?;

        // Perform self-test
        let self_test_passed = true;

        // Restore original configuration
        self.write_register(Register::PMU_CMD, current_power_mode).await?;
        self.write_register(Register::PMU_CMD_AGGR_SET, current_odr).await?;

        Ok(self_test_passed)
    }

    /// Set the output data rate and performance mode
    pub async fn set_odr_performance(&mut self,odr: DataRate,performance: AverageNum,) -> Result<(), I2C::Error> {
        let reg_data = (odr as u8) & 0xf;
        let new_reg_data = (reg_data & Register::AVG_MASK)
            | ((performance as u8) << Register::AVG_POS) & Register::AVG_MASK;

        self.write_register(Register::PMU_CMD_AGGR_SET, new_reg_data).await?;
        self.write_register(Register::PMU_CMD, Register::PMU_CMD_UPD_OAE).await?;

        self.delay.delay_us(1_000).await;
        Ok(())
    }

    /// Enable or disable the data ready interrupt
    pub async fn enable_interrupt(&mut self, enable: InterruptEnableDisable) -> Result<(), I2C::Error> {
        self.read_register(Register::INT_CTRL).await?;
        let reg_data: u8 = 0;
        let new_reg_data = (reg_data & (0x80)) | (((enable as u8) << 0x7) & 0x80);
        let result = self.write_register(Register::INT_CTRL, new_reg_data).await;
        match result{
            Ok(_)=>{
                Ok(())
            }
            Err(i2c_error)=>{
                Err(i2c_error)
            }
        }

    }

    /// Configure interrupt settings
    pub async fn configure_interrupt(&mut self, latch: InterruptLatch, polarity: InterruptPolarity, drive: InterruptDrive, map: InterruptMap, ) -> Result<(), I2C::Error> {
        self.read_register(Register::INT_CTRL).await?;
        let mut reg_data: u8 = 0;
        reg_data = ((reg_data & (0x1)) | (latch as u8 & 0x1))
            | ((reg_data & (0x2)) | ((polarity as u8) << 0x1) & 0x2)
            | ((reg_data & (0x4)) | ((drive as u8) << 0x2) & 0x4)
            | ((reg_data & (0x8)) | ((map as u8) << 0x3) & 0x8);
        let result =  self.write_register(Register::INT_CTRL, reg_data).await;
        match result{
            Ok(_)=>{
                Ok(())
            }
            Err(i2c_error)=>{
                Err(i2c_error)
            }
        }
    }

    /// Read the interrupt status
    pub async fn get_interrupt_status(&mut self) -> Result<bool, I2C::Error> {
        let status = self.read_register(Register::STATUS).await?;
        Ok((status & 0x04) != 0)
    }

    /// Set the I2C watchdog timer
    pub async fn set_i2c_watchdog(&mut self, enable: bool, long_timeout: bool) -> Result<(), I2C::Error> {
        let reg_data = (enable as u8) | ((long_timeout as u8) << 1);
        self.write_register(Register::I2C_WDT_SET, reg_data).await
    }

    async fn write_register_16bit(&mut self, reg: u8, value: u16) -> Result<(), I2C::Error> {
        let bytes = value.to_le_bytes();
        self.write_data(&[reg, bytes[0], bytes[1]]).await
    }

    async fn write_register(&mut self, register: u8, data: u8) -> Result<(), I2C::Error> {
        let payload: [u8; 2] = [register, data];
        self.i2c.write(self.address, &payload).await
    }

    async fn write_data(&mut self, payload: &[u8]) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, payload).await
    }

    async fn read_register(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let mut temp_data = [0u8; 128];
        let mut data = [0u8; 2];
        let result = self.i2c.write_read(self.address, &[reg], &mut temp_data).await;
        for i in 0..data.len() {
            data[i] = temp_data[i+2];
        }
        match result{
            Ok(_)=>{
                Ok(0)
            }
            Err(i2c_error)=>{
                Err(i2c_error)
            }
        }
    }

    async fn read_data<'a>(&mut self, payload: &'a mut [u8]) -> Result<&'a [u8], I2C::Error> {
        let address = payload[0];
        let write_addresss = [address];
        let len = payload.len();
        let data = &mut payload[1..len];

        let total_len = data.len() + 2;
        let mut temp_buf = [0u8; 128]; // Temporary buffer to hold dummy bytes and data
        let data_result = self.i2c.write_read(self.address, &write_addresss, &mut temp_buf[..total_len]).await;
        // Copy data from temp_buf to data, skipping dummy bytes
        data.copy_from_slice(&temp_buf[2..total_len]);
        Ok(data)
    }

    async fn wait_for_data_ready(&mut self) -> Result<i8, I2C::Error> {
        let mut value_return = 0;
        let mut max_count = 0;
        for _ in 0..100 {
            let status = self.get_interrupt_status().await;
            match status{
                Ok(bool)=>{
                    return Ok(value_return)
                }
                Err(i2c_error)=>{
                    if max_count == 100{
                        return Err(i2c_error)
                    }                    
                    max_count = max_count + 1;
                }
            }
        }
        value_return = -1;
        return Ok(value_return)
    }
}
