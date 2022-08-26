#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, NativeOptions};
use egui::{
    style::Margin, text::LayoutJob, Align, Color32, FontId, Frame, Id, LayerId, TextFormat, Ui,
    Window,
};
use egui_dock::{DockArea, NodeIndex, Style, Tab, TabBuilder, Tree};

fn main() {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

struct MyApp {
    style: Style,
    dock: DockArea,
}

impl Default for MyApp {
    fn default() -> Self {
        let tab1 = Box::new(Editor::new("Text".into()));

        let tab2 = TabBuilder::default()
            .title("Tab 2")
            .content(|ui| {
                ui.label("Tab 2");
            })
            .build();
        let tab3 = TabBuilder::default()
            .title("Tab 3")
            .content(|ui| {
                ui.label("Tab 3");
            })
            .build();
        let tab4 = TabBuilder::default()
            .title("Tab 4")
            .content(|ui| {
                ui.label("Tab 4");
            })
            .build();
        let tab5 = TabBuilder::default()
            .title("Tab 5")
            .content(|ui| {
                ui.label("Tab 5");
            })
            .build();

        let mut tree = Tree::new(vec![tab1, tab2]);

        // You can modify the tree before constructing the dock
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![tab3]);
        let [_, _] = tree.split_below(a, 0.7, vec![tab4]);
        let [_, _] = tree.split_below(b, 0.5, vec![tab5]);

        Self {
            style: Style::default(),
            dock: DockArea::from_tree(tree),
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
        Frame::none()
            .inner_margin(Margin::same(2.0))
            .show(&mut ui, |ui| {
                if ui.button("Add Editor").clicked() {
                    self.dock
                        .push_to_active_leaf(Editor::new("New Text".into()));
                }
            });
        ui.separator();
        Frame::none().show(&mut ui, |ui| self.dock.show(ui, id, &self.style));
    }
}

struct Editor {
    name: String,
    modified: bool,
    text: String,
    show_save: bool,
    exit: bool,
}

impl Editor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            modified: false,
            text: "Important text to edit".into(),
            show_save: false,
            exit: false,
        }
    }

    fn save(&mut self) {
        self.modified = false;
        //save text to file or someplace else
    }
}

impl Tab for Editor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        if self.show_save {
            Window::new("Save")
                .collapsible(false)
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "You have unsaved work on {} would you like to save",
                            self.name
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                self.save();
                                self.exit = true;
                                self.show_save = false;
                            }
                            if ui.button("Don't Save").clicked() {
                                self.exit = true;
                                self.show_save = false;
                            }
                            if ui.button("Cancel").clicked() {
                                self.exit = false;
                                self.show_save = false;
                            }
                        });
                    });
                });
        }
        Frame::none()
            .inner_margin(Margin::same(2.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save();
                        }
                    });
                    if ui.code_editor(&mut self.text).changed() {
                        self.modified = true;
                    }
                });
            });
    }

    fn title(&mut self) -> egui::WidgetText {
        if self.modified {
            let mut job = LayoutJob::default();
            job.append(
                self.name.as_str(),
                0.0,
                TextFormat::simple(FontId::default(), Color32::from_rgb(245, 245, 67)),
            );

            job.append(
                " M",
                0.0,
                TextFormat {
                    font_id: FontId::proportional(FontId::default().size / 1.5),
                    color: Color32::from_rgb(245, 245, 67),
                    valign: Align::Min,
                    ..Default::default()
                },
            );

            job.into()
        } else {
            self.name.clone().into()
        }
    }

    fn force_close(&mut self) -> bool {
        self.exit
    }

    fn on_close(&mut self) -> bool {
        self.show_save = true;
        self.exit || !self.modified
    }
}
