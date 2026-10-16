use bevy::prelude::*;

pub struct Ink {
    width: usize,
    height: usize,
    values: Vec<f32>,
}

impl Ink {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "a raster of {width} by {height} holds no pixel",
        );
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn value(&self, column: usize, row: usize) -> f32 {
        assert!(
            column < self.width && row < self.height,
            "the pixel {column} by {row} lies beside a raster of {} by {}",
            self.width,
            self.height,
        );
        self.values[row * self.width + column]
    }

    pub fn clear(&mut self) {
        self.values.fill(0.0);
    }

    pub fn mass(&self) -> f32 {
        self.values.iter().sum()
    }

    pub fn centre(&self) -> Vec2 {
        let mass = self.mass();
        assert!(mass > 0.0, "a blank raster weighs nothing to centre on");
        let weighted = self
            .values
            .iter()
            .enumerate()
            .fold(Vec2::ZERO, |sum, (at, value)| {
                let (column, row) = (at % self.width, at / self.width);
                sum + Vec2::new(column as f32 + 0.5, row as f32 + 0.5) * *value
            });
        weighted / mass
    }

    pub fn stamp(&mut self, from: Vec2, to: Vec2, radius: f32) {
        assert!(
            radius > 0.0 && radius.is_finite(),
            "a pen of radius {radius} draws no stroke",
        );
        let low = from.min(to) - Vec2::splat(radius + 1.0);
        let high = from.max(to) + Vec2::splat(radius + 1.0);
        let columns = span(low.x, high.x, self.width);
        let rows = span(low.y, high.y, self.height);
        for row in rows {
            for column in columns.clone() {
                let centre = Vec2::new(column as f32 + 0.5, row as f32 + 0.5);
                let coverage = (radius + 0.5 - reach(centre, from, to)).clamp(0.0, 1.0);
                let held = &mut self.values[row * self.width + column];
                *held = held.max(coverage);
            }
        }
    }
}

fn span(low: f32, high: f32, bound: usize) -> std::ops::Range<usize> {
    let start = low.floor().max(0.0) as usize;
    let end = (high.ceil().max(0.0) as usize).min(bound);
    start.min(end)..end
}

fn reach(point: Vec2, from: Vec2, to: Vec2) -> f32 {
    let segment = to - from;
    let length = segment.length_squared();
    if length <= f32::EPSILON {
        return point.distance(from);
    }
    let along = ((point - from).dot(segment) / length).clamp(0.0, 1.0);
    point.distance(from + segment * along)
}
