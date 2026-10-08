use bevy::prelude::*;

const COLUMNS: usize = 3;
const ROWS: usize = 5;

const PATTERNS: [[u8; ROWS]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

pub struct Style {
    pub at: Vec2,
    pub dot: f32,
    pub gap: f32,
    pub slot: f32,
    pub slots: usize,
    pub lit: Color,
    pub ghost: Color,
    pub layer: f32,
}

#[derive(Component)]
pub struct Glyphs {
    slots: usize,
    lit: Color,
    ghost: Color,
    text: Vec<u8>,
}

impl Glyphs {
    pub fn show(&mut self, value: u32) -> bool {
        let digits = format!("{:0>width$}", value, width = self.slots);
        let text = digits.as_bytes()[digits.len() - self.slots..].to_vec();
        let changed = text != self.text;
        self.text = text;
        changed
    }

    fn dot(&self, dot: &Dot) -> Color {
        let symbol = self.text[dot.slot].saturating_sub(b'0') as usize;
        let lit = PATTERNS[symbol][dot.row] >> (COLUMNS - 1 - dot.column) & 1 == 1;
        if lit { self.lit } else { self.ghost }
    }
}

#[derive(Component)]
pub struct Dot {
    slot: usize,
    row: usize,
    column: usize,
}

pub fn spawn(commands: &mut Commands, style: Style) -> Entity {
    assert!(style.slots > 0, "a display of no slot holds no digit");
    let pitch = COLUMNS as f32 * (style.dot + style.gap) - style.gap;
    let height = ROWS as f32 * (style.dot + style.gap) - style.gap;
    let reach = style.slots as f32 * pitch + (style.slots - 1) as f32 * style.slot;
    let root = commands
        .spawn((
            Glyphs {
                slots: style.slots,
                lit: style.lit,
                ghost: style.ghost,
                text: vec![b'0'; style.slots],
            },
            Visibility::Inherited,
            Transform::from_xyz(style.at.x, style.at.y, style.layer),
        ))
        .id();
    commands.entity(root).with_children(|display| {
        for slot in 0..style.slots {
            let left = -reach / 2.0 + slot as f32 * (pitch + style.slot);
            for row in 0..ROWS {
                for column in 0..COLUMNS {
                    display.spawn((
                        Dot { slot, row, column },
                        Sprite::from_color(style.ghost, Vec2::splat(style.dot)),
                        Transform::from_xyz(
                            left + column as f32 * (style.dot + style.gap) + style.dot / 2.0,
                            height / 2.0 - row as f32 * (style.dot + style.gap) - style.dot / 2.0,
                            0.0,
                        ),
                    ));
                }
            }
        }
    });
    root
}

pub fn paint(
    displays: Query<(&Glyphs, &Children), Changed<Glyphs>>,
    mut dots: Query<(&Dot, &mut Sprite)>,
) {
    for (glyphs, children) in &displays {
        for child in children {
            let Ok((dot, mut sprite)) = dots.get_mut(*child) else {
                continue;
            };
            sprite.color = glyphs.dot(dot);
        }
    }
}
