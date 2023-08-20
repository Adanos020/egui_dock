//! # `egui_dock`: docking support for `egui`
//!
//! Originally created by [@lain-dono](https://github.com/lain-dono), this library provides docking support for `egui`.
//! It lets you open and close tabs, freely move them around, resize them, and undock them into new egui windows which
//! can also have other tabs docked in them.
//!
//! ## Usage
//!
//! The library is centered around the [`DockState`].
//! It contains a series of [`Surface`]s which all have their own [`Tree`].
//! Each [`Tree`] stores a hierarchy of [`Node`]s which then contain splits and tabs.
//!
//! [`DockState`] is generic (`DockState<Tab>`) so you can use any data to represent a tab.
//! You show the tabs using [`DockArea`] and specify how they are shown by implementing [`TabViewer`].
//!
//! ```rust
//! use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
//! use egui::{Ui, WidgetText};
//!
//! // First, let's pick a type that will be used to attach some data to each tab.
//! // It can be literally any type.
//! type Tab = String;
//!
//! // To define the contents and properties of individual tabs, we need to implement the `TabViewer` trait.
//! // Only three things are mandatory: the `Tab` associated type, and the `ui` and `title` methods.
//! // There are more methods in `TabViewer` which you can also override.
//! struct MyTabViewer;
//!
//! impl TabViewer for MyTabViewer {
//!     // This associated type is used to attach some data to each tab.
//!     type Tab = Tab;
//!
//!     // In this method, we define the contents of a given `tab`.
//!     fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
//!         ui.label(format!("Content of {tab}"));
//!     }
//!
//!     // This method returns the current `tab`'s title.
//!     fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
//!         tab.as_str().into()
//!     }
//! }
//!
//! // Here is a simple example of how you can manage a `DockState` of your application.
//! struct MyTabs {
//!     dock_state: DockState<Tab>
//! }
//!
//! impl MyTabs {
//!     pub fn new() -> Self {
//!         // Create a `DockState` with an initial tab "tab1" in the main `Surface`'s root node.
//!         let mut dock_state = DockState::new(vec!["tab1".to_string()]);
//!
//!         // Currently, the `DockState` only has one Surface: the main one.
//!         // Let's get mutable access to add more nodes in it.
//!         let surface = dock_state.main_surface_mut();
//!
//!         // Insert "tab2" to the left of "tab1", where the width of "tab2" is 20% of root node's width.
//!         let [_old_node, new_node] = surface.split_left(NodeIndex::root(), 0.20, vec!["tab2".to_string()]);
//!
//!         // Insert "tab3" below "tab2" with both tabs having equal size.
//!         surface.split_below(new_node, 0.5, vec!["tab3".to_string()]);
//!
//!         // The layout will look similar to this:
//!         // +------+------------------------+
//!         // |      |                        |
//!         // | tab2 |                        |
//!         // |      |                        |
//!         // +------+          tab1          |
//!         // |      |                        |
//!         // | tab3 |                        |
//!         // |      |                        |
//!         // +------+------------------------+
//!
//!         Self { dock_state }
//!     }
//!
//!     fn ui(&mut self, ui: &mut Ui) {
//!         // Here we just display the `DockState` using a `DockArea`.
//!         // This is where egui handles rendering and all the integrations.
//!         //
//!         // We can specify a custom `Style` for the `DockArea`, or just inherit all of it from egui.
//!         DockArea::new(&mut self.dock_state)
//!             .style(Style::from_egui(ui.style().as_ref()))
//!             .show_inside(ui, &mut MyTabViewer);
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
