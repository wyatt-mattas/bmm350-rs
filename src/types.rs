use core::fmt::Debug;

#[cfg(feature = "defmt")]
use defmt::Format;

/// Possible errors that can occur when interacting with the BMM350
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Error<E> {
    /// Communication error
    Comm(E),
    /// Invalid device (wrong chip ID)
    InvalidDevice,
    /// Invalid configuration
    InvalidConfig,
    /// Timeout error
    Timeout,

    ResetUnfinished,
}

/// Magnetometer power modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerMode {
    /// Suspend mode
    Suspend = 0x00,
    /// Normal mode
    Normal = 0x01,
    /// Forced mode
    Forced = 0x03,
    /// Forced mode fast
    ForcedFast = 0x04,
    FluxGuideReset = 0x05,
    BitReset = 0x07,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataRate {
    ODR400Hz = 0x02,
    ODR200Hz = 0x03,
    ODR100Hz = 0x04,
    ODR50Hz = 0x05,
    ODR25Hz = 0x06,
    ODR12_5Hz = 0x07,
    ODR6_25Hz = 0x08,
    ODR3_125Hz = 0x09,
    ODR1_5625Hz = 0x0A,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceMode {
    UltraLowPower = 0x00,
    LowPower = 0x01,
    Regular = 0x02,
    Enhanced = 0x03,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bandwidth {
    Normal = 0x00,
    High = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AverageNum {
    Avg1 = 0x00,
    Avg2 = 0x01,
    Avg4 = 0x02,
    Avg8 = 0x03,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisEnableDisable {
    Disable = 0x00,
    Enable = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptEnableDisable {
    Disable = 0x00,
    Enable = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptLatch {
    Pulsed = 0x00,
    Latched = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptPolarity {
    ActiveLow = 0x00,
    ActiveHigh = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptDrive {
    OpenDrain = 0x00,
    PushPull = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterruptMap {
    Unmap = 0x00,
    Map = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum I2cWdtEnable {
    Disable = 0x00,
    Enable = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum I2cWdtSelect {
    Short = 0x00,
    Long = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelfTestMode {
    Normal = 0x00,
    PositiveX = 0x01,
    NegativeX = 0x02,
    PositiveY = 0x03,
    NegativeY = 0x04,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CtrlUser {
    Disable = 0x00,
    Enable = 0x01,
}

pub struct MagCompensation {
    pub offset_x: i16,
    pub offset_y: i16,
    pub offset_z: i16,
}

impl Default for MagCompensation {
    fn default() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            offset_z: 0,
        }
    }
}

/// 3D sensor data (raw values)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sensor3DData {
    /// X-axis value
    pub x: i32,
    /// Y-axis value
    pub y: i32,
    /// Z-axis value
    pub z: i32,
}

/// Scaled 3D sensor data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sensor3DDataScaled {
    /// X-axis scaled value
    pub x: f32,
    /// Y-axis scaled value
    pub y: f32,
    /// Z-axis scaled value
    pub z: f32,
}

/// Scaled 3D sensor data
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmuCmdStatus0 {
    pub pmu_cmd_busy: u8,
    pub odr_overwrite: u8,
    pub avg_overwrite: u8,
    pub power_mode_is_normal: u8,
    pub cmd_is_illegal: u8,
    pub pmu_cmd_value: u8,
}
