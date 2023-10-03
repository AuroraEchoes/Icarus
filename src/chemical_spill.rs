use ev3dev_lang_rust::Ev3Result;

use crate::{LineFollowRobot, Icarus};

impl LineFollowRobot {

    fn find_cans(&self) -> Ev3Result<Vec<i32>> { // Degrees from 0° at which any cans were found

        let mut detected_objects = Vec::<i32>::new();

        // Moving in 10° increments
        self.left_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.right_motor.set_speed_sp(self.parameters.targeted_speed)?;
        self.ultrasonic.set_mode_us_dist_cm()?;
        for rot in 0..36 {
            // Run
            self.left_motor.run_to_rel_pos(Some(1))?;
            self.right_motor.run_to_rel_pos(Some(-1))?;
            
            // Try and detect can
            if self.ultrasonic.get_distance_centimeters()? < 30. {
                detected_objects.push(rot);
            }
        }

        return Ok(detected_objects);
    }

    pub fn roh_tah_tey(&self) {
        self.left_motor.set_speed_sp(self.parameters.targeted_speed).unwrap();
        self.right_motor.set_speed_sp(self.parameters.targeted_speed).unwrap();
        self.left_motor.set_position(0).unwrap();
        self.right_motor.set_position(0).unwrap();
        loop {
            println!("Positions: L — {:?} / R — {:?}", self.left_motor.get_position(), self.right_motor.get_position());
            self.left_motor.run_to_rel_pos(Some(1)).unwrap();
            self.right_motor.run_to_rel_pos(Some(-1)).unwrap();
        }
    }

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