#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};

use egui_dock::{DockArea, DockState, NodeIndex, Style};

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Clone)]
enum Tab {
    Stay,
    GoAway,
}

struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Stay => "Stay".into(),
            Tab::GoAway => "GoAway".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Stay => ui.label("This tab will stay"),
            Tab::GoAway => ui.label("This hab will go away"),
        };
    }
}

struct MyApp {
    tree: DockState<Tab>,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = DockState::new(vec![Tab::Stay]);
        let [_left, _right] =
            tree.main_surface_mut()
                .split_left(NodeIndex::root(), 0.5, vec![Tab::GoAway]);
        Self { tree }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
            if ui.button("Filter tabs").clicked() {
                //self.tree = self.tree.filter_tabs(|tab| matches!(tab, Tab::Stay));
                self.tree.retain_tabs(|tab| matches!(tab, Tab::Stay));
            }
        });
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut TabViewer {});
    }
}
