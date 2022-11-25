#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::HashSet;

use eframe::{egui, NativeOptions};
use egui::{
    color_picker::{color_picker_color32, Alpha},
    CentralPanel, Id, LayerId, Slider, TopBottomPanel, Ui, WidgetText,
};

use egui_dock::{DockArea, Node, NodeIndex, Style, TabViewer, Tree};

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyContext {
    pub title: String,
    pub age: u32,
    pub style: Option<Style>,
    open_tabs: HashSet<String>,
}

struct MyApp {
    context: MyContext,
    tree: Tree<String>,
}

impl TabViewer for MyContext {
    type Tab = String;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Simple Demo" => self.simple_demo(ui),
            "Style Editor" => self.style_editor(ui),
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn context_menu(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Simple Demo" => self.simple_demo_menu(ui),
            _ => {
                ui.label(tab.to_string());
                ui.label("This is a context menu");
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.open_tabs.remove(tab);
        true
    }
}

impl MyContext {
    fn simple_demo_menu(&mut self, ui: &mut Ui) {
        ui.label("Egui widget example");
        ui.menu_button("Sub menu", |ui| {
            ui.label("hello :)");
        });
    }

    fn simple_demo(&mut self, ui: &mut Ui) {
        ui.heading("My egui Application");

        ui.horizontal(|ui| {
            ui.label("Your name: ");
            ui.text_edit_singleline(&mut self.title);
        });
        ui.add(Slider::new(&mut self.age, 0..=120).text("age"));
        if ui.button("Click each year").clicked() {
            self.age += 1;
        }
        ui.label(format!("Hello '{}', age {}", &self.title, &self.age));
    }

    fn style_editor(&mut self, ui: &mut Ui) {
        ui.heading("Style Editor");

        let style = self.style.as_mut().unwrap();

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

            ui.label("Idle color");
            color_picker_color32(ui, &mut style.separator_color_idle, Alpha::OnlyBlend);

            ui.label("Hovered color");
            color_picker_color32(ui, &mut style.separator_color_hovered, Alpha::OnlyBlend);

            ui.label("Dragged color");
            color_picker_color32(ui, &mut style.separator_color_dragged, Alpha::OnlyBlend);
        });

        ui.collapsing("Tabs", |ui| {
            ui.separator();

            ui.checkbox(
                &mut style.tab_hover_name,
                "Show tab name when hoverd over them",
            );
            ui.checkbox(&mut style.tabs_are_draggable, "Tabs are draggable");
            ui.checkbox(&mut style.expand_tabs, "Expand tabs");
            ui.checkbox(&mut style.show_context_menu, "Show context_menu");
            ui.checkbox(
                &mut style.tab_include_scrollarea,
                "Include ScrollArea inside of tabs",
            );

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

            ui.label("Title text color unfocused");
            color_picker_color32(ui, &mut style.tab_text_color_unfocused, Alpha::OnlyBlend);

            ui.label("Title text color focused");
            color_picker_color32(ui, &mut style.tab_text_color_focused, Alpha::OnlyBlend);

            ui.separator();

            ui.checkbox(&mut style.show_close_buttons, "Allow closing tabs");

            ui.separator();

            ui.label("Close button color unfocused");
            color_picker_color32(ui, &mut style.close_tab_color, Alpha::OnlyBlend);

            ui.separator();

            ui.label("Close button color focused");
            color_picker_color32(ui, &mut style.close_tab_active_color, Alpha::OnlyBlend);

            ui.separator();

            ui.label("Close button background color");
            color_picker_color32(ui, &mut style.close_tab_background_color, Alpha::OnlyBlend);

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
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = Tree::new(vec!["Simple Demo".to_owned(), "Style Editor".to_owned()]);
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec!["Inspector".to_owned()]);
        let [_, _] = tree.split_below(
            a,
            0.7,
            vec!["File Browser".to_owned(), "Asset Manager".to_owned()],
        );
        let [_, _] = tree.split_below(b, 0.5, vec!["Hierarchy".to_owned()]);

        let mut open_tabs = HashSet::new();

        for node in tree.iter() {
            match node {
                Node::Leaf { tabs, .. } => {
                    for tab in tabs {
                        open_tabs.insert(tab.clone());
                    }
                }
                _ => (),
            }
        }
        let context = MyContext {
            title: "Hello".to_string(),
            age: 24,
            style: None,
            open_tabs,
        };

        Self { context, tree }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("egui_dock::MenuBar").show(&ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    // allow certain tabs to be toggled
                    for tab in &["File Browser", "Asset Manager"] {
                        if ui
                            .selectable_label(self.context.open_tabs.contains(*tab), *tab)
                            .clicked()
                        {
                            if let Some(index) = self.tree.find_tab(&tab.to_string()) {
                                self.tree.remove_tab(index);
                                self.context.open_tabs.remove(*tab);
                            } else {
                                self.tree.push_to_focused_leaf(tab.to_string());
                            }

                            ui.close_menu();
                        }
                    }
                });
            })
        });

        CentralPanel::default().show(ctx, |_ui| {
            let layer_id = LayerId::background();
            let max_rect = ctx.available_rect();
            let clip_rect = ctx.available_rect();
            let id = Id::new("egui_dock::DockArea");
            let mut ui = Ui::new(ctx.clone(), layer_id, id, max_rect, clip_rect);

            let style = self
                .context
                .style
                .get_or_insert(Style::from_egui(&ui.ctx().style()))
                .clone();
            DockArea::new(&mut self.tree)
                .style(style)
                .show_inside(&mut ui, &mut self.context);
        });
    }
}
