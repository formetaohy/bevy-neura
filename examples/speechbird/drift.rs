use bevy::prelude::*;

#[derive(Component)]
pub struct Drift {
    speed: f32,
    span: f32,
}

impl Drift {
    pub fn new(speed: f32, span: f32) -> Self {
        assert!(span > 0.0, "a drift over {span} wraps nowhere");
        Self { speed, span }
    }
}

pub fn sweep(time: Res<Time>, mut drifts: Query<(&mut Transform, &Drift)>) {
    let seconds = time.delta_secs();
    for (mut place, drift) in &mut drifts {
        place.translation.x -= drift.speed * seconds;
        if place.translation.x < -drift.span {
            place.translation.x += 2.0 * drift.span;
        }
    }
}
