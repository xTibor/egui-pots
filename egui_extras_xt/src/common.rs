use std::f32::consts::TAU;

use egui::{vec2, Align2, FontFamily, FontId, Rect, Ui};
use emath::{almost_equal, lerp, Pos2, Rot2, Vec2};
use epaint::{Color32, Shape, Stroke};

use itertools::Itertools;

// ----------------------------------------------------------------------------

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Orientation {
    Right,
    Bottom,
    Left,
    Top,
    Custom(f32),
}

impl Orientation {
    pub(crate) fn rot2(&self) -> Rot2 {
        match *self {
            Self::Right => Rot2::from_angle(TAU * 0.00),
            Self::Bottom => Rot2::from_angle(TAU * 0.25),
            Self::Left => Rot2::from_angle(TAU * 0.50),
            Self::Top => Rot2::from_angle(TAU * 0.75),
            Self::Custom(angle) => Rot2::from_angle(angle),
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Winding {
    Clockwise,
    Counterclockwise,
}

impl Winding {
    pub(crate) fn to_float(&self) -> f32 {
        match *self {
            Self::Clockwise => 1.0,
            Self::Counterclockwise => -1.0,
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum WrapMode {
    None,
    Signed,
    Unsigned,
}

// ----------------------------------------------------------------------------

/// A polar function defining the shape of a knob widget.
pub type WidgetShapeFn<'a> = Box<dyn 'a + Fn(f32) -> f32>;

#[non_exhaustive]
pub enum WidgetShape<'a> {
    Circle,
    Square,
    Squircle(f32),
    Polygon(usize),
    SuperPolygon(usize, f32),
    Rotated(Box<WidgetShape<'a>>, f32),
    Mix(Box<WidgetShape<'a>>, Box<WidgetShape<'a>>, f32),
    Custom(WidgetShapeFn<'a>),
}

impl WidgetShape<'_> {
    const RESOLUTION: usize = 32;

    pub(crate) fn eval(&self, theta: f32) -> f32 {
        match self {
            WidgetShape::Circle => 1.0,
            WidgetShape::Square => (1.0 / theta.cos().abs()).min(1.0 / theta.sin().abs()),
            WidgetShape::Squircle(factor) => {
                assert!(*factor > 0.0, "squircle factor must be positive");
                let a = theta.cos().abs().powf(*factor);
                let b = theta.sin().abs().powf(*factor);
                (a + b).powf(-1.0 / *factor)
            }
            WidgetShape::Polygon(n) => {
                assert!(*n >= 3, "polygon must have at least 3 sides");
                1.0 / ((*n as f32 / 2.0 * theta).cos().asin() * 2.0 / *n as f32).cos()
            }
            WidgetShape::SuperPolygon(n, factor) => {
                assert!(*n >= 3, "polygon must have at least 3 sides");
                assert!(*factor > 0.0, "polygon factor must be positive");
                assert!(
                    (0.0..=2.0).contains(factor),
                    "polygon factor must be between 0.0 and 2.0"
                );

                // https://mathworld.wolfram.com/Superellipse.html
                let a = (0.25 * (*n as f32) * theta).cos().abs().powf(*factor);
                let b = (0.25 * (*n as f32) * theta).sin().abs().powf(*factor);
                (a + b).powf(-1.0 / *factor)
            }
            WidgetShape::Rotated(shape, rotation) => shape.eval(theta - rotation),
            WidgetShape::Mix(shape_a, shape_b, t) => {
                (shape_a.eval(theta) * (1.0 - t)) + (shape_b.eval(theta) * t)
            }
            WidgetShape::Custom(callback) => callback(theta),
        }
    }

    pub(crate) fn paint_shape(
        &self,
        ui: &mut Ui,
        center: Pos2,
        radius: f32,
        fill: Color32,
        stroke: Stroke,
        rotation: Rot2,
    ) {
        let outline_points = (0..Self::RESOLUTION)
            .map(move |i| {
                let angle = (i as f32 / Self::RESOLUTION as f32) * TAU;
                let shape_radius = self.eval(angle - (rotation * Vec2::RIGHT).angle());
                center + Vec2::angled(angle) * radius * shape_radius
            })
            .collect_vec();

        // https://github.com/emilk/egui/issues/513
        outline_points
            .iter()
            .circular_tuple_windows()
            .for_each(|(point_1, point_2)| {
                ui.painter().add(Shape::convex_polygon(
                    vec![center, *point_1, *point_2],
                    fill,
                    Stroke::new(1.0, fill),
                ));
            });

        ui.painter().add(Shape::closed_line(outline_points, stroke));
    }

    pub(crate) fn paint_arc(
        &self,
        ui: &mut Ui,
        center: Pos2,
        inner_radius: f32,
        outer_radius: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Color32,
        stroke: Stroke,
        rotation: Rot2,
    ) {
        // NOTE: convex_polygon() is broken, spews rendering artifacts all over
        //   the window when it tries to render degenerate polygons:
        //     ∃(P1,P2) ∈ Poly (dist(P1,P2) ≈ 0)

        // HACK: convex_polygon() workaround
        if almost_equal(start_angle, end_angle, 0.001) {
            let shape_radius = self.eval(start_angle - (rotation * Vec2::RIGHT).angle());

            ui.painter().add(Shape::line_segment(
                [
                    center + Vec2::angled(start_angle) * inner_radius * shape_radius,
                    center + Vec2::angled(start_angle) * outer_radius * shape_radius,
                ],
                stroke,
            ));
            return;
        }

        let generate_arc_points = |radius| {
            (0..=Self::RESOLUTION).map(move |i| {
                let angle = lerp(start_angle..=end_angle, i as f32 / Self::RESOLUTION as f32);
                let shape_radius = self.eval(angle - (rotation * Vec2::RIGHT).angle());
                center + Vec2::angled(angle) * radius * shape_radius
            })
        };

        // HACK: convex_polygon() workaround
        let inner_radius = inner_radius.max(0.1);

        let outer_arc = generate_arc_points(outer_radius).collect::<Vec<_>>();
        let inner_arc = generate_arc_points(inner_radius).collect::<Vec<_>>();

        // https://github.com/emilk/egui/issues/513
        outer_arc
            .iter()
            .zip(inner_arc.iter())
            .tuple_windows()
            .for_each(|((outer_1, inner_1), (outer_2, inner_2))| {
                ui.painter().add(Shape::convex_polygon(
                    vec![*outer_1, *inner_1, *inner_2, *outer_2],
                    fill,
                    Stroke::new(1.0, fill),
                ));
            });

        let outline_points: Vec<Pos2> = outer_arc
            .iter()
            .chain(inner_arc.iter().rev())
            .cloned()
            .collect();

        ui.painter().add(Shape::closed_line(outline_points, stroke));

        // TODO: Remove hacks and paint the arc with a single call:
        // Shape::concave_polygon(
        //     outline_points, // outer_arc.chain(inner_arc.rev())
        //     fill,
        //     stroke,
        // )
    }
}

// ----------------------------------------------------------------------------

#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum MarkerShape {
    Square,
    Circle,
    RightArrow,
    UpArrow,
    LeftArrow,
    DownArrow,
    Diamond,
    Star(usize, f32),
    Emoji(char),
}

impl MarkerShape {
    pub fn paint(&self, ui: &mut Ui, rect: Rect, fill: Color32, stroke: Stroke) {
        match *self {
            MarkerShape::Square => {
                ui.painter().rect(rect, 0.0, fill, stroke);
            }
            MarkerShape::Circle => {
                ui.painter().rect(rect, rect.width() / 2.0, fill, stroke);
            }
            MarkerShape::RightArrow => {
                let rect = Rect::from_center_size(
                    rect.center(),
                    rect.size() * vec2(3.0f32.sqrt() / 2.0, 1.0),
                );

                ui.painter().add(Shape::convex_polygon(
                    vec![rect.right_center(), rect.left_bottom(), rect.left_top()],
                    fill,
                    stroke,
                ));
            }
            MarkerShape::UpArrow => {
                let rect = Rect::from_center_size(
                    rect.center(),
                    rect.size() * vec2(1.0, 3.0f32.sqrt() / 2.0),
                );

                ui.painter().add(Shape::convex_polygon(
                    vec![rect.center_top(), rect.right_bottom(), rect.left_bottom()],
                    fill,
                    stroke,
                ));
            }
            MarkerShape::LeftArrow => {
                let rect = Rect::from_center_size(
                    rect.center(),
                    rect.size() * vec2(3.0f32.sqrt() / 2.0, 1.0),
                );

                ui.painter().add(Shape::convex_polygon(
                    vec![rect.left_center(), rect.right_top(), rect.right_bottom()],
                    fill,
                    stroke,
                ));
            }
            MarkerShape::DownArrow => {
                let rect = Rect::from_center_size(
                    rect.center(),
                    rect.size() * vec2(1.0, 3.0f32.sqrt() / 2.0),
                );

                ui.painter().add(Shape::convex_polygon(
                    vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
                    fill,
                    stroke,
                ));
            }
            MarkerShape::Diamond => {
                ui.painter().add(Shape::convex_polygon(
                    vec![
                        rect.center_top(),
                        rect.right_center(),
                        rect.center_bottom(),
                        rect.left_center(),
                    ],
                    fill,
                    stroke,
                ));
            }
            MarkerShape::Star(rays, ratio) => {
                assert!(rays >= 2, "star-shaped markers must have at least 2 rays");
                assert!(
                    (0.0..=1.0).contains(&ratio),
                    "ray ratio of star-shaped markers must be normalized"
                );

                let outer_radius = rect.width() * 0.5;
                let inner_radius = outer_radius * ratio;
                let star_rotation = -TAU * 0.25;

                let outer_points = (0..rays).map(|point_index| {
                    rect.center()
                        + Vec2::angled(
                            star_rotation + TAU * ((point_index as f32 + 0.0) / rays as f32),
                        ) * outer_radius
                });

                let inner_points = (0..rays).map(|point_index| {
                    rect.center()
                        + Vec2::angled(
                            star_rotation + TAU * ((point_index as f32 + 0.5) / rays as f32),
                        ) * inner_radius
                });

                // TODO: Broken polygon renderer
                // https://github.com/emilk/egui/issues/513
                ui.painter().add(Shape::convex_polygon(
                    outer_points.interleave(inner_points).collect_vec(),
                    fill,
                    stroke,
                ));
            }
            MarkerShape::Emoji(emoji) => {
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    emoji,
                    FontId::new(rect.height(), FontFamily::Proportional),
                    fill,
                );
            }
        }
    }
}

// ----------------------------------------------------------------------------

/// Wrap angle to `(0..TAU)` range.
pub fn normalized_angle_unsigned_excl(angle: f32) -> f32 {
    ((angle % TAU) + TAU) % TAU
}

/// Wrap angle to `(0..=TAU)` range.
pub fn normalized_angle_unsigned_incl(angle: f32) -> f32 {
    if angle < 0.0 {
        ((angle % TAU) + TAU) % TAU
    } else if angle > TAU {
        angle % TAU
    } else {
        angle
    }
}