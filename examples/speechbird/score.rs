use crate::digit::{self, Glyphs, Style};
use crate::game::{Game, Stage};
use crate::view;
use crate::world;
use bevy::prelude::*;

const SLOTS: usize = 2;
const TOP: f32 = world::CEILING - 78.0;
const RECORD_TOP: f32 = TOP - 42.0;
const LIT: Color = Color::srgb(0.99, 0.95, 0.83);
const GHOST: Color = Color::srgb(0.26, 0.31, 0.43);
const RECORD_LIT: Color = Color::srgb(0.60, 0.86, 0.92);
const RECORD_GHOST: Color = Color::srgb(0.15, 0.20, 0.30);

#[derive(Component)]
pub enum Tally {
    Score,
    Best,
}

#[derive(Component)]
pub struct Swell(f32);

pub fn setup(mut commands: Commands) {
    let score = digit::spawn(
        &mut commands,
        Style {
            at: Vec2::new(0.0, TOP),
            dot: 4.5,
            gap: 2.0,
            slot: 7.0,
            slots: SLOTS,
            lit: LIT,
            ghost: GHOST,
            layer: view::DISPLAY,
        },
    );
    commands.entity(score).insert((Tally::Score, Swell(0.0)));
    let best = digit::spawn(
        &mut commands,
        Style {
            at: Vec2::new(0.0, RECORD_TOP),
            dot: 3.0,
            gap: 1.4,
            slot: 6.0,
            slots: SLOTS,
            lit: RECORD_LIT,
            ghost: RECORD_GHOST,
            layer: view::DISPLAY,
        },
    );
    commands.entity(best).insert((Tally::Best, Swell(0.0)));
}

pub fn paint(
    time: Res<Time>,
    game: Res<Game>,
    mut tallies: Query<(&Tally, &mut Glyphs, &mut Swell, &mut Transform)>,
) {
    let seconds = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let ended = matches!(game.stage(), Stage::Over);
    for (tally, mut glyphs, mut swell, mut place) in &mut tallies {
        let value = match tally {
            Tally::Score => game.score(),
            Tally::Best => game.best(),
        };
        if glyphs.show(value) {
            swell.0 = 1.0;
        }
        swell.0 = (swell.0 - seconds * 3.2).max(0.0);
        let rest = if ended {
            0.04 + 0.04 * (elapsed * 3.4).sin()
        } else {
            0.0
        };
        place.scale = Vec3::splat(1.0 + 0.24 * swell.0 * swell.0 + rest);
    }
}
