pub mod line_follow;

extern crate ev3dev_lang_rust;

use ev3dev_lang_rust::motors::MotorPort;
use ev3dev_lang_rust::Ev3Result;
use ev3dev_lang_rust::sensors::SensorPort;
use line_follow::{LineFollowRobot, LineFollowParameters};

fn main() -> Ev3Result<()> {

    let mut robot = LineFollowRobot::new(
        SensorPort::In1, 
        SensorPort::In2,
        MotorPort::OutA,
        MotorPort::OutB,
        LineFollowParameters::new(20, 1., 100, 200)
    )?;
    robot.calibrate()?;
    robot.line_follow()?;


    Ok(())
}
