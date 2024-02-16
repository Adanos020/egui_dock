# egui_dock changelog

## 0.11.2 - 2024-02-16

### Fixed
From [#225](https://github.com/Adanos020/egui_dock/pull/225):
- Tabs now always appear at the pointer position while being dragged.
- Retaining tabs no longer breaks the binary tree leading to a panic.
- Filtering tabs no longer leaves some leaves empty and now correctly rearranges the tree.

## 0.11.1 - 2024-02-09

### Fixed
- Bug where tabs couldn't be re-docked onto the main surface if it's empty ([#222](https://github.com/Adanos020/egui_dock/pull/222))

## 0.11.0 - 2024-02-06

### Added
- `filter_map_tabs`, `filter_tabs`, and `retain_tabs` ([#217](https://github.com/Adanos020/egui_dock/pull/217))

### Breaking changes
- Upgraded to egui 0.26

## 0.10.0 - 2024-01-09

### Added
- From ([#211](https://github.com/Adanos020/egui_dock/pull/211)):
  - Tabs, the close tab buttons and the add tab buttons are now focusable with the keyboard and interactable with the enter key and space bar.
  - Separators are now focusable with the keyboard and movable using the arrow keys while control or shift is held.
  - `TabStyle::active_with_kb_focus`, `TabStyle::inactive_with_kb_focus` and `TabStyle::focused_with_kb_focus` for style of tabs that are focused with the keyboard.
- Missing translation for the tooltip showing when you hover on a grayed out window close button. ([#216](https://github.com/Adanos020/egui_dock/pull/216))

### Fixed
- Widgets inside tabs are now focusable with the tab key on the keyboard. ([#211](https://github.com/Adanos020/egui_dock/pull/211))

### Breaking changes
- Upgraded to egui 0.25
- Replaced `Default` implementations for `{TabContextMenu,Window,}Translations` with associated functions called `english`. ([#216](https://github.com/Adanos020/egui_dock/pull/216))

## 0.9.1 - 2023-12-10

### Fixed
- Fix crash after calling `DockState::remove_tab`. ([#208](https://github.com/Adanos020/egui_dock/pull/208))

## 0.9.0 - 2023-11-23

### Added
- `DockArea::surfaces_count`
- `DockArea::iter_surfaces[_mut]`
- `DockArea::iter_all_tabs[_mut]`
- `DockArea::iter_all_nodes[_mut]`
- `Node::iter_tabs[_mut]`
- `Surface::iter_nodes[_mut]`
- `Surface::iter_all_tabs[_mut]`

### Breaking changes
- Upgraded to egui 0.24.
- Removed the deprecated `DockState::iter`.

### Deprecated
- `DockState::iter_nodes` – use `iter_all_nodes` instead.
- `DockState::iter_main_surface_nodes[_mut]` – use `dock_state.main_surface().iter()` (and corresponding `mut` versions) instead.

## 0.8.2 - 2023-11-02

### Fixed
- Deserializing `WindowState` no longer crashes when `screen_rect` contains any `f32::INFINITY` values. Make sure to fix your last serialized app state by setting `screen_rect: null`. ([#198](https://github.com/Adanos020/egui_dock/pull/198))

## 0.8.1 - 2023-10-04

### Fixed
- The tab bar no longer remains empty after it ends up having 0 width in any way. ([#191](https://github.com/Adanos020/egui_dock/pull/191))

## 0.8.0 - 2023-09-28

### Breaking changes
- Upgraded `egui` to version 0.23.
- Updated MSRV to Rust 1.70.

### Improvements
- Revised documentation for `TabViewer`.

## 0.7.3 - 2023-09-22

### Fixed

- The "Eject" button is not available on tabs which are disallowed in windows. ([#188](https://github.com/Adanos020/egui_dock/pull/188))

## 0.7.2 - 2023-09-20

### Fixed

- `TabViewer::clear_background` now works as intended. ([#185](https://github.com/Adanos020/egui_dock/pull/185))

## 0.7.1 - 2023-09-18

### Fixed

- (Breaking) Renamed `OverlayStyle::selection_storke_width` to `OverlayStyle::selection_stroke_width`.

## 0.7.0 - 2023-09-18

This is the biggest update so far, introducing the long awaited undocking feature: tabs can now be dragged out into
new egui windows. Massive thanks to [Vickerinox](https://github.com/Vickerinox) for implementing it!

This update also includes an overhaul of the documentation, aiming to not only be more readable and correct, but also
provide a guide of how to use the library.

### Changed

- Adjusted the styling of tabs to closer follow the egui default styling. ([#139](https://github.com/Adanos020/egui_dock/pull/139))
- Double-clicking on a separator resets the size of both adjacent nodes. ([#146](https://github.com/Adanos020/egui_dock/pull/146))
- Tabs can now only be dragged with the primary pointer button (e.g. left mouse button). ([#177](https://github.com/Adanos020/egui_dock/pull/177))

### Fixed

- Correctly draw a border around a dock area using the `Style::border` property. ([#139](https://github.com/Adanos020/egui_dock/pull/139))
- Non-closable tabs now cannot be closed by clicking with the middle mouse button. ([9cdef8c](https://github.com/Adanos020/egui_dock/pull/149/commits/9cdef8cb77e73ef7a065d1313f7fb8feae0253b4))
- Dragging tabs around now works on touchscreens. ([#180](https://github.com/Adanos020/egui_dock/pull/180))

### Added

- From [#139](https://github.com/Adanos020/egui_dock/pull/139):
  - `Style::main_surface_border_rounding` for the rounding of the dock area border.
  - `TabStyle::active` for the active style of a tab.
  - `TabStyle::inactive` for the inactive style of a tab.
  - `TabStyle::focused` for the focused style of a tab.
  - `TabStyle::hovered` for the hovered style of a tab.
  - `TabStyle::tab_body` for styling the body of the tab including background color, stroke color, rounding and inner margin.
  - `TabStyle::minimum_width` to set the minimum width of the tab.
  - `TabInteractionStyle` to style the active/inactive/focused/hovered states of a tab.
- `AllowedSplits` enum which lets you choose in which directions a `DockArea` can be split. ([#145](https://github.com/Adanos020/egui_dock/pull/145))
- From [#149](https://github.com/Adanos020/egui_dock/pull/149):
  - `DockState<Tab>` containing the entire state of the tab hierarchies stored in a collection of `Surfaces`.
  - `Surface<Tab>` enum which represents an area (e.g. a window) with its own `Tree<Tab>`.
  - `SurfaceIndex` to identify a `Surface` stored in the `DockState`.
  - `Split::is_tob_bottom` and `Split::is_left_right`.
  - `TabInsert` which replaces current `TabDestination` (see breaking changes).
  - `impl From<(SurfaceIndex, NodeIndex, TabInsert)> for TabDestination`.
  - `impl From<SurfaceIndex> for TabDestination`.
  - `TabDestination::is_window` (see breaking changes).
  - `Tree::root_node` and `Tree::root_node_mut`.
  - `Node::rect` returning the `Rect` occupied by the node.
  - `Node::tabs` and `Node::tabs_mut` returning an optional slice of tabs if the node is a leaf.
  - `WindowState` representing the current state of a `Surface::Window` and allowing you to manipulate the window.
  - `OverlayStyle` (stored as `Style::overlay`) and `OverlayFeel`: they specify the look and feel of the drag-and-drop overlay.
  - `OverlayType` letting you choose if the overlay should be the new icon buttons or the old highlighted rectangles.
  - `LeafHighlighting` specifying how a currently hovered leaf should be highlighted.
  - `DockArea::window_bounds` setting the area which windows are constrained by.
  - `DockArea::show_window_close_buttons` setting determining if windows should have a close button or not.
  - `DockArea::show_window_collapse_buttons` setting determining if windows should have a collapse button or not.
  - `TabViewer::allowed_in_windows` specifying if a given tab can be shown in a window.
- `TabViewer::closable` lets individual tabs be closable or not. ([#150](https://github.com/Adanos020/egui_dock/pull/150))
- `TabViewer::scroll_bars` specifying if horizontal and vertical scrolling is enabled for given tab – replaces `DockArea::scroll_area_in_tabs` (see breaking changes). ([#160](https://github.com/Adanos020/egui_dock/pull/160))
- `Translations` specifying what text will be displayed in some parts of the `DockingArea`, e.g. the tab context menus (defined in `TabContextMenuTranslations`). ([#178](https://github.com/Adanos020/egui_dock/pull/178)) 

### Breaking changes

- From [#139](https://github.com/Adanos020/egui_dock/pull/139):
  - Moved `TabStyle::inner_margin` to `TabBodyStyle::inner_margin`.
  - Moved `TabStyle::fill_tab_bar` to `TabBarStyle::fill_tab_bar`.
  - Moved `TabStyle::outline_color` to `TabInteractionStyle::outline_color`.
  - Moved `TabStyle::rounding` to `TabInteractionStyle::rounding`.
  - Moved `TabStyle::bg_fill` to `TabInteractionStyle::bg_fill`.
  - Moved `TabStyle::text_color_unfocused` to `TabStyle::inactive.text_color`.
  - Moved `TabStyle::text_color_active_focused` to `TabStyle::focused.text_color`.
  - Moved `TabStyle::text_color_active_unfocused` to `TabStyle::active.text_color`.
  - Renamed `Style::tabs` to `Style::tab`.
  - Removed `TabStyle::text_color_focused`. This style was practically never reachable.
- From [#149](https://github.com/Adanos020/egui_dock/pull/149):
  - `TabDestination` now specifies if a tab will be moved to a `Window`, a `Node`, or an `EmptySurface`. Its original purpose is now served by `TabInsert`. 
  - `Tree::split` now panics if supplied `fraction` is not in range 0..=1.
  - Moved `Tree::move_tab` to `DockState::move_tab`.
  - Renamed `Style::border` to `Style::main_surface_border_stroke`.
  - Moved `Style::selection_color` to `OverlayStyle::selection_color`.
  - `DockArea::new` now takes in a `DockState` instead of a `Tree`.
- Removed `DockArea::scroll_area_in_tabs` – override `TabViewer::scroll_bars` instead. ([#160](https://github.com/Adanos020/egui_dock/pull/160))
- Methods `TabViewer::{context_menu,on_add,add_popup}` now take in an additional `SurfaceIndex` parameter. ([#167](https://github.com/Adanos020/egui_dock/pull/167))

## 0.6.3 - 2023-06-16

### Fixed
- Made the `DockArea` always allocate an area ([#143](https://github.com/Adanos020/egui_dock/pull/143))

## 0.6.2 - 2023-06-09

### Fixed
- Make the `max_size` of `tabbar_inner_rect` finite ([#141](https://github.com/Adanos020/egui_dock/pull/141))

## 0.6.1 - 2023-05-29

### Fixed
- Ensure rect size are calculated before drawing node bodies ([#134](https://github.com/Adanos020/egui_dock/pull/134))

## 0.6.0 - 2023-05-24

### Added

- `TabViewer::tab_style_override` that lets you define a custom `TabsStyle` for an individual tab ([99333b0](https://github.com/Adanos020/egui_dock/commit/99333b093d307181c288b3e134379cfe47647a7c))
- `ButtonsStyle::add_tab_border_color` for the `+` button's left border ([99333b0](https://github.com/Adanos020/egui_dock/commit/99333b093d307181c288b3e134379cfe47647a7c))
- `TabBarStyle::rounding` for rounding of the tab bar, independent from tab rounding ([99333b0](https://github.com/Adanos020/egui_dock/commit/99333b093d307181c288b3e134379cfe47647a7c))
- Separate `from_egui` methods for `ButtonsStyle`, `SeparatorStyle`, `TabBarStyle`, and `TabStyle` ([a660497](https://github.com/Adanos020/egui_dock/commit/a660497b21651dd9920665bf50d8fc9e75d0e1e0))

### Breaking changes

- Upgraded `egui` to version 0.22 ([c2e8fee](https://github.com/Adanos020/egui_dock/commit/c2e8feeb7713e2b2d2f0fa1b13a46732f9c6df62))
- Renamed `TabsStyle` to `TabStyle` ([89f3248](https://github.com/Adanos020/egui_dock/commit/89f32487a9e1fe8dee92f1fbdc296a2d460c0909))
- Removed `StyleBuilder` ([9a9b275](https://github.com/Adanos020/egui_dock/commit/9a9b2750cd290bebcc4088761249e02102cb0ce7))
- Removed `TabViewer::inner_margin_override` – no deprecation as it's in direct conflict with `TabViewer::tab_style_override` ([99333b0](https://github.com/Adanos020/egui_dock/commit/99333b093d307181c288b3e134379cfe47647a7c))
- Moved `Style::default_inner_margin` to `TabsStyle::inner_margin` ([78ecf3a](https://github.com/Adanos020/egui_dock/commit/78ecf3a175ffb960724f328274682dfded800e0f))
- Moved `TabStyle::hline_color` to `TabBarStyle::hline_color` ([99333b0](https://github.com/Adanos020/egui_dock/commit/99333b093d307181c288b3e134379cfe47647a7c))

## 0.5.2 - 2023-06-04

### Fixed
- Ensure rect size are calculated before drawing node bodies ([#134](https://github.com/Adanos020/egui_dock/pull/134))

## 0.5.1 - 2023-05-20

## Fixed
- Ensure close button can be scrolled to when tab bar is small ([#129](https://github.com/Adanos020/egui_dock/pull/129))

### Added
- `SeparatorStyle::extra_interact_width` option that adds "logical" width to separators so that they are easier to grab ([#128](https://github.com/Adanos020/egui_dock/pull/128))

## 0.5.0 - 2023-04-22

### Fixed
- Ensure `Tab` have a stable `egui::Id` when moved ([#121](https://github.com/Adanos020/egui_dock/pull/121))
- Don't display the "grab" cursor icon on tabs when hovered and the `draggable_tabs` flag is unset ([#123](https://github.com/Adanos020/egui_dock/pull/123))

### Added
- `Tree::move_tab` method that allows moving a tab from one node to the other ([#115](https://github.com/Adanos020/egui_dock/pull/107))
- `Tree::remove_leaf` method that deletes a selected leaf node ([#115](https://github.com/Adanos020/egui_dock/pull/107))
- New methods in `DockArea` ([#115](https://github.com/Adanos020/egui_dock/pull/115))
  - `show_add_popup`
  - `show_add_buttons`
  - `show_close_buttons`
  - `draggable_tabs`
  - `tab_context_menus`
  - `scroll_area_in_tabs`
  - `show_tab_name_on_hover`
- Make tabs scrollable when they overflow ([#116](https://github.com/Adanos020/egui_dock/pull/116))
- `TabViewer::id` method that allows specifying a custom id for each tab ([#121](https://github.com/Adanos020/egui_dock/pull/121))

### Breaking changes
- Removed `remove_empty_leaf` which was used for internal usage and should not be needed by users ([#115](https://github.com/Adanos020/egui_dock/pull/107))
- Removed `show_close_buttons` from `StyleBuilder` ([#115](https://github.com/Adanos020/egui_dock/pull/115))
- Moved the following fields from `Style` to `DockArea` ([#115](https://github.com/Adanos020/egui_dock/pull/115))
  - `show_add_popup`
  - `show_add_buttons`
  - `show_close_buttons`
  - `tabs_are_draggable` (renamed to `draggable_tabs`)
  - `show_context_menu` (renamed to `tab_context_menus`)
  - `tab_include_scrollarea` (renamed to `scroll_area_in_tabs`)
  - `tab_hover_name` (renamed to `show_tab_name_on_hover`)
- `Style` is now split up into smaller structs for maintainability and consistence with `egui::Style` ([#115](https://github.com/Adanos020/egui_dock/pull/115))

| Old names and locations                         | New names and locations                            |
|-------------------------------------------------|----------------------------------------------------|
| `Style::border_color` and `Style::border_width` | `Style::border` (which is now an `egui::Stroke`)   |
| `Style::separator_width`                        | `Separator::width`                                 |
| `Style::separator_extra`                        | `Separator::extra`                                 |
| `Style::separator_color_idle`                   | `Separator::color_idle`                            |
| `Style::separator_color_hovered`                | `Separator::color_hovered`                         |
| `Style::separator_color_dragged`                | `Separator::color_dragged`                         |
| `Style::tab_bar_background_color`               | `TabBar::bg_fill`                                  |
| `Style::tab_bar_height`                         | `TabBar::height`                                   |
| `Style::tab_outline_color`                      | `Tabs::outline_color`                              |
| `Style::hline_color`                            | `Tabs::hline_color`                                |
| `Style::hline_below_active_tab_name`            | `Tabs::hline_below_active_tab_name`                |
| `Style::tab_rounding`                           | `Tabs::rounding`                                   |
| `Style::tab_background_color`                   | `Tabs::bg_fill`                                    |
| `Style::tab_text_color_unfocused`               | `Tabs::text_color_unfocused`                       |
| `Style::tab_text_color_focused`                 | `Tabs::text_color_focused`                         |
| `Style::tab_text_color_active_unfocused`        | `Tabs::text_color_active_unfocused`                |
| `Style::tab_text_color_active_focused`          | `Tabs::text_color_active_focused`                  |
| `Style::expand_tabs`                            | `Tabs::fill_tab_bar`                               |
| `Style::close_tab_color`                        | `Buttons::close_tab_color`                         |
| `Style::close_tab_active_color`                 | `Buttons::close_tab_active_color`                  |
| `Style::close_tab_background_color`             | `Buttons::close_tab_bg_fill`                       |
| `Style::add_tab_align`                          | `Buttons::add_tab_align`                           |
| `Style::add_tab_color`                          | `Buttons::add_tab_color`                           |
| `Style::add_tab_active_color`                   | `Buttons::add_tab_active_color`                    |
| `Style::add_tab_background_color`               | `Buttons::add_tab_bg_fill`                         |

### Deprecated
- `StyleBuilder`

## 0.4.2 - 2023-03-17

### Fixed
- `TabViewer::clear_background` works again ([#110](https://github.com/Adanos020/egui_dock/pull/110))

## 0.4.1 - 2023-03-14

### Fixed
- Light mode now works in tabs ([528b892](https://github.com/Adanos020/egui_dock/commit/528b89245928d055dabb00cd9001c22d275f789b))
- `DockArea::show_inside` no longer obscures previously added elements ([#102](https://github.com/Adanos020/egui_dock/pull/102))
- Splitter drag now behaves like egui `DragValue` ([#103](https://github.com/Adanos020/egui_dock/pull/103))

## 0.4.0 - 2023-02-09

### Added
- Added `TabViewer::on_tab_button` ([#93](https://github.com/Adanos020/egui_dock/pull/93)).

### Breaking changes
- Updated to egui 0.21
- Deleted `dynamic_tab` which was deprecated in 0.3.0

### Fixed
- Make splitter drag behave like egui `DragValue` ([#103](https://github.com/Adanos020/egui_dock/pull/103))

## 0.3.1 - 2022-12-21

### Added
- `Style` now includes an option to change the tab's height - `tab_bar_height`. ([#62](https://github.com/Adanos020/egui_dock/pull/62))
- Implemented the `std::fmt::Debug` trait on all exported types. ([#84](https://github.com/Adanos020/egui_dock/pull/84))

### Fixed
- Errors in the README

## 0.3.0 - 2022-12-10

### Added
- `TabViewer::clear_background` method that returns if current tab's background should be cleared. ([#35](https://github.com/Adanos020/egui_dock/pull/35))
- You can now close tabs with middle mouse button if `Style::show_close_buttons` is true. ([#34](https://github.com/Adanos020/egui_dock/pull/34))
- Option to disable dragging tabs.
- New option `expand_tabs` in `Style` and `StyleBuiler` causes tab titles to expand to match the width of their tab bars.
- `StyleBuilder::from_egui`. ([#40](https://github.com/Adanos020/egui_dock/pull/40))
- `Tree::find_active_focused`. ([#40](https://github.com/Adanos020/egui_dock/pull/40))
- Added `context_menu` into `TabViewer`. ([#46](https://github.com/Adanos020/egui_dock/pull/46))
- The `ScrollArea` inside a tab is now optional via `Style`. ([#49](https://github.com/Adanos020/egui_dock/pull/49))
- `Tree::tabs`: an iterator over the tabs in a tree. ([#53](https://github.com/Adanos020/egui_dock/pull/53))
- `Style` now includes an option to show the hovered tab's name. ([#56](https://github.com/Adanos020/egui_dock/pull/56))
- `Style` now includes an option to change default inner_margin. ([#67](https://github.com/Adanos020/egui_dock/pull/67))
- The split separator now highlights on hover ([#68](https://github.com/Adanos020/egui_dock/pull/68))
- Tabs can now be removed with `Tree::remove_tab` ([#70](https://github.com/Adanos020/egui_dock/pull/70))

### Breaking changes
- Renamed `TabViewer::inner_margin` to `TabViewer::inner_margin_override`. ([#67](https://github.com/Adanos020/egui_dock/pull/67))
- `Style::with_separator_color` has been split into `separator_color_idle`, `separator_color_hovered`, `separator_color_dragged` ([#68](https://github.com/Adanos020/egui_dock/pull/68))
- Updated `egui` to 0.20.0 [#77](https://github.com/Adanos020/egui_dock/pull/77)

### Deprecated (will be deleted in the next release)
- `dynamic_tab::TabContent`
- `dynamic_tab::OnClose`
- `dynamic_tab::ForceClose`
- `dynamic_tab::TabBuilder`
- `dynamic_tab::Tab`
- `dynamic_tab::BuiltTab`
- `dynamic_tab::DynamicTree`
- `dynamic_tab::DynamicTabViewer`

## 0.2.1 - 2022-09-09

### Added
- Added opt-in `serde` feature to enable serialization of `Tree`.
- You can now change the tab text color with `Style::tab_text_color_unfocused` and `Style::tab_text_color_focused`.

### Fixed
- `Tree::push_to_first_leaf` no longer panics when used on an empty `Tree`.
- The tab text color will now follow the egui text color.


## 0.2.0 - 2022-09-04

### Added

- It is now possible to close tabs with a close button that can be shown/hidden through `Style`.
- When dragging tabs onto the tab bar if the tab will be inserted a highlighted region will show where the tab will end up if dropped.
- The dock will keep track of the currently focused leaf.
- Using `Tree::push_to_focused_leaf` will push the given tab to the currently active leaf.
- `StyleBuilder` for the `Style`.
- New fields in `Style:` `separator_color`, `border_color`, and `border_width` (last two for the cases when used `Margin`).
- `TabBuilder` for the `BuiltTab`.
- Support for all implementations of `Into<WidgetText>` in tab titles.
- Style editor in the `hello` example.
- Added `Tree::find_tab`, `TabViewer`, `DynamicTabViewer`, `DynamicTree`.
- Added a `text_editor` example.

### Changed

- If a tab is dropped onto the tab bar it will be inserted into the index that it is dropped onto.
- Now when you drag a tab it has an outline along the entire length of the edges of it.
- Bumped MSRV to `1.62`.
- `Tree` is now generic over how you want to represent a tab.

### Breaking changes

- Ui code of the dock has been moved into `DockArea` and is displayed with `DockArea::show` or `DockArea::show_inside`.
- Renamed `Style::border_size` to `Style::border_width`.
- Renamed `Style::separator_size` to `Style::separator_width`.
- Removed `Style::tab_text_color` as you can now set the tab text color of a tab by passing `RichText` for its title.
- Removed the requirement of creating your own Context type.
- Renamed `Tree::set_focused` to `Tree::set_focused_node`.
- Renamed `Node::None` to `Node::Empty`.

### Fixed
- Now selection color of the placing area for the tab isn't showing if the tab is targeted on its own node when the tab is the only member of  this node.
- Dock vertical and horizontal separators are now displayed properly.
- Prevent Id clashes from multiple tabs being displayed at once.
- Tab content is now displayed inside a `egui::ScrollArea`, so it's now accessible in its entirety even if the tab is too small to fit all of it.
- Fixed an issue where some tabs couldn't be resized.
