use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use std::f32::consts::TAU;

pub fn disc(radius: f32, rings: u32, segments: u32, shade: impl Fn(Vec2) -> Color) -> Mesh {
    assert!(
        radius > 0.0 && rings > 0 && segments >= 3,
        "a disc of {radius} over {rings} rings of {segments} segments holds no area",
    );
    let mut positions = vec![[0.0, 0.0, 0.0]];
    let mut colors = vec![linear(shade(Vec2::ZERO))];
    for ring in 1..=rings {
        let reach = radius * ring as f32 / rings as f32;
        for segment in 0..segments {
            let offset = Vec2::from_angle(TAU * segment as f32 / segments as f32) * reach;
            positions.push([offset.x, offset.y, 0.0]);
            colors.push(linear(shade(offset)));
        }
    }
    let mut indices = Vec::new();
    for segment in 0..segments {
        indices.extend_from_slice(&[0, 1 + segment, 1 + (segment + 1) % segments]);
    }
    for ring in 1..rings {
        let low = 1 + (ring - 1) * segments;
        let high = 1 + ring * segments;
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            indices.extend_from_slice(&[
                low + segment,
                high + next,
                high + segment,
                low + segment,
                low + next,
                high + next,
            ]);
        }
    }
    finish(&positions, &colors, &indices)
}

pub fn grid(rows: &[Vec<Vec2>], shade: impl Fn(usize, usize, Vec2) -> Color) -> Mesh {
    let columns = rows.first().map_or(0, Vec::len);
    assert!(
        rows.len() >= 2 && columns >= 2,
        "a grid of {} rows of {columns} points holds no cell",
        rows.len(),
    );
    assert!(
        rows.iter().all(|row| row.len() == columns),
        "a grid holds rows of one length",
    );
    let mut positions = Vec::with_capacity(rows.len() * columns);
    let mut colors = Vec::with_capacity(rows.len() * columns);
    for (index, row) in rows.iter().enumerate() {
        for (column, point) in row.iter().enumerate() {
            positions.push([point.x, point.y, 0.0]);
            colors.push(linear(shade(index, column, *point)));
        }
    }
    let stride = columns as u32;
    let mut indices = Vec::new();
    for row in 0..rows.len() as u32 - 1 {
        for column in 0..stride - 1 {
            let corner = row * stride + column;
            indices.extend_from_slice(&[
                corner,
                corner + 1,
                corner + stride + 1,
                corner,
                corner + stride + 1,
                corner + stride,
            ]);
        }
    }
    finish(&positions, &colors, &indices)
}

pub fn blend(color: Color) -> ColorMaterial {
    ColorMaterial {
        color,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    }
}

pub fn mix(low: Color, high: Color, share: f32) -> Color {
    let low = low.to_srgba();
    let high = high.to_srgba();
    let share = share.clamp(0.0, 1.0);
    Color::srgba(
        low.red + (high.red - low.red) * share,
        low.green + (high.green - low.green) * share,
        low.blue + (high.blue - low.blue) * share,
        low.alpha + (high.alpha - low.alpha) * share,
    )
}

pub fn ramp(stops: &[(f32, Color)], share: f32) -> Color {
    assert!(
        stops.len() >= 2,
        "a ramp of {} stops holds no color",
        stops.len()
    );
    let share = share.clamp(0.0, 1.0);
    let mut index = 0;
    while index + 2 < stops.len() && share > stops[index + 1].0 {
        index += 1;
    }
    let (low_at, low) = stops[index];
    let (high_at, high) = stops[index + 1];
    mix(low, high, (share - low_at) / (high_at - low_at))
}

pub fn gain(color: Color, factor: f32) -> Color {
    let color = color.to_srgba();
    Color::srgb(
        (color.red * factor).min(1.0),
        (color.green * factor).min(1.0),
        (color.blue * factor).min(1.0),
    )
}

pub fn ball(offset: Vec2, radius: f32, light: Vec2, dark: Color, bright: Color) -> Color {
    let normal = offset / radius;
    let lambert = if normal.length_squared() < 1e-6 {
        0.6
    } else {
        normal.dot(light).clamp(0.0, 1.0)
    };
    mix(dark, bright, (0.35 + 0.65 * lambert).powf(1.25))
}

fn linear(color: Color) -> [f32; 4] {
    let color = color.to_linear();
    [color.red, color.green, color.blue, color.alpha]
}

fn finish(positions: &[[f32; 3]], colors: &[[f32; 4]], indices: &[u32]) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.to_vec());
    mesh.insert_indices(Indices::U32(indices.to_vec()));
    mesh
}
