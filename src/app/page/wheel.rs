mod spinner;

use egui::{Color32, RichText};

use crate::hat::{DrawError, Hat, Pair, Person};
use crate::valid_pair;

use self::spinner::{Spinner, SpinnerTarget};

use super::UiExtensions;
const SPIN_TIME: f32 = 5.0;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
enum WheelState {
    Idle,
    Windup(Pair),
    HoldAtTopSpeed { pair: Pair, start_time: f32 },
    SlowToStop { pair: Pair },
    Stopped { pair: Pair },
}
impl WheelState {
    fn try_transition(&mut self, spinner: &mut Spinner, time: f32) {
        *self = match std::mem::take(self) {
            WheelState::Idle => WheelState::Idle,
            WheelState::Windup(pair) => {
                if spinner.speed() + 0.05 >= spinner::FULL_SPEED {
                    WheelState::HoldAtTopSpeed {
                        pair,
                        start_time: time,
                    }
                } else {
                    WheelState::Windup(pair)
                }
            }
            WheelState::HoldAtTopSpeed { pair, start_time } => {
                if time - start_time > SPIN_TIME {
                    //TODO figure out which item should be selected when the spinner stops
                    spinner.target = SpinnerTarget::Speed(0.0);
                    WheelState::SlowToStop { pair }
                } else {
                    WheelState::HoldAtTopSpeed {
                        pair,
                        start_time,
                    }
                }
            }
            WheelState::SlowToStop { pair } => {
                if spinner.speed() <= 0.05 {
                    WheelState::Stopped { pair }
                } else {
                    WheelState::SlowToStop { pair }
                }
            }
            WheelState::Stopped { pair } => WheelState::Stopped { pair },
        };
    }
}

impl Default for WheelState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub(crate) struct WheelPage {
    hat: Hat,
    state: WheelState,
    drawn_names: Vec<Pair>,
    error_message: Option<String>,
    spinner: Spinner,
}

impl WheelPage {
    fn reset(&mut self, people: &[Person]) {
        self.state = WheelState::Idle;
        self.spinner.target = SpinnerTarget::Speed(spinner::IDLE_SPEED);
        self.spinner.items = people.to_vec();
        self.error_message = None;
        self.hat = Hat::with_people(people.into());
        self.drawn_names.clear();
    }

    fn spin(&mut self) {
        match self.state {
            WheelState::Idle => (),
            _ => panic!("WheelPage::spin called in wrong state"),
        }
        self.error_message = None;
        match self.hat.draw_name(valid_pair) {
            Ok(pair) => {
                self.spinner.target = SpinnerTarget::Speed(spinner::FULL_SPEED);
                self.state = WheelState::Windup(pair);
            }
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

        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_animation(ui);
            self.spinner.render(ui);
        });
    }

    fn update_animation(&mut self, ui: &egui::Ui) {
        let delta_time = ui.input().stable_dt.min(0.1);
        let time = ui.input().time as f32;

        self.state.try_transition(&mut self.spinner, time);

        self.spinner.step_animation(delta_time);
    }

    fn add_result(&mut self, pair: Pair) {
        self.spinner.items.retain(|p| p != &pair.receiver);

        self.drawn_names.push(pair);

        self.spinner.target = SpinnerTarget::Speed(spinner::IDLE_SPEED);
        self.state = WheelState::Idle;
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
    match &wheel.state {
        WheelState::Idle => {
            if !wheel.hat.givers().is_empty() {
                if ui.button(RichText::new("Spin Wheel").heading()).clicked() {
                    wheel.spin();
                }
            } else {
                ui.label(RichText::new("All names drawn").heading());
            }
        }
        WheelState::Windup(pair)
        | WheelState::HoldAtTopSpeed { pair, .. }
        | WheelState::SlowToStop { pair, .. } => {
            ui.label(RichText::new(format!("Spinning for {}", pair.giver.name)).heading());
        }
        WheelState::Stopped { pair } => {
            let pair = pair.clone();
            ui.horizontal(|ui| {
                ui.heading(format!(
                    "{} is giving to {}",
                    pair.giver.name, pair.receiver.name
                ));
                if ui.button(RichText::new("Next Spin").heading()).clicked() {
                    wheel.add_result(pair)
                }
            });
        }
    }
    if let Some(msg) = &wheel.error_message {
        ui.colored_label(Color32::RED, msg);
    }

    if ui.button("Restart").clicked() {
        wheel.reset(people);
    }
}
