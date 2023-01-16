use eframe::emath;
use eframe::epaint::TextShape;
use egui::{Align, Color32, FontId, Frame, Layout, Pos2, Rect, RichText, Shape, Stroke, Vec2};

use crate::hat::{DrawError, Hat, Pair, Person};
use crate::valid_pair;

use super::UiExtensions;

const ACCELERATION: f32 = 2.0;
const IDLE_SPEED: f32 = 0.4;
const FULL_SPEED: f32 = 5.0;
const SPIN_TIME: f32 = 5.0;

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum WheelAnim {
    Idle,
    Windup,
    HoldAtTopSpeed{
        start_time: f32
    },
    SlowToStop,
}

impl Default for WheelAnim {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub(crate) struct WheelPage {
    hat: Hat,
    drawn_names: Vec<Pair>,
    error_message: Option<String>,
    animation: WheelAnim,
    angle: f32,
    speed: f32,
}

impl WheelPage {
    fn reset(&mut self, people: &[Person]) {
        self.animation = WheelAnim::default();
        self.error_message = None;
        self.hat = Hat::with_people(people.into());
        self.drawn_names.clear();
    }

    fn spin(&mut self) {
        self.animation = WheelAnim::Windup;
        self.error_message = None;
        match self.hat.draw_name(valid_pair) {
            Ok(pair) => {
                self.drawn_names.push(pair);
            },
            Err(DrawError::NoGivers) => self.error_message = Some("No one left to assign".into()),
            //This case needs to have some 'just draw someone' option
            Err(DrawError::NoValidReceiver) => {
                self.error_message = Some("It isn't possible to assign everyone".into())
            }
        }
    }

    pub(crate) fn display(&mut self, people: &[Person], ctx: &egui::Context) {
        
        egui::SidePanel::left("wheel-left").show(ctx, |ui| side_panel(ui, self));

        egui::TopBottomPanel::bottom("wheel-bottom").show(ctx, |ui| bottom_panel(ui, self, people));

        egui::CentralPanel::default().show(ctx, |ui| center_spinner(ui, self));
    }

    fn update_animation(&mut self, ui: &egui::Ui) {
        let delta_time = ui.input().stable_dt.min(0.1);
        let time = ui.input().time;
        match self.animation {
            WheelAnim::Idle => {
                self.step_physics(IDLE_SPEED, delta_time);
            },
            WheelAnim::Windup => {
                self.step_physics(FULL_SPEED, delta_time);
                if self.speed == FULL_SPEED {
                    self.animation = WheelAnim::HoldAtTopSpeed { start_time: time as f32};
                }
            },
            WheelAnim::HoldAtTopSpeed { start_time } => {
                self.step_physics(FULL_SPEED, delta_time);
                if time as f32 - start_time >= SPIN_TIME {
                    self.animation = WheelAnim::SlowToStop;
                }
            },
            WheelAnim::SlowToStop => {
                self.step_physics(0.0, delta_time);
            }
            
        }
    
    }

    fn step_physics(&mut self, target_speed: f32, delta_time: f32) {
        let speed_delta = target_speed - self.speed;
        self.speed += speed_delta.clamp(-delta_time * ACCELERATION, delta_time * ACCELERATION);
        self.angle = (self.angle + self.speed * delta_time) % std::f32::consts::TAU;
    }
}

fn side_panel(ui: &mut egui::Ui, wheel: &mut WheelPage) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Remaining Givers");
        for person in wheel.hat.givers().iter() {
            ui.person(person);
        }
        ui.separator();
        ui.heading("Results");
        for pair in &wheel.drawn_names {
            ui.horizontal(|ui| {
                ui.person(&pair.giver);
                ui.label("==>");
                ui.person(&pair.receiver);
            });
        }
    });
}

fn bottom_panel(ui: &mut egui::Ui, wheel: &mut WheelPage, people: &[Person]) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        if !wheel.hat.givers().is_empty()
            && ui.button(RichText::new("Spin Wheel").heading()).clicked()
        {
            wheel.spin();
        }

        if let Some(msg) = &wheel.error_message {
            ui.colored_label(Color32::RED, msg);
        }

        if ui.button("Restart").clicked() {
            wheel.reset(people);
        }
    });
}

fn center_spinner(ui: &mut egui::Ui, wheel: &mut WheelPage) {
    wheel.update_animation(ui);
    
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

        let inner_angle = std::f32::consts::TAU / wheel.hat.receivers().len() as f32;

        for (idx, person) in wheel.hat.receivers().iter().enumerate() {
            let start_angle = wheel.angle + inner_angle * idx as f32;
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
            let galley = ui
                .fonts()
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
