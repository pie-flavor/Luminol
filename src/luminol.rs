// Copyright (C) 2022 Lily Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.

use crate::lumi::Lumi;
use crate::prelude::*;

/// The main Luminol struct. Handles rendering, GUI state, that sort of thing.
pub struct Luminol {
    top_bar: TopBar,
    style: Arc<egui::Style>,
    lumi: Lumi,
}

impl Luminol {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        try_load_path: Option<std::ffi::OsString>,
    ) -> Self {
        let storage = cc.storage.unwrap();

        let state = eframe::get_value(storage, "SavedState").unwrap_or_default();
        let style =
            eframe::get_value(storage, "EguiStyle").map_or_else(|| cc.egui_ctx.style(), |s| s);
        cc.egui_ctx.set_style(style.clone());

        let info = State::new(cc.gl.as_ref().unwrap().clone(), state);
        crate::set_state(info);

        if let Some(path) = try_load_path {
            state!()
                .filesystem
                .try_open_project(path)
                .expect("failed to load project");
        }

        let lumi = Lumi::new().expect("failed to load lumi images");

        Self {
            top_bar: TopBar::default(),
            style,
            lumi,
        }
    }
}

impl eframe::App for Luminol {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value::<crate::SavedState>(
            storage,
            "SavedState",
            &state!().saved_state.borrow(),
        );
        eframe::set_value::<Arc<egui::Style>>(storage, "EguiStyle", &self.style);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if let Some(f) = i.raw.dropped_files.first() {
                let path = f.path.clone().expect("dropped file has no path");

                if let Err(e) = state!().filesystem.try_open_project(path) {
                    state!()
                        .toasts
                        .error(format!("Error opening dropped project: {e}"))
                }
            }
        });

        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar.ui(ui, &mut self.style, frame);
            });
        });

        // Central panel with tabs.
        egui::CentralPanel::default().show(ctx, |ui| {
            state!().tabs.ui(ui);
        });

        // Update all windows.
        state!().windows.update(ctx);

        // Show toasts.
        state!().toasts.show(ctx);

        poll_promise::tick_local();

        self.lumi.ui(ctx);
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}
