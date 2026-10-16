use crate::canvas::{self, Canvas};
use crate::ink::Ink;
use bevy::prelude::*;

pub const SIDE: usize = 28;
const BOX: usize = 20;
const MARGIN: i32 = ((SIDE - BOX) / 2) as i32;
const PEN_CEILING: f32 = 2.5;

pub struct Digit {
    values: [f32; SIDE * SIDE],
}

impl Digit {
    pub fn drawn(canvas: &Canvas) -> Option<Self> {
        let bounds = canvas.bounds()?;
        let scale = BOX as f32 / bounds.size().max_element();
        let pen = (canvas::PEN * scale).min(PEN_CEILING);
        let centre = bounds.center();
        let mut inner = Ink::new(BOX, BOX);
        let place = |point: Vec2| (point - centre) * scale + Vec2::splat(BOX as f32 / 2.0);
        for (from, to) in canvas.segments() {
            inner.stamp(place(from), place(to), pen);
        }
        let shift = (Vec2::splat(BOX as f32 / 2.0) - inner.centre()).round();
        let mut values = [0.0f32; SIDE * SIDE];
        for row in 0..BOX {
            for column in 0..BOX {
                let at = IVec2::new(column as i32, row as i32) + MARGIN + shift.as_ivec2();
                if (0..SIDE as i32).contains(&at.x) && (0..SIDE as i32).contains(&at.y) {
                    values[at.y as usize * SIDE + at.x as usize] = inner.value(column, row);
                }
            }
        }
        Some(Self { values })
    }

    pub fn values(&self) -> &[f32] {
        self.values.as_slice()
    }
}
