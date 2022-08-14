#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};
use egui::{Id, LayerId, Ui};
use egui_dock::{NodeIndex, Style, TabBuilder, Tree};

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    style: Style,
    tree: Tree,
}

impl Default for MyApp {
    fn default() -> Self {
        let tab1 = TabBuilder::default()
            .title("Tab 1")
            .content(|ui| {
                ui.label("Tab 1");
            })
            .build();
        let tab2 = TabBuilder::default()
            .title("Tab 2")
            .content(|ui| {
                ui.label("Tab 2");
            })
            .build();
        let tab3 = TabBuilder::default()
            .title("Tab 3")
            .content(|ui| {
                ui.label("Tab 3");
            })
            .build();
        let tab4 = TabBuilder::default()
            .title("Tab 4")
            .content(|ui| {
                ui.label("Tab 4");
            })
            .build();
        let tab5 = TabBuilder::default()
            .title("Tab 5")
            .content(|ui| {
                ui.label("Tab 5");
            })
            .build();

        let mut tree = Tree::new(vec![tab1, tab2]);

        // You can modify the tree in runtime
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![tab3]);
        let [_, _] = tree.split_below(a, 0.7, vec![tab4]);
        let [_, _] = tree.split_below(b, 0.5, vec![tab5]);

        Self {
            style: Style::default(),
            tree,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.style = Style::from_egui(ctx.style().as_ref());

        let id = Id::new("some hashable string");
        let layer_id = LayerId::background();
        let max_rect = ctx.available_rect();
        let clip_rect = ctx.available_rect();

        let mut ui = Ui::new(ctx.clone(), layer_id, id, max_rect, clip_rect);
        egui_dock::show(&mut ui, id, &self.style, &mut self.tree)
    }
}
