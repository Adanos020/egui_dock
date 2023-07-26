#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::BTreeMap;

use eframe::{egui, NativeOptions};

/// We identify tabs by the title of the file we are editing.
type Title = String;

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "Text editor examples",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct Buffers {
    buffers: BTreeMap<Title, String>,
}

impl egui_dock::TabViewer for Buffers {
    type Tab = Title;

    fn ui(&mut self, ui: &mut egui::Ui, title: &mut Title) {
        let text = self.buffers.entry(title.clone()).or_default();
        egui::TextEdit::multiline(text)
            .desired_width(f32::INFINITY)
            .show(ui);
    }

    fn title(&mut self, title: &mut Title) -> egui::WidgetText {
        egui::WidgetText::from(&*title)
    }
}

struct MyApp {
    buffers: Buffers,
    tree: egui_dock::DockState<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut buffers = BTreeMap::default();
        buffers.insert(
            "CHANGELOG.md".to_owned(),
            include_str!("../CHANGELOG.md").to_owned(),
        );
        buffers.insert("LICENSE".to_owned(), include_str!("../LICENSE").to_owned());
        buffers.insert(
            "README.md".to_owned(),
            include_str!("../README.md").to_owned(),
        );

        let tree =
            egui_dock::DockState::new(vec!["README.md".to_owned(), "CHANGELOG.md".to_owned()]);

        Self {
            buffers: Buffers { buffers },
            tree,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("documents").show(ctx, |ui| {
            for title in self.buffers.buffers.keys() {
                let tab_location = self.tree.find_tab(title);
                let is_open = tab_location.is_some();
                if ui.selectable_label(is_open, title).clicked() {
                    if let Some((node_index, tab_index)) = tab_location {
                        self.tree.set_active_tab(node_index, tab_index);
                    } else {
                        // Open the file for editing:
                        self.tree.push_to_focused_leaf(title.clone());
                    }
                }
            }
        });

        egui_dock::DockArea::new(&mut self.tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.buffers);
    }
}
