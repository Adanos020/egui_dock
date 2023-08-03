//! # `egui_dock`: docking support for `egui`
//!
//! Originally created by [@lain-dono](https://github.com/lain-dono), this library provides docking support for `egui`.
//! It lets you open and close tabs, freely move them around, insert them in selected parts of the [`DockArea`], and resize them.
//!
//! ## Usage
//!
//! The library is centered around the [`DockState`].
//! It contains a series of [`Surface`]s which all have their own [`Tree`]
//! each [`Tree`] stores a layout of [`Node`]s which then contains tabs.
//!
//! [`DockState`] is generic (`DockState<Tab>`) so you can use any data to represent a tab.
//! You show the tabs using [`DockArea`] and specify how they are shown
//! by implementing [`TabViewer`].
//!
//! ```rust
//! use egui_dock::{NodeIndex, Style, DockState};
//!
//! struct MyTabs {
//!     tree: DockState<String>
//! }
//!
//! impl MyTabs {
//!     pub fn new() -> Self {
//!         let tab1 = "tab1".to_string();
//!         let tab2 = "tab2".to_string();
//!
//!         let mut tree = DockState::new(vec![tab1]);
//!         tree.root_split_left(NodeIndex::root(), 0.20, vec![tab2]);
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

#[allow(deprecated)]
pub use dock_state::*;
pub use egui;
pub use style::*;
pub use tree::*;
pub use widgets::*;

/// The main Structure of the library.
pub mod dock_state;

/// egui_dock theme (color, sizes...).
pub mod style;

/// Widgets provided by the library.
pub mod widgets;

mod utils;
