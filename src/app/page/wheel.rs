use eframe::emath;
use eframe::epaint::TextShape;
use egui::{Align, Color32, Frame, Layout, Pos2, Rect, RichText, Shape, Stroke, Vec2, Align2, FontId};

use crate::hat::{DrawError, Hat, Pair, Person};
use crate::valid_pair;

use super::UiExtensions;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(crate) struct WheelPage {
    hat: Hat,
    drawn_names: Vec<Pair>,
    error_message: Option<String>,
}

impl WheelPage {
    fn reset(&mut self, people: &[Person]) {
        self.error_message = None;
        self.hat = Hat::with_people(people.into());
        self.drawn_names.clear();
    }

    fn spin(&mut self) {
        self.error_message = None;
        match self.hat.draw_name(valid_pair) {
            Ok(pair) => self.drawn_names.push(pair),
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
        if wheel.hat.givers().len() > 0 {
            if ui.button(RichText::new("Spin Wheel").heading()).clicked() {
                wheel.spin();
            }
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
    let text_color = if ui.visuals().dark_mode {
        Color32::from_additive_luminance(196)
    } else {
        Color32::from_black_alpha(240)
    };

    let colors = [
        Color32::YELLOW,
        Color32::GREEN,
        Color32::RED,
    ];
    let stroke = Stroke::new(1.0, text_color);

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let time = ui.input().time;

        let smaller_dimension = ui.available_width().min(ui.available_height());
        

        let desired_size = smaller_dimension * Vec2::new(1.0, 1.0);
        let (_id, rect) = ui.allocate_space(desired_size);

        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(-1.0..=1.0, -1.0..=1.0), rect);

        let center = to_screen * Pos2::new(0.,0.);

        let mut shapes = vec![];

        let inner_angle = std::f32::consts::TAU / wheel.hat.receivers().len() as f32;

        for (idx, person) in wheel.hat.receivers().iter().enumerate() {
            let start_angle = time as f32 + inner_angle * idx as f32;
            let r = smaller_dimension / 2.0 - 5.0;
            shapes.push(wedge(center, r, start_angle, inner_angle, colors[idx % colors.len()], stroke));

            let galley = ui.fonts().layout_no_wrap(person.name.to_string(), FontId::default(), text_color);
            let dir = Vec2::angled(start_angle + inner_angle / 2.0);
            let xoffset = r - galley.rect.width() - 5.0;
            let yoffset = galley.rect.height() / 2.0;

            let text_pos = center +  dir * xoffset + dir.rot90() * yoffset;

            let mut text_shape = TextShape::new(text_pos, 
                galley
            );
            text_shape.angle = start_angle + inner_angle / 2.0;

            shapes.push(Shape::Text(text_shape));
        }

        ui.painter().extend(shapes);

    });
}

fn wedge(center: Pos2, r: f32, start_angle: f32, inner_angle: f32, fill: Color32, stroke: Stroke) -> Shape {
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