//! UMC Desktop application — global state + navigation.

use eframe::egui;
use egui::{Color32, RichText};

use crate::history::History;
use crate::screens::{convert, formats, history, settings};

/// Available screens in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Convert,
    History,
    Formats,
    Settings,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Screen::Convert => "Convert",
            Screen::History => "History",
            Screen::Formats => "Formats",
            Screen::Settings => "Settings",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Screen::Convert => "⇄",
            Screen::History => "🕘",
            Screen::Formats => "▤",
            Screen::Settings => "⚙",
        }
    }
}

/// Global application state.
pub struct UmcApp {
    pub screen: Screen,
    pub history: History,
    pub convert: convert::ConvertState,
    pub settings: settings::SettingsState,
    pub theme_dark: bool,
}

impl UmcApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = settings::SettingsState::load();
        let theme_dark = settings.theme_dark;

        if theme_dark {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        }

        Self {
            screen: Screen::Convert,
            history: History::load(),
            convert: convert::ConvertState::default(),
            settings,
            theme_dark,
        }
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.theme_dark {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }
}

impl eframe::App for UmcApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);

        // ── Sidebar ─────────────────────────────────────────────────────────
        egui::Panel::left("sidebar")
            .exact_size(190.0)
            .resizable(false)
            .show(ui, |ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("UMC")
                            .size(26.0)
                            .strong()
                            .color(Color32::from_rgb(0x4f, 0x9d, 0xe9)),
                    );
                    ui.label(RichText::new("Universal Model Converter").size(11.0).weak());
                });
                ui.add_space(16.0);

                for screen in [
                    Screen::Convert,
                    Screen::History,
                    Screen::Formats,
                    Screen::Settings,
                ] {
                    let selected = self.screen == screen;
                    let text = format!("{}  {}", screen.icon(), screen.title());
                    let button = egui::Button::new(RichText::new(text).size(15.0))
                        .fill(if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .corner_radius(6.0)
                        .min_size(egui::vec2(0.0, 34.0));

                    if ui.add_sized([ui.available_width(), 34.0], button).clicked() {
                        self.screen = screen;
                    }
                    ui.add_space(2.0);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("v{}", crate::worker::version()))
                            .size(10.0)
                            .weak(),
                    );
                });
            });

        // ── Active screen ───────────────────────────────────────────────────
        match self.screen {
            Screen::Convert => convert::show(ui, &mut self.convert, &mut self.history),
            Screen::History => history::show(ui, &mut self.history),
            Screen::Formats => formats::show(ui),
            Screen::Settings => settings::show(ui, &mut self.settings, &mut self.theme_dark),
        }
    }
}
