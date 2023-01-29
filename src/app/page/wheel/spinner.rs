use eframe::emath;
use eframe::epaint::TextShape;
use egui::{Color32, FontId, Frame, Pos2, Rect, Shape, Stroke, Vec2};
use serde::{Deserialize, Serialize};

use crate::hat::Person;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SpinnerTarget {
    Speed(f32),
    Item(usize),
}

pub(crate) const ACCELERATION: f32 = 2.0;
pub(crate) const IDLE_SPEED: f32 = 0.4;
pub(crate) const FULL_SPEED: f32 = 5.0;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Spinner {
    pub(crate) items: Vec<Person>,
    pub(crate) target: SpinnerTarget,
    angle: f32,
    speed: f32,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            target: SpinnerTarget::Speed(IDLE_SPEED),
            angle: 0.0,
            speed: IDLE_SPEED,
        }
    }
}

impl Spinner {
    pub(crate) fn speed(&self) -> f32 {
        self.speed
    }

    pub(crate) fn step_animation(&mut self, delta_time: f32) {
        match self.target {
            SpinnerTarget::Speed(target_speed) => {
                let speed_delta = target_speed - self.speed;
                self.speed +=
                    speed_delta.clamp(-delta_time * ACCELERATION, delta_time * ACCELERATION);
                self.angle = (self.angle + self.speed * delta_time) % std::f32::consts::TAU;
            }
            SpinnerTarget::Item(_) => todo!(),
        }
    }

    pub(crate) fn render(&self, ui: &mut egui::Ui) {
        let text_color = if ui.visuals().dark_mode {
            Color32::from_additive_luminance(196)
        } else {
            Color32::from_black_alpha(240)
        };

        let colors = [Color32::YELLOW, Color32::GREEN, Color32::RED];
        let stroke = Stroke::new(1.0, text_color);

        Frame::canvas(ui.style()).show(ui, |ui| {
            ui.ctx().request_repaint();
            //let time = ui.input().time;

            let smaller_dimension = ui.available_width().min(ui.available_height());

            let desired_size = smaller_dimension * Vec2::new(1.0, 1.0);
            let (_id, rect) = ui.allocate_space(desired_size);

            let to_screen =
                emath::RectTransform::from_to(Rect::from_x_y_ranges(-1.0..=1.0, -1.0..=1.0), rect);

            let center = to_screen * Pos2::new(0., 0.);

            let mut shapes = vec![];

            let inner_angle = std::f32::consts::TAU / self.items.len().max(1) as f32;

            for (idx, person) in self.items.iter().enumerate() {
                let start_angle = self.angle + inner_angle * idx as f32;
                let r = smaller_dimension / 2.0 - 5.0;
                shapes.push(wedge(
                    center,
                    r,
                    start_angle,
                    inner_angle,
                    colors[idx % colors.len()],
                    stroke,
                ));

                let font_id = FontId {
                    size: r * 0.1,
                    ..Default::default()
                };
                let galley =
                    ui.fonts()
                        .layout_no_wrap(person.name.to_string(), font_id, text_color);

                let dir = Vec2::angled(start_angle + inner_angle / 2.0);
                let xoffset = r - galley.rect.width() - r * 0.1;
                let yoffset = galley.rect.height() / 2.0;

                let text_pos = center + dir * xoffset + dir.rot90() * yoffset;

                let mut text_shape = TextShape::new(text_pos, galley);
                text_shape.angle = start_angle + inner_angle / 2.0;

                shapes.push(Shape::Text(text_shape));
            }

            ui.painter().extend(shapes);
        });
    }
}

fn wedge(
    center: Pos2,
    r: f32,
    start_angle: f32,
    inner_angle: f32,
    fill: Color32,
    stroke: Stroke,
) -> Shape {
    let arc_points = 30;
    let mut points = Vec::with_capacity(arc_points + 1);

    if inner_angle < std::f32::consts::TAU - 0.01 {
        points.push(center);
    }

    let step = inner_angle / arc_points as f32;

    for a in (0..=arc_points).map(|a| start_angle + a as f32 * step) {
        points.push(center + r * Vec2::new(a.cos(), a.sin()));
    }

    Shape::convex_polygon(points, fill, stroke)
}
