//! The controls pressed into their block (`specs.md`, Shapes): an unchecked checkbox or radio
//! and a min–max field, on their fill with no line, a dark shadow inside along the top and the
//! left and a light one along the bottom and the right, as the box shadow `inset` of the
//! mock-up (section "Soft relief (22i)").
//!
//! egui casts a shadow outside a shape only (epaint 0.37), so the shape is painted as a mesh, a
//! vertex at the center of each pixel near its edges, each colored by the shadows the mock-up
//! casts there: the gaussian blur of the shape moved by the depth, read from its distance to
//! the point.

use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape, Vec2};

/// How far each shadow is cast inside, in points (the mock-up: 1 px).
const DEPTH: f32 = 1.0;

/// The blur of the shadows, in points, as the blur radius of CSS: twice the standard deviation
/// of its gaussian (the mock-up: 2.5 px).
const BLUR: f32 = 2.5;

/// The opacity of the dark shadow, black along the top and the left (the mock-up: 70 %).
const DARK_OPACITY: f32 = 0.70;

/// The opacity of the light shadow, white along the bottom and the right (the mock-up: 3 %).
const LIGHT_OPACITY: f32 = 0.03;

/// How far from its edge the shadows reach into a shape, in points: past it, the shadow moved by
/// the depth covers all but 1 in 30,000 (four standard deviations of its blur), and the fill
/// alone shows.
const SHADOW_REACH: f32 = DEPTH + 2.0 * BLUR;

/// A control pressed in: `rect` rounded by `radius` (half its side for a radio), and the width
/// of its `border`, the line of the mock-up made transparent, under which the fill runs and
/// inside which the shadows start, as CSS casts a box shadow `inset`.
#[derive(Debug, Clone, Copy)]
pub struct Pressed {
    pub rect: Rect,
    pub radius: f32,
    pub border: f32,
}

/// The control pressed in, on `fill`.
pub fn pressed_in(painter: &Painter, control: Pressed, fill: Color32) -> Shape {
    Shape::mesh(mesh(control, fill, painter.pixels_per_point()))
}

/// The mesh of `control` on `fill`: a vertex at the center of each pixel near its edges, where
/// the edge and the shadows change the color, and none across its middle, of the fill alone.
fn mesh(control: Pressed, fill: Color32, pixels_per_point: f32) -> Mesh {
    let mut mesh = Mesh::default();
    // A layout gone wrong: a rect without end would give vertices without end.
    if !control.rect.is_finite() {
        return mesh;
    }
    let pixel = 1.0 / pixels_per_point;
    let band = control.radius.max(control.border + SHADOW_REACH) + pixel;
    let rect = control.rect;
    let columns = vertex_places(rect.min.x, rect.max.x, pixels_per_point, band);
    let rows = vertex_places(rect.min.y, rect.max.y, pixels_per_point, band);
    mesh.reserve_vertices(columns.len() * rows.len());
    for &y in &rows {
        for &x in &columns {
            let at = Pos2::new(x, y);
            mesh.colored_vertex(at, control.color_at(at, fill, pixel));
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

/// The places of the vertices from `start` to `end`, in points: the centers of the pixels of the
/// screen the span covers, and of one more on each side, outside it, those within `band` of
/// either end. A pixel is read at its center: a vertex there gives it its own color, the edge
/// as sharp as egui draws a rect.
fn vertex_places(start: f32, end: f32, pixels_per_point: f32, band: f32) -> Vec<f32> {
    let first = (start * pixels_per_point).floor() - 1.0;
    let last = (end * pixels_per_point).ceil();
    // A finite control is a few dozen pixels across: the casts lose nothing.
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = (last - first) as usize;
    (0..=count)
        .map(|index| {
            #[expect(clippy::cast_precision_loss)]
            let index = index as f32;
            (first + index + 0.5) / pixels_per_point
        })
        .filter(|&place| place - start <= band || end - place <= band)
        .collect()
}

impl Pressed {
    /// The color of the control at `at`: `fill`, lightened by the light shadow then darkened by
    /// the dark one over it (the first shadow of CSS is on top), inside the border only; then
    /// as much of it as the control covers of the pixel `pixel` wide there.
    fn color_at(self, at: Pos2, fill: Color32, pixel: f32) -> Color32 {
        let inside = self.padding().cover(at, pixel);
        let shadow = |offset: f32, opacity: f32| {
            opacity * (1.0 - self.padding().blurred_cover(at, Vec2::splat(offset))) * inside
        };
        let dark = shadow(DEPTH, DARK_OPACITY);
        let light = shadow(-DEPTH, LIGHT_OPACITY);
        let cover = Rounded::of(self.rect, self.radius).cover(at, pixel);
        let channel = |value: u8| {
            let shaded = (f32::from(value) * (1.0 - light) + 255.0 * light) * (1.0 - dark);
            to_byte(shaded * cover)
        };
        Color32::from_rgba_premultiplied(
            channel(fill.r()),
            channel(fill.g()),
            channel(fill.b()),
            to_byte(255.0 * cover),
        )
    }

    /// The shape inside the border, where CSS casts a shadow `inset`, its corners rounded by
    /// the radius less the border.
    fn padding(self) -> Rounded {
        Rounded::of(
            self.rect.shrink(self.border),
            (self.radius - self.border).max(0.0),
        )
    }
}

/// A rectangle with rounded corners, a circle when `radius` is half its side.
#[derive(Debug, Clone, Copy)]
struct Rounded {
    rect: Rect,
    radius: f32,
}

impl Rounded {
    fn of(rect: Rect, radius: f32) -> Self {
        Self { rect, radius }
    }

    /// The signed distance from `at` to the edge: below zero inside, above outside.
    fn distance(self, at: Pos2) -> f32 {
        let half = self.rect.size() / 2.0 - Vec2::splat(self.radius);
        let from_center = (at - self.rect.center()).abs() - half;
        let outside = from_center.max(Vec2::ZERO).length();
        let inside = from_center.x.max(from_center.y).min(0.0);
        outside + inside - self.radius
    }

    /// How much of the pixel `pixel` wide at `at` the shape covers, from 0 to 1.
    fn cover(self, at: Pos2, pixel: f32) -> f32 {
        (0.5 - self.distance(at) / pixel).clamp(0.0, 1.0)
    }

    /// How much of the shape moved by `offset` and blurred covers `at`, from 0 to 1: the blur
    /// of the half-plane at the nearest edge, exact along a straight edge and a few hundredths
    /// light in a rounded corner, under a step of a color on the dark fills of the controls.
    fn blurred_cover(self, at: Pos2, offset: Vec2) -> f32 {
        let deviation = BLUR / 2.0;
        normal_cumulative(-self.distance(at - offset) / deviation)
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

    /// A min–max field 60 by 20 points from the origin, rounded as the controls, its border of
    /// 1 point.
    fn field() -> Pressed {
        Pressed {
            rect: Rect::from_min_size(Pos2::ZERO, vec2(60.0, 20.0)),
            radius: 3.0,
            border: 1.0,
        }
    }

    /// The gray of the field on a fill of gray 100, at each center of a pixel of `places`.
    fn shades<const N: usize>(places: [(f32, f32); N]) -> [u8; N] {
        let fill = Color32::from_rgb(100, 100, 100);
        places.map(|(x, y)| field().color_at(pos2(x, y), fill, 1.0).r())
    }

    #[test]
    fn the_distance_is_below_zero_inside_and_above_outside() {
        let field = Rounded::of(field().rect, 3.0);
        assert_eq!(field.distance(pos2(30.0, 10.0)), -10.0);
        assert_eq!(field.distance(pos2(30.0, 0.0)), 0.0);
        assert_eq!(field.distance(pos2(30.0, -2.0)), 2.0);
        // Past the round corner, though inside the rectangle.
        assert!(field.distance(pos2(0.2, 0.2)) > 0.0);
    }

    #[test]
    fn a_radio_is_a_circle() {
        let radio = Rounded::of(Rect::from_center_size(Pos2::ZERO, Vec2::splat(13.0)), 6.5);
        let diagonal = 6.5 / std::f32::consts::SQRT_2;
        assert!(radio.distance(pos2(diagonal, diagonal)).abs() < 1e-5);
    }

    #[test]
    fn the_shadow_starts_inside_the_border() {
        // The pixel under the border keeps the fill; the next one is the darkest, the light
        // shadow under the dark one: (100 × (1 − 0.3 %) + 255 × 0.3 %) × (1 − 46 %).
        assert_eq!(
            shades([(30.0, 0.5), (30.0, 1.5), (30.0, 2.5)]),
            [100, 54, 76]
        );
        assert_eq!(shades([(0.5, 10.0), (1.5, 10.0)]), [100, 54]);
    }

    #[test]
    fn the_shadow_is_darker_along_the_top_and_the_left() {
        // Inside the border: 54 at the top and the left, 95 at the bottom and the right, where
        // the dark shadow still reaches a little.
        let inner_edges = [(30.0, 1.5), (1.5, 10.0), (30.0, 18.5), (58.5, 10.0)];
        assert_eq!(shades(inner_edges), [54, 54, 95, 95]);
    }

    #[test]
    fn the_middle_keeps_its_fill_and_the_outside_is_clear() {
        let fill = Color32::from_rgb(100, 100, 100);
        assert_eq!(field().color_at(pos2(30.0, 10.0), fill, 1.0), fill);
        assert_eq!(
            field().color_at(pos2(30.0, -0.5), fill, 1.0),
            Color32::TRANSPARENT
        );
    }

    #[test]
    fn the_vertices_are_at_the_centers_of_the_pixels_near_the_edges() {
        // 62 points across from 0: the pixels 0 to 61 and one more on each side, those within
        // the band from either end.
        let band = 1.0 + SHADOW_REACH + 1.0;
        let places = vertex_places(0.0, 62.0, 1.0, band);
        let ends = [-0.5, 0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5];
        let from_the_end = ends.iter().rev().map(|place| 62.0 - place);
        let expected: Vec<f32> = ends.iter().copied().chain(from_the_end).collect();
        assert_eq!(places, expected);
        // A rect off the pixels, at two pixels a point: the centers of the pixels all the same.
        let places = vertex_places(0.3, 2.0, 2.0, band);
        assert_eq!(places, [-0.25, 0.25, 0.75, 1.25, 1.75, 2.25]);
    }

    #[test]
    fn the_edge_is_sharp_on_the_pixels() {
        let check = Pressed {
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::splat(15.0)),
            radius: 3.0,
            border: 1.0,
        };
        let mesh = mesh(check, Color32::WHITE, 1.0);
        let alpha = |at: Pos2| {
            let vertex = mesh.vertices.iter().find(|vertex| vertex.pos == at);
            vertex.map(|vertex| vertex.color.a())
        };
        assert_eq!(alpha(pos2(-0.5, 7.5)), Some(0));
        assert_eq!(alpha(pos2(0.5, 7.5)), Some(255));
    }

    #[test]
    fn the_mesh_leaves_the_middle_without_vertices() {
        // 60 by 20 points: the pixels and one more on each side, those within the border, the
        // reach of the shadows and a pixel from an edge, 8 points: 9 at each end of each side;
        // at two pixels a point, within 7.5 points, 16.
        for (pixels_per_point, side) in [(1.0, 18), (2.0, 32)] {
            let mesh = mesh(field(), Color32::WHITE, pixels_per_point);
            assert_eq!(mesh.vertices.len(), side * side);
            assert_eq!(mesh.indices.len(), 3 * 2 * (side - 1) * (side - 1));
        }
    }

    #[test]
    fn a_rect_without_end_gives_an_empty_mesh() {
        let control = Pressed {
            rect: Rect::EVERYTHING,
            radius: 3.0,
            border: 1.0,
        };
        assert!(mesh(control, Color32::WHITE, 1.0).is_empty());
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
