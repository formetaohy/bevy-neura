use bevy::math::Vec2;

#[derive(Clone)]
pub struct Polygon {
    vertices: Vec<Vec2>,
}

#[derive(Clone, Copy, Debug)]
pub struct MassData {
    pub mass: f32,
    pub center: Vec2,
    pub inertia: f32,
}

impl Polygon {
    pub fn new(vertices: Vec<Vec2>) -> Self {
        assert!(
            vertices.len() >= 3,
            "a polygon of {} vertices holds no area",
            vertices.len(),
        );
        let area = signed_area(&vertices);
        assert!(
            area.abs() > 1e-6,
            "a polygon of {vertices:?} encloses no area",
        );
        let mut polygon = Self { vertices };
        if area < 0.0 {
            polygon.vertices.reverse();
        }
        polygon
    }

    pub fn boxed(half_extents: Vec2) -> Self {
        let (x, y) = (half_extents.x, half_extents.y);
        Self::new(vec![
            Vec2::new(-x, -y),
            Vec2::new(x, -y),
            Vec2::new(x, y),
            Vec2::new(-x, y),
        ])
    }

    pub fn circle(radius: f32, sides: u32) -> Self {
        assert!(sides >= 8, "a circle of {sides} sides is not round");
        let step = std::f32::consts::TAU / sides as f32;
        Self::new(
            (0..sides)
                .map(|side| radius * Vec2::from_angle(step * side as f32))
                .collect(),
        )
    }

    pub fn rotated(self, angle: f32) -> Self {
        Self {
            vertices: self
                .vertices
                .iter()
                .map(|vertex| Vec2::from_angle(angle).rotate(*vertex))
                .collect(),
        }
    }

    pub fn shifted(self, offset: Vec2) -> Self {
        Self {
            vertices: self
                .vertices
                .iter()
                .map(|vertex| *vertex + offset)
                .collect(),
        }
    }

    pub fn vertices(&self) -> &[Vec2] {
        &self.vertices
    }
    pub fn mass_data(&self, density: f32) -> MassData {
        const THIRD: f32 = 1.0 / 3.0;
        let count = self.vertices.len();
        let (mut area, mut center, mut second) = (0.0f32, Vec2::ZERO, 0.0f32);
        for index in 0..count {
            let first = self.vertices[index];
            let next = self.vertices[(index + 1) % count];
            let cross = first.perp_dot(next);
            let triangle = 0.5 * cross;
            area += triangle;
            center += triangle * THIRD * (first + next);
            let vertical = first.x * first.x + next.x * first.x + next.x * next.x;
            let horizontal = first.y * first.y + next.y * first.y + next.y * next.y;
            second += 0.25 * THIRD * cross * (vertical + horizontal);
        }
        MassData {
            mass: density * area,
            center: center / area,
            inertia: density * second,
        }
    }
}

fn signed_area(vertices: &[Vec2]) -> f32 {
    let mut area = 0.0;
    for index in 0..vertices.len() {
        area += vertices[index].perp_dot(vertices[(index + 1) % vertices.len()]);
    }
    0.5 * area
}
