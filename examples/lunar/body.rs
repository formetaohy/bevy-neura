use crate::shape::{MassData, Polygon};
use bevy::math::{Rot2, Vec2};

pub const LINEAR_SLOP: f32 = 0.005;
pub const BAUMGARTE: f32 = 0.2;
pub const MAX_LINEAR_CORRECTION: f32 = 0.2;
pub const VELOCITY_THRESHOLD: f32 = 1.0;
pub const LINEAR_SLEEP_TOLERANCE: f32 = 0.01;
pub const ANGULAR_SLEEP_TOLERANCE: f32 = 2.0f32.to_radians();
pub const SLEEP_TIME: f32 = 0.5;

pub const HULL: u8 = 0;
pub const LEFT_LEG: u8 = 1;
pub const RIGHT_LEG: u8 = 2;

#[derive(Clone)]
pub struct Part {
    pub tag: u8,
    pub polygon: Polygon,
    pub density: f32,
}

impl Part {
    pub fn hull(polygon: Polygon, density: f32) -> Self {
        Self {
            tag: HULL,
            polygon,
            density,
        }
    }

    pub fn leg(tag: u8, polygon: Polygon, density: f32) -> Self {
        assert!(
            tag == LEFT_LEG || tag == RIGHT_LEG,
            "a lander holds two legs, not the gear {tag}",
        );
        Self {
            tag,
            polygon,
            density,
        }
    }

    pub fn round(tag: u8, radius: f32, sides: u32, density: f32) -> Self {
        Self {
            tag,
            polygon: Polygon::circle(radius, sides),
            density,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Touch {
    pub part: u8,
    pub edge: usize,
    pub normal: f32,
    pub tangent: f32,
    pub touching: bool,
}

pub struct Body {
    parts: Vec<Part>,
    pub inverse_mass: f32,
    pub local_center: Vec2,
    pub inverse_inertia: f32,
    pub center: Vec2,
    pub angle: f32,
    pub linear: Vec2,
    pub angular: f32,
    pub friction: f32,
    pub restitution: f32,
    pub force: Vec2,
    pub torque: f32,
    pub awake: bool,
    pub quiet: f32,
    touches: Vec<Touch>,
}

impl Body {
    pub fn at_origin(
        parts: Vec<Part>,
        origin: Vec2,
        angle: f32,
        friction: f32,
        restitution: f32,
    ) -> Self {
        assert!(!parts.is_empty(), "a body of no part holds no mass");
        let mass = parts
            .iter()
            .map(|part| part.polygon.mass_data(part.density))
            .fold(
                MassData {
                    mass: 0.0,
                    center: Vec2::ZERO,
                    inertia: 0.0,
                },
                |total, part| MassData {
                    mass: total.mass + part.mass,
                    center: total.center + part.mass * part.center,
                    inertia: total.inertia + part.inertia,
                },
            );
        let center = mass.center / mass.mass;
        let inertia = mass.inertia - mass.mass * center.length_squared();
        assert!(
            mass.mass > 0.0 && inertia > 0.0,
            "a body of mass {} and inertia {inertia} moves like nothing",
            mass.mass,
        );
        let mut body = Self {
            parts,
            inverse_mass: 1.0 / mass.mass,
            local_center: center,
            inverse_inertia: 1.0 / inertia,
            center: origin + Rot2::radians(angle) * center,
            angle,
            linear: Vec2::ZERO,
            angular: 0.0,
            friction,
            restitution,
            force: Vec2::ZERO,
            torque: 0.0,
            awake: true,
            quiet: 0.0,
            touches: Vec::new(),
        };
        body.touches = body.probes().collect::<Vec<Touch>>();
        body
    }

    pub fn part(&self, tag: u8) -> &Part {
        self.parts
            .iter()
            .find(|part| part.tag == tag)
            .unwrap_or_else(|| panic!("a body holds no part under the tag {tag}"))
    }

    pub fn probe_count(&self) -> usize {
        self.touches.len()
    }

    fn probes(&self) -> impl Iterator<Item = Touch> + '_ {
        self.parts.iter().flat_map(|part| {
            part.polygon.vertices().iter().map(move |_| Touch {
                part: part.tag,
                ..Touch::default()
            })
        })
    }

    fn probe(&self, index: usize) -> Vec2 {
        let mut at = index;
        for part in &self.parts {
            let count = part.polygon.vertices().len();
            if at < count {
                return part.polygon.vertices()[at];
            }
            at -= count;
        }
        panic!(
            "a body of {} probes holds no probe {index}",
            self.probe_count()
        )
    }

    pub fn touch(&self, index: usize) -> Touch {
        self.touches[index]
    }

    pub fn part_touching(&self, tag: u8) -> bool {
        self.touches
            .iter()
            .any(|touch| touch.touching && touch.part == tag)
    }
    pub fn touched(&mut self, index: usize, touch: Touch) {
        self.touches[index] = touch;
    }

    pub fn clear_touches(&mut self) {
        for touch in &mut self.touches {
            touch.touching = false;
        }
    }
    pub fn origin(&self) -> Vec2 {
        self.center - Rot2::radians(self.angle) * self.local_center
    }

    pub fn world_point(&self, local: Vec2) -> Vec2 {
        self.center + Rot2::radians(self.angle) * (local - self.local_center)
    }

    pub fn velocity_at(&self, point: Vec2) -> Vec2 {
        self.linear + self.angular * (point - self.center).perp()
    }

    pub fn apply_impulse(&mut self, impulse: Vec2, point: Vec2) {
        self.wake();
        self.apply_solver_impulse(impulse, point);
    }

    pub fn apply_solver_impulse(&mut self, impulse: Vec2, point: Vec2) {
        self.linear += impulse * self.inverse_mass;
        self.angular += self.inverse_inertia * (point - self.center).perp_dot(impulse);
    }

    pub fn apply_force_to_center(&mut self, force: Vec2) {
        self.wake();
        self.force += force;
    }

    pub fn integrate_velocity(&mut self, dt: f32, gravity: Vec2) {
        if !self.awake {
            return;
        }
        self.linear += dt * (gravity + self.force * self.inverse_mass);
        self.angular += dt * self.torque * self.inverse_inertia;
        self.force = Vec2::ZERO;
        self.torque = 0.0;
    }

    pub fn integrate_position(&mut self, dt: f32) {
        if !self.awake {
            return;
        }
        self.center += dt * self.linear;
        self.angle += dt * self.angular;
    }

    pub fn wake(&mut self) {
        self.awake = true;
        self.quiet = 0.0;
    }

    pub fn settle(&mut self, dt: f32) {
        if !self.awake {
            return;
        }
        if self.linear.length() < LINEAR_SLEEP_TOLERANCE
            && self.angular.abs() < ANGULAR_SLEEP_TOLERANCE
        {
            self.quiet += dt;
        } else {
            self.quiet = 0.0;
        }
    }

    pub fn asleep(&self) -> bool {
        self.quiet >= SLEEP_TIME
    }

    pub fn sleep(&mut self) {
        self.awake = false;
        self.linear = Vec2::ZERO;
        self.angular = 0.0;
    }

    pub fn probe_world(&self, index: usize) -> Vec2 {
        self.world_point(self.probe(index))
    }
}
