use bevy::prelude::*;

pub fn text(size: f32, color: Color) -> (TextFont, TextColor) {
    (TextFont::from_font_size(size), TextColor(color))
}

pub fn screen() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(20.0),
        ..default()
    }
}
