use crate::body::Body;
use crate::contact::{self, Contact};
use crate::terrain::Terrain;
use bevy::math::Vec2;

pub const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const SUBSTEPS: u32 = 2;
const VELOCITY_ITERATIONS: u32 = 40;
const POSITION_ITERATIONS: u32 = 20;

pub struct World {
    pub bodies: Vec<Body>,
    pub terrain: Terrain,
    contacts: Vec<Contact>,
}

impl World {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            bodies: Vec::new(),
            terrain,
            contacts: Vec::new(),
        }
    }

    pub fn add(&mut self, body: Body) -> usize {
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    pub fn step(&mut self, dt: f32) {
        for _ in 0..SUBSTEPS {
            self.substep(dt / SUBSTEPS as f32);
        }
        for body in &mut self.bodies {
            body.settle(dt);
        }
        if self.bodies.iter().all(|body| body.asleep()) {
            for body in &mut self.bodies {
                body.sleep();
            }
        }
    }

    fn substep(&mut self, dt: f32) {
        for body in &mut self.bodies {
            body.integrate_velocity(dt, GRAVITY);
        }
        contact::generate(&mut self.bodies, &self.terrain, &mut self.contacts);
        for _ in 0..VELOCITY_ITERATIONS {
            for contact in &mut self.contacts {
                contact::solve_velocity(&mut self.bodies[contact.body], contact);
            }
        }
        contact::store_touches(&mut self.bodies, &self.contacts);
        for body in &mut self.bodies {
            body.integrate_position(dt);
        }
        for _ in 0..POSITION_ITERATIONS {
            for contact in &mut self.contacts {
                let body = contact.body;
                contact::solve_position(&mut self.bodies[body], &self.terrain, contact);
            }
        }
    }
}
