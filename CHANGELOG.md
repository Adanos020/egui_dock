# egui_dock changelog

## Unreleased

### Added
- `TabViewer::clear_background` method that returns if current tab's background should be cleared. ([#35](https://github.com/Adanos020/egui_dock/pull/35))
- You can now close tabs with middle mouse button if `Style::show_close_buttons` is true. ([#34](https://github.com/Adanos020/egui_dock/pull/34))
- Option to disable dragging tabs.
- New option `expand_tabs` in `Style` and `StyleBuiler` causes tab titles to expand to fill the width of their tab bars.
- `StyleBuilder::from_egui`. ([#40](https://github.com/Adanos020/egui_dock/pull/40))
- `Tree::find_active_focused`. ([#40](https://github.com/Adanos020/egui_dock/pull/40))
- Added `context_menu` into `TabViewer`. ([#46](https://github.com/Adanos020/egui_dock/pull/46))
- The `ScrollArea` inside a tab is now optional via `Style`. ([#49](https://github.com/Adanos020/egui_dock/pull/49))
- `Tree::tabs`: an iterator over the tabs in a tree. ([#53](https://github.com/Adanos020/egui_dock/pull/53))
- `Style` now includes an option to show the hoverd tab's name. ([#56](https://github.com/Adanos020/egui_dock/pull/56))
- `Style` now includes an option to change default inner_margin. ([#67](https://github.com/Adanos020/egui_dock/pull/67))
- The split separator now highlights on hover ([#68](https://github.com/Adanos020/egui_dock/pull/68))
- Tabs can now be removed with `Tree::remove_tab` ([#70](https://github.com/Adanos020/egui_dock/pull/70))

### Breaking changes
- Renamed `TabViewer::inner_margin` to `TabViewer::inner_margin_override`. ([#67](https://github.com/Adanos020/egui_dock/pull/67))
- `Style::with_separator_color` has been split into `separator_color_idle`, `separator_color_hovered`, `separator_color_dragged` ([#68](https://github.com/Adanos020/egui_dock/pull/68))

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
