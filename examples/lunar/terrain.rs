use crate::random::Random;
use bevy::math::Vec2;

pub const WIDTH: f32 = 20.0;
pub const HEIGHT: f32 = 40.0 / 3.0;
pub const SCALE: f32 = 30.0;
pub const CHUNKS: usize = 11;
pub const PAD_Y: f32 = HEIGHT / 4.0;
pub const PAD_X1: f32 = WIDTH / (CHUNKS - 1) as f32 * 4.0;
pub const PAD_X2: f32 = WIDTH / (CHUNKS - 1) as f32 * 6.0;
pub const FRICTION: f32 = 0.1;
const SPAN: f32 = WIDTH / (CHUNKS - 1) as f32;
const SMOOTHING: f32 = 0.33;

#[derive(Clone, Copy, Debug)]
pub struct Closest {
    pub point: Vec2,
    pub normal: Vec2,
    pub separation: f32,
    pub edge: usize,
}

#[derive(Clone)]
pub struct Terrain {
    points: Vec<Vec2>,
    normals: Vec<Vec2>,
}

impl Terrain {
    pub fn generated(random: &mut Random) -> Self {
        let mut heights = [0.0f32; CHUNKS + 1];
        for height in &mut heights {
            *height = random.uniform(0.0, HEIGHT / 2.0);
        }
        for height in &mut heights[CHUNKS / 2 - 2..CHUNKS / 2 + 3] {
            *height = PAD_Y;
        }
        let points = (0..CHUNKS)
            .map(|chunk| {
                let previous = heights[(chunk + CHUNKS) % (CHUNKS + 1)];
                let smooth = SMOOTHING * (previous + heights[chunk] + heights[chunk + 1]);
                Vec2::new(SPAN * chunk as f32, smooth)
            })
            .collect::<Vec<Vec2>>();
        let normals = points
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).perp().normalize())
            .collect();
        Self { points, normals }
    }

    pub fn points(&self) -> &[Vec2] {
        &self.points
    }
    pub fn closest(&self, point: Vec2) -> Closest {
        let span = WIDTH / (CHUNKS - 1) as f32;
        let edge = ((point.x / span) as usize).min(CHUNKS - 2);
        let start = self.points[edge];
        let end = self.points[edge + 1];
        let segment = end - start;
        let along = ((point.x - start.x) / segment.x).clamp(0.0, 1.0);
        let anchor = start + along * segment;
        let normal = self.normals[edge];
        Closest {
            point: anchor,
            normal,
            separation: (point - anchor).dot(normal),
            edge,
        }
    }
}
