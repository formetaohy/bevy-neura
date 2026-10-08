use crate::body::{BAUMGARTE, Body, LINEAR_SLOP, MAX_LINEAR_CORRECTION, Touch, VELOCITY_THRESHOLD};
use crate::terrain::{self, Terrain};
use bevy::math::Vec2;

pub const TOUCH: f32 = 0.01;

pub struct Contact {
    pub body: usize,
    pub probe: usize,
    pub part: u8,
    pub edge: usize,
    pub point: Vec2,
    pub normal: Vec2,
    pub tangent: Vec2,
    pub separation: f32,
    pub friction: f32,
    pub restitution: f32,
    pub mass_normal: f32,
    pub mass_tangent: f32,
    pub normal_impulse: f32,
    pub tangent_impulse: f32,
}

pub fn generate(bodies: &mut [Body], terrain: &Terrain, contacts: &mut Vec<Contact>) {
    contacts.clear();
    for body in bodies.iter_mut() {
        body.clear_touches();
    }
    for (index, body) in bodies.iter_mut().enumerate() {
        if !body.awake {
            continue;
        }
        for probe in 0..body.probe_count() {
            let point = body.probe_world(probe);
            let closest = terrain.closest(point);
            if closest.separation > TOUCH {
                continue;
            }
            let touch = body.touch(probe);
            let carried = touch.edge == closest.edge && touch.touching;
            let normal_impulse = if carried { touch.normal } else { 0.0 };
            let tangent_impulse = if carried { touch.tangent } else { 0.0 };
            let arm = point - body.center;
            let normal = closest.normal;
            let tangent = normal.perp();
            let mass_normal =
                1.0 / (body.inverse_mass + body.inverse_inertia * arm.perp_dot(normal).powi(2));
            let mass_tangent =
                1.0 / (body.inverse_mass + body.inverse_inertia * arm.perp_dot(tangent).powi(2));
            body.touched(
                probe,
                Touch {
                    part: touch.part,
                    edge: closest.edge,
                    normal: normal_impulse,
                    tangent: tangent_impulse,
                    touching: true,
                },
            );
            if normal_impulse != 0.0 || tangent_impulse != 0.0 {
                body.apply_solver_impulse(
                    normal_impulse * normal + tangent_impulse * tangent,
                    point,
                );
            }
            let friction = (body.friction * terrain::FRICTION).sqrt();
            let restitution = body.restitution;
            contacts.push(Contact {
                body: index,
                probe,
                part: touch.part,
                edge: closest.edge,
                point,
                normal,
                tangent,
                separation: closest.separation,
                friction,
                restitution,
                mass_normal,
                mass_tangent,
                normal_impulse,
                tangent_impulse,
            });
        }
    }
}

pub fn solve_velocity(body: &mut Body, contact: &mut Contact) {
    let approach = body.velocity_at(contact.point).dot(contact.normal);
    let bias = if approach < -VELOCITY_THRESHOLD {
        -contact.restitution * approach
    } else {
        0.0
    };
    let previous = contact.normal_impulse;
    contact.normal_impulse =
        (contact.normal_impulse - contact.mass_normal * (approach - bias)).max(0.0);
    let applied = (contact.normal_impulse - previous) * contact.normal;
    body.apply_solver_impulse(applied, contact.point);
    let slide = body.velocity_at(contact.point).dot(contact.tangent);
    let friction = contact.friction * contact.normal_impulse;
    let previous = contact.tangent_impulse;
    contact.tangent_impulse =
        (contact.tangent_impulse - contact.mass_tangent * slide).clamp(-friction, friction);
    let applied = (contact.tangent_impulse - previous) * contact.tangent;
    body.apply_solver_impulse(applied, contact.point);
}

pub fn solve_position(body: &mut Body, terrain: &Terrain, contact: &mut Contact) {
    let point = body.probe_world(contact.probe);
    let closest = terrain.closest(point);
    contact.separation = closest.separation;
    contact.normal = closest.normal;
    contact.tangent = closest.normal.perp();
    contact.point = point;
    if contact.separation > -LINEAR_SLOP {
        return;
    }
    let correction =
        (BAUMGARTE * (contact.separation + LINEAR_SLOP)).clamp(-MAX_LINEAR_CORRECTION, 0.0);
    let arm = contact.point - body.center;
    let mass_normal =
        1.0 / (body.inverse_mass + body.inverse_inertia * arm.perp_dot(contact.normal).powi(2));
    let impulse = -mass_normal * correction * contact.normal;
    body.center += body.inverse_mass * impulse;
    body.angle += body.inverse_inertia * arm.perp_dot(impulse);
}

pub fn store_touches(bodies: &mut [Body], contacts: &[Contact]) {
    for contact in contacts {
        bodies[contact.body].touched(
            contact.probe,
            Touch {
                part: contact.part,
                edge: contact.edge,
                normal: contact.normal_impulse,
                tangent: contact.tangent_impulse,
                touching: true,
            },
        );
    }
}
