use crate::env::{Action, OBSERVATION};

const ANGLE_REACH: f32 = 0.4;
const HOVER_GAIN: f32 = 0.55;
const ANGLE_GAIN: f32 = 0.5;
const ANGLE_DAMPING: f32 = 1.0;
const ALTITUDE_GAIN: f32 = 0.5;
const ALTITUDE_DAMPING: f32 = 0.5;
const THRESHOLD: f32 = 0.05;

pub fn pilot(state: &[f32; OBSERVATION]) -> Action {
    let angle_target = (state[0] * ANGLE_GAIN + state[2]).clamp(-ANGLE_REACH, ANGLE_REACH);
    let hover_target = HOVER_GAIN * state[0].abs();
    let mut angle_todo = (angle_target - state[4]) * ANGLE_GAIN - state[5] * ANGLE_DAMPING;
    let mut hover_todo = (hover_target - state[1]) * ALTITUDE_GAIN - state[3] * ALTITUDE_DAMPING;
    if state[6] != 0.0 || state[7] != 0.0 {
        angle_todo = 0.0;
        hover_todo = -state[3] * ALTITUDE_DAMPING;
    }
    if hover_todo > angle_todo.abs() && hover_todo > THRESHOLD {
        Action::Main
    } else if angle_todo < -THRESHOLD {
        Action::Right
    } else if angle_todo > THRESHOLD {
        Action::Left
    } else {
        Action::Coast
    }
}
