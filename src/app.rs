use crate::hat::Person;

use self::page::{Page, PeoplePage, WheelPage};

mod page;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GiftExchangeApp {
    people: Vec<Person>,
    page: Page,
    people_page: PeoplePage,
    wheel_page: WheelPage,
}

impl GiftExchangeApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for GiftExchangeApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Gift Exchange Wheel");

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.page, Page::People, "People");
                ui.selectable_value(&mut self.page, Page::Wheel, "Wheel");
                ui.selectable_value(&mut self.page, Page::About, "About");
            });
        });

        match self.page {
            Page::People => page::dipslay_people(&mut self.people_page, &mut self.people, ctx),
            Page::Wheel => self.wheel_page.display(&self.people, ctx),
            Page::About => page::display_about(ctx),
        }
    }
}
