use std::{fmt::Display, thread::sleep, time::Duration, ops::{Add, Div}};

use ev3dev_lang_rust::{sensors::{ColorSensor, SensorPort}, Ev3Result, motors::{LargeMotor, MotorPort}};

pub struct LineFollowRobot {
    left_light: ColorSensor,
    right_light: ColorSensor,
    left_motor: LargeMotor,
    right_motor: LargeMotor,
    calibration: Option<CalibrationProfile>,
    parameters: LineFollowParameters,
}

impl LineFollowRobot {
    pub fn new(left_light: SensorPort, right_light: SensorPort, left_motor: MotorPort, right_motor: MotorPort, params: LineFollowParameters) -> Ev3Result<Self> {
        return Ok(Self { 
            left_light: ColorSensor::get(left_light)?, 
            right_light: ColorSensor::get(right_light)?, 
            left_motor: LargeMotor::get(left_motor)?, 
            right_motor: LargeMotor::get(right_motor)?,
            calibration: None, 
            parameters: params 
        });
    }
}

pub struct LineFollowParameters {
    green_thresh: i32,
    kp: f32,
    tick: u64, // In ms
    targeted_speed: i32 // [0, 1]; decimal
}

impl LineFollowParameters {
    pub fn new(green_thresh: i32, kp: f32, tick: u64, targeted_speed: i32) -> Self {
        return Self { green_thresh, kp, tick, targeted_speed }
    }
}

pub struct CalibrationProfile {
    left: RGB,
    right: RGB,
}

impl From<(RGB, RGB)> for CalibrationProfile {
    fn from(value: (RGB, RGB)) -> Self {
        return Self { left: value.0, right: value.1 };
    }
}

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
        sleep(Duration::from_secs(3));
        Icarus::info("Abort program to avert calibration".to_string());
        
        self.left_light.set_mode_rgb_raw()?;
        self.right_light.set_mode_rgb_raw()?;

        let mut left_rgb = RGB::from((0, 0, 0));
        let mut right_rgb = RGB::from((0, 0, 0));

        for _ in 0..100 {
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

    pub fn line_follow(&self) -> Ev3Result<()> {
        if let Some(profile) = &self.calibration {
            loop {
                let left_reading = RGB::from(self.left_light.get_rgb()?);
                let right_reading = RGB::from(self.right_light.get_rgb()?);
                let (left_reading, right_reading) = RGB::calibrated((left_reading, right_reading), profile);
                let heading = left_reading.average() - right_reading.average();

                let green_left = left_reading.g - left_reading.rb_ave() > self.parameters.green_thresh;
                let green_right = right_reading.g - right_reading.rb_ave() > self.parameters.green_thresh;

                if green_left {
                    Icarus::debug(format!("Green left {}", left_reading));
                }

                if green_right {
                    Icarus::debug(format!("Green left {}", left_reading));
                }

                let left_motor_speed = self.parameters.targeted_speed - (heading / 100) * self.parameters.targeted_speed;
                let right_motor_speed = self.parameters.targeted_speed + (heading / 100) * self.parameters.targeted_speed;

                self.left_motor.set_speed_sp(left_motor_speed)?;
                self.right_motor.set_speed_sp(right_motor_speed)?;
                self.left_motor.run_timed(Some(Duration::from_millis(self.parameters.tick))).unwrap();
                self.right_motor.run_timed(Some(Duration::from_millis(self.parameters.tick))).unwrap();

            }
        } else {
            Icarus::warn("Calibration is required before line follow can be executed".to_string());
        }

        Ok(())
    }
}

pub struct Icarus;

// The fact that terminal colour don't work for EV3 has possibly
// been the most devestating thing of all time for me :((

impl Icarus {
    fn info(message: String) {
        println!("{} [{}] » {}", "(?)", "ICARUS", message);
    }
    fn warn(message: String) {
        println!("{} [{}] » {}", "(!)", "ICARUS", message);
    }
    fn debug(message: String) {
        println!("{} [{}] » {}", "(>)", "ICARUS", message);
    }
}