#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};

use egui_dock::{DockArea, NodeIndex, Style, Tree};

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    );
}

struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}

struct MyApp {
    tree: Tree<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = Tree::new(vec!["tab1".to_owned(), "tab2".to_owned()]);

        // You can modify the tree before constructing the dock
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec!["tab3".to_owned()]);
        let [_, _] = tree.split_below(a, 0.7, vec!["tab4".to_owned()]);
        let [_, _] = tree.split_below(b, 0.5, vec!["tab5".to_owned()]);

        Self { tree }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer {});
    }
}
