#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};

use egui_dock::{DockArea, NodeIndex, StyleBuilder, Tree};

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<NodeIndex>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = usize;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of tab {tab}"));
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("Tab {tab}").into()
    }

    fn on_add(&mut self, node: NodeIndex) {
        self.added_nodes.push(node);
    }
}

struct MyApp {
    tree: Tree<usize>,
    counter: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = Tree::new(vec![1, 2]);

        // You can modify the tree before constructing the dock
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![3]);
        let [_, _] = tree.split_below(a, 0.7, vec![4]);
        let [_, _] = tree.split_below(b, 0.5, vec![5]);

        Self { tree, counter: 6 }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut added_nodes = Vec::new();
        DockArea::new(&mut self.tree)
            .style(
                StyleBuilder::from_egui(ctx.style().as_ref())
                    .show_add_buttons(true)
                    .expand_tabs(true)
                    .build(),
            )
            .show(
                ctx,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                },
            );

        added_nodes.drain(..).for_each(|node| {
            self.tree.set_focused_node(node);
            self.tree.push_to_focused_leaf(self.counter);
            self.counter += 1;
        });
    }
}
