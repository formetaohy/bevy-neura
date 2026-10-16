use crate::ink::Ink;
use bevy::prelude::*;

pub const SIDE: f32 = 300.0;
pub const PEN: f32 = 12.0;

pub fn placed(normal: Vec2) -> Vec2 {
    (normal + Vec2::splat(0.5)) * SIDE
}

pub struct Canvas {
    ink: Ink,
    strokes: Vec<Vec<Vec2>>,
    drawing: bool,
    revision: u32,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            ink: Ink::new(SIDE as usize, SIDE as usize),
            strokes: Vec::new(),
            drawing: false,
            revision: 0,
        }
    }

    pub fn press(&mut self, at: Vec2) {
        self.strokes.push(vec![at]);
        self.drawing = true;
        self.ink.stamp(at, at, PEN);
        self.revision += 1;
    }

    pub fn drag(&mut self, at: Vec2) {
        if !self.drawing {
            return;
        }
        let stroke = self
            .strokes
            .last_mut()
            .expect("a pen that draws extends the stroke it pressed");
        let from = *stroke
            .last()
            .expect("a pen that draws extends the stroke it pressed");
        if from == at {
            return;
        }
        stroke.push(at);
        self.ink.stamp(from, at, PEN);
        self.revision += 1;
    }

    pub fn lift(&mut self) {
        self.drawing = false;
    }

    pub fn clear(&mut self) {
        self.strokes.clear();
        self.ink.clear();
        self.drawing = false;
        self.revision += 1;
    }

    pub fn drawing(&self) -> bool {
        self.drawing
    }

    pub fn drawn(&self) -> bool {
        !self.strokes.is_empty()
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }

    pub fn ink(&self) -> &Ink {
        &self.ink
    }

    pub fn bounds(&self) -> Option<Rect> {
        let mut low = Vec2::splat(f32::MAX);
        let mut high = Vec2::splat(f32::MIN);
        for stroke in &self.strokes {
            for point in stroke {
                low = low.min(*point);
                high = high.max(*point);
            }
        }
        if low.x > high.x {
            return None;
        }
        Some(Rect::from_corners(
            low - Vec2::splat(PEN),
            high + Vec2::splat(PEN),
        ))
    }

    pub fn segments(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        self.strokes.iter().flat_map(|stroke| {
            stroke
                .windows(2)
                .map(|pair| (pair[0], pair[1]))
                .chain(stroke.first().map(|point| (*point, *point)))
        })
    }
}
