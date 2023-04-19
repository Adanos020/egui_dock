//! # `egui_dock`: docking support for `egui`
//!
//! Originally created by [@lain-dono](https://github.com/lain-dono), this library provides docking support for `egui`.
//! It lets you open and close tabs, freely move them around, insert them in selected parts of the [`DockArea`], and resize them.
//!
//! ## Usage
//!
//! The library is centered around the [`Tree`].
//! It stores the layout of [`Node`]s which contains tabs.
//!
//! [`Tree`] is generic (`Tree<Tab>`) so you can use any data to represent a tab.
//! You show the tabs using [`DockArea`] and specify how they are shown
//! by implementing [`TabViewer`].
//!
//! ```rust
//! use egui_dock::{NodeIndex, Style, Tree};
//!
//! struct MyTabs {
//!     tree: Tree<String>
//! }
//!
//! impl MyTabs {
//!     pub fn new() -> Self {
//!         let tab1 = "tab1".to_string();
//!         let tab2 = "tab2".to_string();
//!
//!         let mut tree = Tree::new(vec![tab1]);
//!         tree.split_left(NodeIndex::root(), 0.20, vec![tab2]);
//!
//!         Self { tree }
//!     }
//!
//!     fn ui(&mut self, ui: &mut egui::Ui) {
//!         egui_dock::DockArea::new(&mut self.tree)
//!             .style(Style::from_egui(ui.style().as_ref()))
//!             .show_inside(ui, &mut TabViewer {});
//!     }
//! }
//!
//! struct TabViewer;
//!
//! impl egui_dock::TabViewer for TabViewer {
//!     type Tab = String;
//!
//!     fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
//!         ui.label(format!("Content of {tab}"));
//!     }
//!
//!     fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
//!         (&*tab).into()
//!     }
//! }
//!
//! # let mut my_tabs = MyTabs::new();
//! # egui::__run_test_ctx(|ctx| {
//! #     egui::CentralPanel::default().show(ctx, |ui| my_tabs.ui(ui));
//! # });
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub use egui;
#[allow(deprecated)]
pub use style::*;
pub use tree::*;
pub use widgets::{dock_area::DockArea, tab_viewer::TabViewer};

/// egui_dock theme (color, sizes...).
pub mod style;
pub mod tree;
mod utils;

/// Widgets provided by the library.
pub mod widgets;
