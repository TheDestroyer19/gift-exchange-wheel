use egui::{Align, Color32, Layout, RichText};

use crate::hat::{DrawError, Hat, Pair, Person};
use crate::valid_pair;

use super::UiExtensions;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(crate) struct WheelPage {
    hat: Hat,
    drawn_names: Vec<Pair>,
    error_message: Option<String>,
}

pub(crate) fn display_wheel(wheel: &mut WheelPage, people: &[Person], ctx: &egui::Context) {
    egui::SidePanel::left("wheel-left").show(ctx, |ui| side_panel(ui, wheel));

    egui::TopBottomPanel::bottom("wheel-bottom").show(ctx, |ui| bottom_panel(ui, wheel, people));
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
                wheel.error_message = None;
                match wheel.hat.draw_name(valid_pair) {
                    Ok(pair) => wheel.drawn_names.push(pair),
                    Err(DrawError::NoGivers) => {
                        wheel.error_message = Some("No one left to assign".into())
                    }
                    //This case needs to have some 'just draw someone' option
                    Err(DrawError::NoValidReceiver) => {
                        wheel.error_message = Some("It isn't possible to assign everyone".into())
                    }
                }
            }
        }

        if let Some(msg) = &wheel.error_message {
            ui.colored_label(Color32::RED, msg);
        }

        if ui.button("Restart").clicked() {
            wheel.error_message = None;
            wheel.hat = Hat::with_people(people.into());
            wheel.drawn_names.clear();
        }
    });
}
