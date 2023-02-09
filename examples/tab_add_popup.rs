#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};

use egui::{Color32, RichText};
use egui_dock::{DockArea, NodeIndex, StyleBuilder, Tree};

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

enum MyTabKind {
    Regular,
    Fancy,
}

struct MyTab {
    kind: MyTabKind,
    node: NodeIndex,
}
impl MyTab {
    fn regular(node_index: usize) -> Self {
        Self {
            kind: MyTabKind::Regular,
            node: NodeIndex(node_index),
        }
    }

    fn fancy(node_index: usize) -> Self {
        Self {
            kind: MyTabKind::Fancy,
            node: NodeIndex(node_index),
        }
    }

    fn title(&self) -> String {
        match self.kind {
            MyTabKind::Regular => format!("Regular Tab {}", self.node.0),
            MyTabKind::Fancy => format!("Fancy Tab {}", self.node.0),
        }
    }

    fn content(&self) -> RichText {
        match self.kind {
            MyTabKind::Regular => {
                RichText::new(format!("Content of {}. This tab is ho-hum.", self.title()))
            }
            MyTabKind::Fancy => RichText::new(format!(
                "Content of {}. This tab sure is fancy!",
                self.title()
            ))
            .italics()
            .size(20.0)
            .color(Color32::from_rgb(255, 128, 64)),
        }
    }
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<MyTab>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = MyTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(tab.content());
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, node: NodeIndex) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;

        if ui.button("Regular tab").clicked() {
            self.added_nodes.push(MyTab::regular(node.0));
        }

        if ui.button("Fancy tab").clicked() {
            self.added_nodes.push(MyTab::fancy(node.0));
        }
    }
}

struct MyApp {
    tree: Tree<MyTab>,
    counter: usize,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = Tree::new(vec![MyTab::regular(1), MyTab::fancy(2)]);

        // You can modify the tree before constructing the dock
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![MyTab::fancy(3)]);
        let [_, _] = tree.split_below(a, 0.7, vec![MyTab::fancy(4)]);
        let [_, _] = tree.split_below(b, 0.5, vec![MyTab::regular(5)]);

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
                    .show_add_popup(true)
                    .build(),
            )
            .show(
                ctx,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                },
            );

        added_nodes.drain(..).for_each(|node| {
            self.tree.set_focused_node(node.node);
            self.tree.push_to_focused_leaf(MyTab {
                kind: node.kind,
                node: NodeIndex(self.counter),
            });
            self.counter += 1;
        });
    }
}
