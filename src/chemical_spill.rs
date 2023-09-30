use ev3dev_lang_rust::Ev3Result;

use crate::{LineFollowRobot, Icarus};

impl LineFollowRobot {
    pub fn chemical_spill(&self) -> Ev3Result<()> {

        Icarus::info("Entering chemical spill".to_string());

        // Put your hands in the air like you just don't care
        self.claw_vert.set_speed_sp(200)?;
        self.claw_vert.set_position_sp((0.25 * self.claw_vert.get_count_per_rot()? as f32) as i32)?;
        self.claw_vert.run_to_rel_pos(None)?;


        // Move into the center
        self.left_motor.set_speed_sp(400)?;
        self.right_motor.set_speed_sp(400)?;
        self.left_motor.set_time_sp(2000)?;
        self.right_motor.set_time_sp(2000)?;
        self.left_motor.run_timed(None)?;
        self.right_motor.run_timed(None)?;

        // Spin
        self.ultrasonic.set_mode_us_dist_cm()?;
        let mut spotted_can = false;
        let mut spin_count = 0;
        while !spotted_can {
            let dist = self.ultrasonic.get_distance_centimeters()?;
            if dist < 20. {
                spotted_can = true;
            }
            
            // Pirouette slightly
            self.left_motor.set_speed_sp(30)?;
            self.right_motor.set_speed_sp(-30)?;
            self.left_motor.set_time_sp(100)?;
            self.right_motor.set_time_sp(100)?;
            self.left_motor.run_timed(None)?;
            self.right_motor.run_timed(None)?;

            Icarus::debug(format!("DIST: {:?}, sc: {:?}", dist, spin_count));
            spin_count += 1;
        }

        return Ok(());
    }
}