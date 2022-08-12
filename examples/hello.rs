#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};
use egui::{style::Margin, Frame, Id, LayerId, Slider, Ui};
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
        let node_tree = Box::new(PlaceholderTab::new("Node Tree"));
        let scene = Box::new(PlaceholderTab::new("Scene"));

        let hierarchy = Box::new(PlaceholderTab::new("Hierarchy"));
        let inspector = Box::new(PlaceholderTab::new("Inspector"));

        let files = Box::new(PlaceholderTab::new("File Browser"));
        let assets = Box::new(PlaceholderTab::new("Asset Manager"));

        let mut tree = Tree::new(vec![scene, node_tree]);

        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![inspector]);
        let [_, _] = tree.split_below(a, 0.7, vec![files, assets]);
        let [_, _] = tree.split_below(b, 0.5, vec![hierarchy]);

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

struct PlaceholderTab {
    title: String,

    age: u32,
}

impl PlaceholderTab {
    fn new(title: impl ToString) -> Self {
        Self {
            title: title.to_string(),
            age: 42,
        }
    }
}

impl Tab<MyContext> for PlaceholderTab {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui, _ctx: &mut MyContext) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.title);
            });
            ui.add(Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.title, self.age));
        });
    }
}
