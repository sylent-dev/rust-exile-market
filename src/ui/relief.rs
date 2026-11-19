//! The controls pressed into their block (`specs.md`, Shapes): an unchecked checkbox or radio
//! and a min–max field, on their fill with no line, a dark shadow inside along the top and the
//! left and a light one along the bottom and the right, as the box shadow `inset` of the
//! mock-up (section "Soft relief (22i)").
//!
//! egui casts a shadow outside a shape only (epaint 0.37), so the shape is painted as a mesh, a
//! vertex a pixel near its edges, each colored by the shadows the mock-up casts there: the
//! gaussian blur of the shape moved by the depth, read from its distance to the point.

use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape, Vec2};

/// How far each shadow is cast inside, in points (the mock-up: 1 px).
const DEPTH: f32 = 1.0;

/// The blur of the shadows, in points, as the blur radius of CSS: twice the standard deviation
/// of its gaussian (the mock-up: 2.5 px).
const BLUR: f32 = 2.5;

/// The dark shadow, along the top and the left: black at 70 % (the mock-up).
const DARK: f32 = 0.70;

/// The light shadow, along the bottom and the right: white at 3 % (the mock-up).
const LIGHT: f32 = 0.03;

/// The shape pressed in: `rect` rounded by `radius` (half its side for a radio), on `fill`.
pub fn pressed_in(painter: &Painter, rect: Rect, radius: f32, fill: Color32) -> Shape {
    let pixel = 1.0 / painter.pixels_per_point();
    Shape::mesh(mesh(Rounded { rect, radius }, fill, pixel))
}

/// How far from its edge the shadows reach into a shape, in points: past it, the shadow moved by
/// the depth covers all but 1 in 30,000 (four standard deviations of its blur), and the fill
/// alone shows.
const SHADOW_REACH: f32 = DEPTH + 2.0 * BLUR;

/// The mesh of `shape` on `fill`: a vertex every `pixel` near its edges, where the edge and the
/// shadows change the color, and none across its middle, of the fill alone.
fn mesh(shape: Rounded, fill: Color32, pixel: f32) -> Mesh {
    // One pixel more on each side, for the edge drawn smooth.
    let area = shape.rect.expand(pixel);
    let band = shape.radius.max(SHADOW_REACH) + pixel;
    let columns = vertex_places(area.min.x, area.max.x, pixel, band);
    let rows = vertex_places(area.min.y, area.max.y, pixel, band);
    let mut mesh = Mesh::default();
    mesh.reserve_vertices(columns.len() * rows.len());
    for &y in &rows {
        for &x in &columns {
            let at = Pos2::new(x, y);
            mesh.colored_vertex(at, shape.color_at(at, fill, pixel));
        }
    }
    mesh.reserve_triangles(2 * (columns.len() - 1) * (rows.len() - 1));
    let index = |row: usize, column: usize| {
        u32::try_from(row * columns.len() + column).expect("a control holds few vertices")
    };
    for row in 0..rows.len() - 1 {
        for column in 0..columns.len() - 1 {
            let (top_left, top_right) = (index(row, column), index(row, column + 1));
            let (bottom_left, bottom_right) = (index(row + 1, column), index(row + 1, column + 1));
            mesh.add_triangle(top_left, top_right, bottom_left);
            mesh.add_triangle(top_right, bottom_right, bottom_left);
        }
    }
    mesh
}

/// The places of the vertices from `start` to `end`, both included: a pixel apart within `band`
/// of either end, none between.
fn vertex_places(start: f32, end: f32, pixel: f32, band: f32) -> Vec<f32> {
    // A control is a few dozen pixels across: the cast can't lose anything.
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let pixels = ((end - start) / pixel).ceil() as usize;
    (0..=pixels)
        .map(|index| (start + step(index, pixel)).min(end))
        .filter(|&place| place - start <= band || end - place <= band)
        .collect()
}

/// The offset of the vertex `index`, a pixel each.
fn step(index: usize, pixel: f32) -> f32 {
    #[expect(clippy::cast_precision_loss)]
    let index = index as f32;
    index * pixel
}

/// A rectangle with rounded corners, a circle when `radius` is half its side.
#[derive(Debug, Clone, Copy)]
struct Rounded {
    rect: Rect,
    radius: f32,
}

impl Rounded {
    /// The signed distance from `at` to the edge: below zero inside, above outside.
    fn distance(self, at: Pos2) -> f32 {
        let half = self.rect.size() / 2.0 - Vec2::splat(self.radius);
        let from_center = (at - self.rect.center()).abs() - half;
        let outside = from_center.max(Vec2::ZERO).length();
        let inside = from_center.x.max(from_center.y).min(0.0);
        outside + inside - self.radius
    }

    /// How much of the shape moved by `offset` and blurred covers `at`, from 0 to 1.
    fn blurred_cover(self, at: Pos2, offset: Vec2) -> f32 {
        let deviation = BLUR / 2.0;
        normal_cumulative(-self.distance(at - offset) / deviation)
    }

    /// The color of the shape at `at`: `fill` darkened by the dark shadow and lightened by the
    /// light one, then as much of it as the shape covers of the pixel `pixel` wide there.
    fn color_at(self, at: Pos2, fill: Color32, pixel: f32) -> Color32 {
        let dark = DARK * (1.0 - self.blurred_cover(at, Vec2::splat(DEPTH)));
        let light = LIGHT * (1.0 - self.blurred_cover(at, Vec2::splat(-DEPTH)));
        let cover = (0.5 - self.distance(at) / pixel).clamp(0.0, 1.0);
        let channel = |value: u8| {
            let shaded = f32::from(value) * (1.0 - dark) * (1.0 - light) + 255.0 * light;
            to_byte(shaded * cover)
        };
        Color32::from_rgba_premultiplied(
            channel(fill.r()),
            channel(fill.g()),
            channel(fill.b()),
            to_byte(255.0 * cover),
        )
    }
}

/// A value from 0 to 255 as a byte.
fn to_byte(value: f32) -> u8 {
    // Clamped first, so the cast loses only the fraction.
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = value.round().clamp(0.0, 255.0) as u8;
    byte
}

/// The share of a normal distribution below `x` standard deviations.
fn normal_cumulative(x: f32) -> f32 {
    f32::midpoint(1.0, error_function(x / std::f32::consts::SQRT_2))
}

/// The error function, by the approximation 7.1.26 of Abramowitz and Stegun (an error below
/// 1.5e-7, far under a step of a color).
fn error_function(x: f32) -> f32 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let polynomial = t
        * (0.254_829_6
            + t * (-0.284_496_74 + t * (1.421_413_7 + t * (-1.453_152_1 + t * 1.061_405_4))));
    let value = 1.0 - polynomial * (-x * x).exp();
    value.copysign(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    /// A min–max field 60 by 20 points from the origin, rounded as the controls.
    fn field() -> Rounded {
        Rounded {
            rect: Rect::from_min_size(Pos2::ZERO, vec2(60.0, 20.0)),
            radius: 3.0,
        }
    }

    #[test]
    fn the_distance_is_below_zero_inside_and_above_outside() {
        let field = field();
        assert_eq!(field.distance(pos2(30.0, 10.0)), -10.0);
        assert_eq!(field.distance(pos2(30.0, 0.0)), 0.0);
        assert_eq!(field.distance(pos2(30.0, -2.0)), 2.0);
        // Past the round corner, though inside the rectangle.
        assert!(field.distance(pos2(0.2, 0.2)) > 0.0);
    }

    #[test]
    fn a_radio_is_a_circle() {
        let radio = Rounded {
            rect: Rect::from_center_size(Pos2::ZERO, Vec2::splat(13.0)),
            radius: 6.5,
        };
        let diagonal = 6.5 / std::f32::consts::SQRT_2;
        assert!(radio.distance(pos2(diagonal, diagonal)).abs() < 1e-5);
    }

    #[test]
    fn the_shadow_is_dark_along_the_top_and_the_left_only() {
        let field = field();
        let fill = Color32::from_rgb(100, 100, 100);
        // Half a pixel inside each edge: the dark shadow there, black at 70 % of what the
        // shape moved down right and blurred leaves uncovered, then the light one.
        let edges = [(30.0, 0.5), (0.5, 10.0), (30.0, 19.5), (59.5, 10.0)];
        let shades = edges.map(|(x, y)| field.color_at(pos2(x, y), fill, 1.0).r());
        assert_eq!(shades, [55, 55, 95, 95]);
    }

    #[test]
    fn the_middle_keeps_its_fill_and_the_outside_is_clear() {
        let field = field();
        let fill = Color32::from_rgb(100, 100, 100);
        assert_eq!(field.color_at(pos2(30.0, 10.0), fill, 1.0), fill);
        assert_eq!(
            field.color_at(pos2(30.0, -1.0), fill, 1.0),
            Color32::TRANSPARENT
        );
    }

    #[test]
    fn the_mesh_has_a_vertex_a_pixel_near_the_edges_and_none_across_the_middle() {
        // 17 points across with the pixel around: 18 places, less the 2 of the middle beyond the
        // reach of the shadows; at two pixels a point, 33 less 5.
        let check = Rounded {
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(15.0)),
            radius: 3.0,
        };
        for (pixel, side) in [(1.0, 16), (0.5, 28)] {
            let mesh = mesh(check, Color32::WHITE, pixel);
            assert_eq!(mesh.vertices.len(), side * side);
            assert_eq!(mesh.indices.len(), 3 * 2 * (side - 1) * (side - 1));
        }
        // 62 points across, -1 to 63: the 8 places from each end within the reach of the
        // shadows (6 points) and the pixel around.
        let places = vertex_places(-1.0, 63.0, 1.0, SHADOW_REACH + 1.0);
        let ends: [f32; 8] = [-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let from_the_end = ends.iter().rev().map(|place| 62.0 - place);
        let expected: Vec<f32> = ends.iter().copied().chain(from_the_end).collect();
        assert_eq!(places, expected);
    }

    #[test]
    fn the_error_function_matches_its_table() {
        // erf(0.5) and erf(1) to six decimals (Abramowitz and Stegun, table 7.1).
        assert!((error_function(0.5) - 0.520_500).abs() < 1e-5);
        assert!((error_function(1.0) - 0.842_701).abs() < 1e-5);
        assert_eq!(error_function(-1.0), -error_function(1.0));
        assert!((normal_cumulative(0.0) - 0.5).abs() < 1e-6);
    }
}
