use crate::hat::Person;

mod wheel;
pub(crate) use wheel::*;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum Page {
    People,
    Wheel,
    About,
}

impl Default for Page {
    fn default() -> Self {
        Page::About
    }
}

pub(crate) fn display_about(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::warn_if_debug_build(ui);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
        });
    });
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub(crate) struct PeoplePage {
    pub(crate) person: Person,
}

pub(crate) fn dipslay_people(page: &mut PeoplePage, people: &mut Vec<Person>, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("new-person").show(ctx, |ui| {
        ui.edit_person(&mut page.person);

        if ui.button("Add").clicked() {
            let person = std::mem::replace(&mut page.person, Person::new("", ""));
            people.push(person);
        }
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut to_swap = None;
                let mut to_remove = None;
                let count = people.len();
                for (idx, person) in people.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.person(person);
                            if idx > 0 && ui.button("/\\").clicked() {
                                to_swap = Some((idx - 1, idx));
                            }
                            if idx < count - 1 && ui.button("\\/").clicked() {
                                to_swap = Some((idx, idx+1))
                            }
                            if ui.button("X").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    });
                }

                if let Some((first, second)) = to_swap {
                    people.swap(first, second);
                }
                if let Some(index) = to_remove {
                    people.remove(index);
                }
            });
    });
}

trait UiExtensions {
    fn person(&mut self, person: &Person);
    fn edit_person(&mut self, person: &mut Person);
}
impl UiExtensions for egui::Ui {
    fn person(&mut self, person: &Person) {
        self.horizontal(|ui| {
            ui.label(&person.name);
            ui.label("-");
            ui.label(&person.group);
        });
    }

    fn edit_person(&mut self, person: &mut Person) {
        self.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut person.name);
        });

        self.horizontal(|ui| {
            ui.label("Group:");
            ui.text_edit_singleline(&mut person.group);
        });
    }
}
