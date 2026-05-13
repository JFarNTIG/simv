use macroquad::prelude::*;

/// Renders an arrow from `(x1, y1)` to `(x2, y2)`.
pub fn draw_arrow(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    draw_line(x1, y1, x2, y2, thickness, color);
    let angle = (y2 - y1).atan2(x2 - x1);
    let head_len = 15.0;
    let head_angle = 0.5;

    let wing = |a_off: f32| {
        (x2 - head_len * (angle - a_off).cos(), y2 - head_len * (angle - a_off).sin())
    };

    let (xw1, yw1) = wing(head_angle);
    let (xw2, yw2) = wing(-head_angle);

    draw_line(x2, y2, xw1, yw1, thickness, color);
    draw_line(x2, y2, xw2, yw2, thickness, color);
}

/// Renders a selection box around a point.
pub fn draw_selection_box(x1: f32, y1: f32, size: f32, thickness: f32, color: Color) {
    let half_size = size / 2.;
    let bracket_size = size / 4.;

    // top left
    draw_line(x1 - half_size, y1 - half_size, x1 - half_size + bracket_size, y1 - half_size, thickness, color);
    draw_line(x1 - half_size, y1 - half_size, x1 - half_size, y1 - half_size + bracket_size, thickness, color);

    // top right
    draw_line(x1 + half_size, y1 - half_size, x1 + half_size - bracket_size, y1 - half_size, thickness, color);
    draw_line(x1 + half_size, y1 - half_size, x1 + half_size, y1 - half_size + bracket_size, thickness, color);

    // bottom left
    draw_line(x1 - half_size, y1 + half_size, x1 - half_size + bracket_size, y1 + half_size, thickness, color);
    draw_line(x1 - half_size, y1 + half_size, x1 - half_size, y1 + half_size - bracket_size, thickness, color);

    // bottom right
    draw_line(x1 + half_size, y1 + half_size, x1 + half_size - bracket_size, y1 + half_size, thickness, color);
    draw_line(x1 + half_size, y1 + half_size, x1 + half_size, y1 + half_size - bracket_size, thickness, color);
}