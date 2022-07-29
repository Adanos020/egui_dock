#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{NativeOptions, egui};
use egui::{Frame, Id, LayerId, Ui, style::Margin};
use egui_dock::{NodeIndex, Style, Tab, Tree};

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    context: MyContext,
    style: Style,
    tree: Tree<MyContext>,
}

impl Default for MyApp {
    fn default() -> Self {
        let tab1 = Box::new(MyTab::new("Tab 1"));
        let tab2 = Box::new(MyTab::new("Tab 2"));
        let tab3 = Box::new(MyTab::new("Tab 3"));
        let tab4 = Box::new(MyTab::new("Tab 4"));
        let tab5 = Box::new(MyTab::new("Tab 5"));

        let mut tree = Tree::new(vec![tab1, tab2]);

        // You can modify the tree in runtime
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![tab3]);
        let [_, _] = tree.split_below(a, 0.7, vec![tab4]);
        let [_, _] = tree.split_below(b, 0.5, vec![tab5]);

        Self {
            style: Style::default(),
            context: MyContext,
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
        egui_dock::show(&mut ui, id, &self.style, &mut self.tree, &mut self.context)
    }
}

struct MyContext;

struct MyTab {
    text: String,
}

impl MyTab {
    fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl Tab<MyContext> for MyTab {
    fn title(&self) -> &str {
        &self.text
    }

    fn ui(&mut self, ui: &mut Ui, _ctx: &mut MyContext) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            ui.label(&self.text);
        });
    }
}
