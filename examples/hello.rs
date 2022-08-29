#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::cell::RefCell;
use std::rc::Rc;

use eframe::{egui, NativeOptions};
use egui::color_picker::{color_picker_color32, Alpha};
use egui::{Color32, RichText, Slider};

use egui_dock::{DockArea, NodeIndex, Style, TabBuilder, Tree};

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
    style: Rc<RefCell<Style>>,
    tree: Tree,
}

impl Default for MyApp {
    fn default() -> Self {
        let context = Rc::new(RefCell::new(MyContext {
            title: "Hello".to_string(),
            age: 24,
        }));
        let style = Rc::new(RefCell::new(Style::default()));

        let mt_ctx = context.clone();
        let node_tree = TabBuilder::default()
            .title(RichText::new("Simple Demo").color(Color32::BLUE))
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

        let style_clone = style.clone();
        let style_editor = TabBuilder::default()
            .title("Style Editor")
            .content(move |ui| {
                let mut style = style_clone.borrow_mut();
                ui.heading("Style Editor");

                ui.collapsing("Border", |ui| {
                    ui.separator();

                    ui.label("Width");
                    ui.add(Slider::new(&mut style.border_width, 1.0..=50.0));

                    ui.separator();

                    ui.label("Color");
                    color_picker_color32(ui, &mut style.border_color, Alpha::OnlyBlend);
                });

                ui.collapsing("Selection", |ui| {
                    ui.separator();

                    ui.label("Color");
                    color_picker_color32(ui, &mut style.selection_color, Alpha::OnlyBlend);
                });

                ui.collapsing("Separator", |ui| {
                    ui.separator();

                    ui.label("Width");
                    ui.add(Slider::new(&mut style.separator_width, 1.0..=50.0));

                    ui.label("Offset limit");
                    ui.add(Slider::new(&mut style.separator_extra, 1.0..=300.0));

                    ui.separator();

                    ui.label("Color");
                    color_picker_color32(ui, &mut style.separator_color, Alpha::OnlyBlend);
                });

                ui.collapsing("Tab", |ui| {
                    ui.separator();

                    ui.label("Rounding");
                    ui.horizontal(|ui| {
                        ui.add(Slider::new(&mut style.tab_rounding.nw, 0.0..=15.0));
                        ui.label("North-West");
                    });
                    ui.horizontal(|ui| {
                        ui.add(Slider::new(&mut style.tab_rounding.ne, 0.0..=15.0));
                        ui.label("North-East");
                    });
                    ui.horizontal(|ui| {
                        ui.add(Slider::new(&mut style.tab_rounding.sw, 0.0..=15.0));
                        ui.label("South-West");
                    });
                    ui.horizontal(|ui| {
                        ui.add(Slider::new(&mut style.tab_rounding.se, 0.0..=15.0));
                        ui.label("South-East");
                    });

                    ui.separator();

                    ui.label("Bar background color");
                    color_picker_color32(ui, &mut style.tab_bar_background_color, Alpha::OnlyBlend);

                    ui.separator();

                    ui.label("Outline color");
                    color_picker_color32(ui, &mut style.tab_outline_color, Alpha::OnlyBlend);

                    ui.separator();

                    ui.label("Background color");
                    color_picker_color32(ui, &mut style.tab_background_color, Alpha::OnlyBlend);
                });
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

        let mut tree = Tree::new(vec![node_tree, style_editor]);

        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![inspector]);
        let [_, _] = tree.split_below(a, 0.7, vec![files, assets]);
        let [_, _] = tree.split_below(b, 0.5, vec![hierarchy]);

        Self {
            style,
            _context: context,
            tree,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let style = self.style.borrow().clone();
        DockArea::new(&mut self.tree).style(style).show(ctx);
    }
}
