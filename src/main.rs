pub mod line_follow;
pub mod chemical_spill;

extern crate ev3dev_lang_rust;

use ev3dev_lang_rust::motors::{LargeMotor, MediumMotor};
use ev3dev_lang_rust::{motors::MotorPort, sensors::ColorSensor};
use ev3dev_lang_rust::Ev3Result;
use ev3dev_lang_rust::sensors::{SensorPort, UltrasonicSensor};
use line_follow::{LineFollowParameters, CalibrationProfile};

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

pub struct LineFollowRobot {
    pub left_light: ColorSensor,
    pub right_light: ColorSensor,
    pub ultrasonic: UltrasonicSensor,
    pub left_motor: LargeMotor,
    pub right_motor: LargeMotor,
    pub claw_vert: LargeMotor,
    pub claw_horiz: MediumMotor,
    pub calibration: Option<CalibrationProfile>,
    pub parameters: LineFollowParameters,
}

impl LineFollowRobot {
    pub fn new(left_light: SensorPort, right_light: SensorPort, ultrasonic: SensorPort, left_motor: MotorPort, right_motor: MotorPort, claw_vert: MotorPort, claw_horiz: MotorPort, params: LineFollowParameters) -> Ev3Result<Self> {
        return Ok(Self { 
            left_light: ColorSensor::get(left_light)?, 
            right_light: ColorSensor::get(right_light)?, 
            ultrasonic: UltrasonicSensor::get(ultrasonic)?,
            left_motor: LargeMotor::get(left_motor)?, 
            right_motor: LargeMotor::get(right_motor)?,
            claw_vert: LargeMotor::get(claw_vert)?,
            claw_horiz: MediumMotor::get(claw_horiz)?,
            calibration: None, 
            parameters: params 
        });
    }
}

fn main() -> Ev3Result<()> {

    let mut robot = LineFollowRobot::new(
        SensorPort::In1, 
        SensorPort::In2,
        SensorPort::In3,
        MotorPort::OutA,
        MotorPort::OutB,
        MotorPort::OutC,
        MotorPort::OutD,
        LineFollowParameters::new(3., 50, 100, 1.7)
    )?; 
    robot.calibrate()?;
    robot.line_follow()?;

    Ok(())
}
