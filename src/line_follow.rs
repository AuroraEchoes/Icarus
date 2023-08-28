use std::{fmt::Display, thread::sleep, time::Duration, ops::{Add, Div}};

use ev3dev_lang_rust::Ev3Result;

use crate::{LineFollowRobot, Icarus};

pub struct LineFollowParameters {
    kp: f32,
    tick: u64, // In ms
    targeted_speed: i32 // [0, 1]; decimal
}

impl LineFollowParameters {
    pub fn new(kp: f32, tick: u64, targeted_speed: i32) -> Self {
        return Self { kp, tick, targeted_speed }
    }
}

#[derive(Clone)]
pub struct CalibrationProfile {
    left: RGB,
    right: RGB,
}

impl From<(RGB, RGB)> for CalibrationProfile {
    fn from(value: (RGB, RGB)) -> Self {
        return Self { left: value.0, right: value.1 };
    }
}

#[derive(Clone)]
pub struct RGB {
    r: i32,
    g: i32,
    b: i32,
}

impl RGB {
    pub fn average(&self) -> i32 {
        return (self.r + self.g + self.b) / 3;
    }

    fn calibrated(mut readings: (Self, Self), profile: &CalibrationProfile) -> (Self, Self) {
        readings.0.r -= profile.left.r;
        readings.0.g -= profile.left.g;
        readings.0.b -= profile.left.b;
        readings.1.r -= profile.left.r;
        readings.1.g -= profile.left.g;
        readings.1.b -= profile.left.b;

        return readings;
    }

    fn rb_ave(&self) -> i32 {
        return (self.r + self.b) / 2;
    }

    fn reflectivity(&self) -> f32 {
        return 0.2125 * self.r as f32 + 0.7154 * self.g as f32 + 0.0721 * self.b as f32;
    }
}

impl From<(i32, i32, i32)> for RGB {
    fn from(value: (i32, i32, i32)) -> Self {
        return Self { r: value.0, g: value.1, b: value.2 };
    }
}

impl Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str(format!("R: {}, G: {}, B: {}", self.r.to_string(), self.g.to_string(), self.b.to_string()).as_str());
    }
}

impl Add for RGB {
    type Output = RGB;

    fn add(self, rhs: Self) -> Self::Output {
        return Self { r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b };
    }
} 

impl Div for RGB {
    type Output = RGB;

    fn div(self, rhs: Self) -> Self::Output {
        return Self { r: self.r / rhs.r, g: self.g / rhs.g, b: self.b / rhs.b };
    }
}

impl LineFollowRobot {
    pub fn calibrate(&mut self) -> Ev3Result<()> {
        Icarus::info("Calibrating in 3 seconds".to_string());
        Icarus::info("Abort program to avert calibration".to_string());
        self.left_light.set_mode_rgb_raw()?;
        self.right_light.set_mode_rgb_raw()?;
        
        sleep(Duration::from_secs(3));

        let mut left_rgb = RGB::from((0, 0, 0));
        let mut right_rgb = RGB::from((0, 0, 0));

        for _ in 0..100 {
            sleep(Duration::from_millis(10));
            left_rgb = left_rgb.add(RGB::from(self.left_light.get_rgb()?));
            right_rgb = right_rgb.add(RGB::from(self.right_light.get_rgb()?));
        }

        left_rgb = left_rgb.div(RGB::from((100, 100, 100)));
        right_rgb = right_rgb.div(RGB::from((100, 100, 100)));

        let calibration = CalibrationProfile::from((left_rgb, right_rgb));
        Icarus::info(format!("Calibration completed! Left: {}, Right: {}", calibration.left, calibration.right));
        self.calibration = Some(calibration);

        Ok(())  
    }

    pub fn line_follow(&mut self) -> Ev3Result<()> {
        self.ultrasonic.set_mode_us_dist_cm()?;
        if let Some(profile) = &self.calibration.clone() {
            let mut green_timeout = 0;
            loop {

                // Water tower
                // Ideally we want to make a _/‾‾‾‾\_ shape
                let ultrasonic_reading = self.ultrasonic.get_distance_centimeters()?;
                if ultrasonic_reading < 25. {
                    Icarus::info("Avoiding water tower".to_string());
                    self.avoid_water_tower()?;

                }


                let left_reading = RGB::from(self.left_light.get_rgb()?);
                let right_reading = RGB::from(self.right_light.get_rgb()?);

                let green_left = left_reading.g as f32 > 1.65 * left_reading.rb_ave() as f32;
                let green_right = right_reading.g as f32 > 1.65 * right_reading.rb_ave() as f32;

                if green_left && green_right {
                    self.chemical_spill()?;
                }

                if green_left && green_timeout > 100 {
                    Icarus::info(format!("Detected green turn on the left"));

                    // Stop
                    self.left_motor.stop()?;
                    self.right_motor.stop()?;

                    // Bump
                    self.left_motor.set_position_sp((self.left_motor.get_count_per_rot()? as f32 * 0.3) as i32)?;
                    self.right_motor.set_position_sp((self.right_motor.get_count_per_rot()? as f32 * 0.3) as i32)?;
                    self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
                    self.right_motor.set_speed_sp(self.parameters.targeted_speed)?;
                    self.left_motor.run_to_rel_pos(None)?;
                    self.right_motor.run_to_rel_pos(None)?;
                    #[cfg(target_os = "linux")]
                    self.right_motor.wait_until_not_moving(None);
                    #[cfg(target_os = "linux")]
                    self.left_motor.wait_until_not_moving(None);

                    // Turn
                    self.left_motor.set_position_sp(-(self.left_motor.get_count_per_rot()? as f32 * 0.5) as i32)?;
                    self.right_motor.set_position_sp((self.right_motor.get_count_per_rot()? as f32 * 0.5) as i32)?;
                    self.left_motor.set_speed_sp(-self.parameters.targeted_speed)?;
                    self.right_motor.set_speed_sp(self.parameters.targeted_speed)?;
                    self.left_motor.run_to_rel_pos(None)?;
                    self.right_motor.run_to_rel_pos(None)?;
                    #[cfg(target_os = "linux")]
                    self.right_motor.wait_until_not_moving(None);
                    #[cfg(target_os = "linux")]
                    self.left_motor.wait_until_not_moving(None);
                    green_timeout = 0;
                }

                if green_right && green_timeout > 100 {
                    Icarus::info(format!("Detected green turn on the right"));
                    
                    // Stop
                    self.left_motor.stop()?;
                    self.right_motor.stop()?;
                    
                    // Bump
                    self.left_motor.set_position_sp((self.left_motor.get_count_per_rot()? as f32 * 0.3) as i32)?;
                    self.right_motor.set_position_sp((self.right_motor.get_count_per_rot()? as f32 * 0.3) as i32)?;
                    self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
                    self.right_motor.set_speed_sp(self.parameters.targeted_speed)?;
                    self.left_motor.run_to_rel_pos(None)?;
                    self.right_motor.run_to_rel_pos(None)?;
                    #[cfg(target_os = "linux")]
                    self.right_motor.wait_until_not_moving(None);
                    #[cfg(target_os = "linux")]
                    self.left_motor.wait_until_not_moving(None);

                    // Turn
                    self.left_motor.set_position_sp((self.left_motor.get_count_per_rot()? as f32 * 0.5) as i32)?;
                    self.right_motor.set_position_sp(-(self.right_motor.get_count_per_rot()? as f32 * 0.5) as i32)?;
                    self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
                    self.right_motor.set_speed_sp(-self.parameters.targeted_speed)?;
                    self.left_motor.run_to_rel_pos(None)?;
                    self.right_motor.run_to_rel_pos(None)?;
                    #[cfg(target_os = "linux")]
                    self.right_motor.wait_until_not_moving(None);
                    #[cfg(target_os = "linux")]
                    self.left_motor.wait_until_not_moving(None);
                    green_timeout = 0;
                }

                let (left_reading, right_reading) = RGB::calibrated((left_reading, right_reading), profile);
                let heading = left_reading.reflectivity() - right_reading.reflectivity();
                
                // let left_motor_speed = self.parameters.targeted_speed - (heading / 100.) as i32 * self.parameters.targeted_speed;
                // let right_motor_speed = self.parameters.targeted_speed + (heading / 100.) as i32 * self.parameters.targeted_speed;

                let mut left_motor_speed = 
                    (self.parameters.targeted_speed as f32) + 
                    (
                        (self.parameters.kp * heading / 100.) * 
                        (self.parameters.targeted_speed as f32)
                    );

                let mut right_motor_speed = 
                    (self.parameters.targeted_speed as f32) - 
                    (
                        (self.parameters.kp * heading / 300.) * 
                        (self.parameters.targeted_speed as f32)
                    );

                // Guard for max speed
                if left_motor_speed.abs() >= 800. {
                    left_motor_speed = left_motor_speed.signum() * 800.;
                }
                if right_motor_speed.abs() >= 800. {
                    right_motor_speed = right_motor_speed.signum() * 800.;
                }

                self.left_motor.set_speed_sp(left_motor_speed as i32)?;
                self.right_motor.set_speed_sp(right_motor_speed as i32)?; 
                self.left_motor.run_timed(Some(Duration::from_millis(self.parameters.tick))).unwrap();
                self.right_motor.run_timed(Some(Duration::from_millis(self.parameters.tick))).unwrap();
                
                green_timeout += 1;
                
            }
        } else {
            Icarus::warn("Calibration is required before line follow can be executed".to_string());
        }

        Ok(())
    }

    pub fn avoid_water_tower(&self) -> Ev3Result<()>{
        let pivot_rotations = 0.3;
        let short_rotations = 1.8;
        let long_rotations = 1.8;

        
        // Turn [_ -> /]
        self.left_motor.set_position_sp(-(self.left_motor.get_count_per_rot()? as f32 * pivot_rotations) as i32)?;
        self.right_motor.set_position_sp((self.right_motor.get_count_per_rot()? as f32 * pivot_rotations) as i32)?;
        self.left_motor.set_speed_sp(-self.parameters.targeted_speed)?;
        self.right_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.left_motor.run_to_rel_pos(None)?;
        self.right_motor.run_to_rel_pos(None)?;
        #[cfg(target_os = "linux")]
        self.right_motor.wait_until_not_moving(None);
        #[cfg(target_os = "linux")]
        self.left_motor.wait_until_not_moving(None);

        // Move [/]
        self.left_motor.set_position_sp((self.left_motor.get_count_per_rot()? as f32 * short_rotations) as i32)?;
        self.right_motor.set_position_sp((self.right_motor.get_count_per_rot()? as f32 * short_rotations) as i32)?;
        self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.right_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.left_motor.run_to_rel_pos(None)?;
        self.right_motor.run_to_rel_pos(None)?;
        #[cfg(target_os = "linux")]
        self.right_motor.wait_until_not_moving(None);
        #[cfg(target_os = "linux")]
        self.left_motor.wait_until_not_moving(None);

        // Turn [/ -> ‾]
        self.left_motor.set_position_sp((self.left_motor.get_count_per_rot()? as f32 * pivot_rotations) as i32)?;
        self.right_motor.set_position_sp(-(self.right_motor.get_count_per_rot()? as f32 * pivot_rotations) as i32)?;
        self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.right_motor.set_speed_sp(-self.parameters.targeted_speed)?;
        self.left_motor.run_to_rel_pos(None)?;
        self.right_motor.run_to_rel_pos(None)?;
        #[cfg(target_os = "linux")]
        self.right_motor.wait_until_not_moving(None);
        #[cfg(target_os = "linux")]
        self.left_motor.wait_until_not_moving(None);

        // Move [‾‾‾‾]
        self.left_motor.set_position_sp((self.left_motor.get_count_per_rot()? as f32 * long_rotations) as i32)?;
        self.right_motor.set_position_sp((self.right_motor.get_count_per_rot()? as f32 * long_rotations) as i32)?;
        self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.right_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.left_motor.run_to_rel_pos(None)?;
        self.right_motor.run_to_rel_pos(None)?;
        #[cfg(target_os = "linux")]
        self.right_motor.wait_until_not_moving(None);
        #[cfg(target_os = "linux")]
        self.left_motor.wait_until_not_moving(None);

        // Turn [‾ -> \]
        self.left_motor.set_position_sp((self.left_motor.get_count_per_rot()? as f32 * pivot_rotations) as i32)?;
        self.right_motor.set_position_sp(-(self.right_motor.get_count_per_rot()? as f32 * pivot_rotations) as i32)?;
        self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.right_motor.set_speed_sp(-self.parameters.targeted_speed)?;
        self.left_motor.run_to_rel_pos(None)?;
        self.right_motor.run_to_rel_pos(None)?;
        #[cfg(target_os = "linux")]
        self.right_motor.wait_until_not_moving(None);
        #[cfg(target_os = "linux")]
        self.left_motor.wait_until_not_moving(None);

        Icarus::info("Returning to line follow".to_string());
        return Ok(());
    }
}