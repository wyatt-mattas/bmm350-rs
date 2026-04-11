use crate::types::{Sensor3DData, Sensor3DDataScaled};

/// Standard gravity in m/s^2
pub const GRAVITY: f32 = 9.81;

impl Sensor3DData {
    /// Convert raw sensor data to scaled values
    ///
    /// # Arguments
    ///
    /// * `scale` - The full scale value in uT
    fn to_scaled(&self, scale: [f32; 4]) -> Sensor3DDataScaled {
        Sensor3DDataScaled {
            x: Self::lsb_to_scaled(self.x, scale[0]),
            y: Self::lsb_to_scaled(self.y, scale[1]),
            z: Self::lsb_to_scaled(self.z, scale[2]),
        }
    }

    /// Convert raw LSB value to scaled value
    ///
    /// # Arguments
    ///
    /// * `val` - Raw LSB value
    /// * `scale` - The full scale value
    /// * `half_scale` - Half of the full scale value
    fn lsb_to_scaled(val: i32, scale: f32) -> f32 {
        (scale * val as f32)
    }

    /// Convert raw magnetometer data to uT
    ///
    /// # Arguments
    ///
    /// * `scale` - The full scale value in uT
    pub fn to_ut(&self, scale: [f32; 4]) -> Sensor3DDataScaled {
        self.to_scaled(scale)
    }
}
