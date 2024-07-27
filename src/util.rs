use bevy::prelude::*;

/// Splits a string into sections based on simple markup:
///
/// * Text surrounded by * (e.g. *Hello*) will be highlighted
pub fn highlight_text(
    str: &str,
    base_color: Color,
    highlight_color: Color,
    font_size: f32,
    font: Handle<Font>,
) -> Vec<TextSection> {
    str.split('*')
        .zip((&[false, true]).into_iter().copied().cycle())
        .map(move |(s, highlight)| TextSection {
            value: s.to_string(),
            style: TextStyle {
                color: if highlight {
                    highlight_color
                } else {
                    base_color
                },
                font_size,
                font: font.clone(),
                ..Default::default()
            },
        })
        .collect()
}
