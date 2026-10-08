use crate::body::{Body, Part};
use crate::contact;
use crate::terrain::{self, Terrain};
use crate::world::GRAVITY;
use bevy::math::Vec2;

const TOUCH_ITERATIONS: u32 = 6;
const FADE: f32 = 0.15;
const BOUNCE: f32 = 0.3;
const SIDES: u32 = 12;
const RADIUS: f32 = 2.0 / terrain::SCALE;

#[derive(Default)]
pub struct Particles {
    bodies: Vec<Body>,
    ttl: Vec<f32>,
    contacts: Vec<contact::Contact>,
}

impl Particles {
    pub fn spawn(&mut self, at: Vec2, impulse: Vec2, density: f32) {
        let mut body = Body::at_origin(
            vec![Part::round(0, RADIUS, SIDES, density)],
            at,
            0.0,
            terrain::FRICTION,
            BOUNCE,
        );
        body.apply_impulse(impulse, at);
        self.bodies.push(body);
        self.ttl.push(1.0);
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
        self.ttl.clear();
    }

    pub fn step(&mut self, terrain: &Terrain, dt: f32) {
        for body in &mut self.bodies {
            body.integrate_velocity(dt, GRAVITY);
            contact::generate(std::slice::from_mut(body), terrain, &mut self.contacts);
            for _ in 0..TOUCH_ITERATIONS {
                for contact in &mut self.contacts {
                    contact::solve_velocity(body, contact);
                }
            }
            body.integrate_position(dt);
            for _ in 0..TOUCH_ITERATIONS {
                for contact in &mut self.contacts {
                    contact::solve_position(body, terrain, contact);
                }
            }
        }
    }

    pub fn fade(&mut self) {
        for ttl in &mut self.ttl {
            *ttl -= FADE;
        }
        let mut index = 0;
        while index < self.ttl.len() {
            if self.ttl[index] >= 0.0 {
                index += 1;
                continue;
            }
            self.bodies.remove(index);
            self.ttl.remove(index);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Body, f32)> {
        self.bodies.iter().zip(self.ttl.iter().copied())
    }
}
