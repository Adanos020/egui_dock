#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};
use egui::{Id, LayerId, Slider, Ui};
use egui_dock::{NodeIndex, Style, TabBuilder, Tree};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

#[derive(Clone)]
struct MyContext {
    pub title: String,
    pub age: u32,
}

struct MyApp {
    _context: Rc<RefCell<MyContext>>,
    style: Style,
    tree: Tree,
}

impl Default for MyApp {
    fn default() -> Self {
        let context = Rc::new(RefCell::new(MyContext {
            title: "Hello".to_string(),
            age: 24,
        }));

        let mt_ctx = context.clone();
        let node_tree = TabBuilder::default()
            .title("Node Tree")
            .content(move |ui| {
                let mut ctx = mt_ctx.borrow_mut();
                ui.heading("My egui Application");
                ui.horizontal(|ui| {
                    ui.label("Your name: ");
                    ui.text_edit_singleline(&mut ctx.title);
                });
                ui.add(Slider::new(&mut ctx.age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    ctx.age += 1;
                }
                ui.label(format!("Hello '{}', age {}", &ctx.title, &ctx.age));
            })
            .build();

        let scene = TabBuilder::default()
            .title("Scene")
            .content(|ui| {
                ui.label("Scene");
            })
            .build();

        let hierarchy = TabBuilder::default()
            .title("Hierarchy")
            .content(|ui| {
                ui.label("Hierarchy");
            })
            .build();

        let inspector = TabBuilder::default()
            .title("Inspector")
            .content(|ui| {
                ui.label("Inspector");
            })
            .build();

        let files = TabBuilder::default()
            .title("File Browser")
            .content(|ui| {
                ui.label("File Browser");
            })
            .build();

        let assets = TabBuilder::default()
            .title("Asset Manager")
            .content(|ui| {
                ui.label("Asset Manager");
            })
            .build();

        let mut tree = Tree::new(vec![node_tree, scene]);

        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![inspector]);
        let [_, _] = tree.split_below(a, 0.7, vec![files, assets]);
        let [_, _] = tree.split_below(b, 0.5, vec![hierarchy]);

        Self {
            style: Style::default(),
            _context: context,
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
