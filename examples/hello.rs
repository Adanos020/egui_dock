#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    context: MyContext,
    style: egui_docking::Style,
    tree: egui_docking::Tree<MyContext>,
}

impl Default for MyApp {
    fn default() -> Self {
        use egui_docking::{NodeIndex, Split};
        let node_tree = Box::new(PlaceholderTab::new("Node Tree"));
        let scene = Box::new(PlaceholderTab::new("Scene"));

        let hierarchy = Box::new(PlaceholderTab::new("Hierarchy"));
        let inspector = Box::new(PlaceholderTab::new("Inspector"));

        let files = Box::new(PlaceholderTab::new("File Browser"));
        let assets = Box::new(PlaceholderTab::new("Asset Manager"));

        let root = egui_docking::Node::leaf_with(vec![scene, node_tree]);
        let mut tree = egui_docking::Tree::new(root);

        let [a, b] = tree.split_tabs(NodeIndex::root(), Split::Left, 0.3, vec![inspector]);
        let [_, _] = tree.split_tabs(a, Split::Below, 0.7, vec![files, assets]);
        let [_, _] = tree.split_tabs(b, Split::Below, 0.5, vec![hierarchy]);

        Self {
            style: egui_docking::Style::default(),
            context: MyContext,
            tree,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let id = egui::Id::new("some hashable string");
        let layer_id = egui::LayerId::background();
        let max_rect = ctx.available_rect();
        let clip_rect = ctx.available_rect();

        let mut ui = egui::Ui::new(ctx.clone(), layer_id, id, max_rect, clip_rect);
        egui_docking::show(&mut ui, id, &self.style, &mut self.tree, &mut self.context)
    }
}

struct MyContext;

struct PlaceholderTab {
    title: String,
}

impl PlaceholderTab {
    fn new(title: impl ToString) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

impl egui_docking::Tab<MyContext> for PlaceholderTab {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut MyContext) {}
}
