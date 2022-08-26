# Unreleased

## Added

- It is now possible to close tabs with a close button that can be shown/hidden through `Style`
- When dragging tabs onto the tab bar if the tab will be inserted a highlighted region will show where the tab will end up if dropped.
- The dock will keep track of the currently focused leaf.
- Using `push_to_active_leaf` will push the given tab to the currently active leaf.
- Fix to prevent Id clashes from multiple tabs being displayed at once.
- `StyleBuilder` for the `Style`
- New fields in `Style:` `separator_color`, `border_color`, and `border_size` (last two for the cases when used `Margin`)
- `TabBuilder` for the `Tab`
- Support for all implementations of `Into<WidgetText>` in tab titles
- Style editor in the `hello` example

## Changed

- If a tab is dropped onto the tab bar it will be inserted into the index that it is dropped onto.
- Now when you drag a tab it has an outline along the entire length of the edges of it
- Bumped MSRV to `1.62`

## Breaking changes

- Ui code of the dock has been moved into `DockArea` and is displayed with `DockArea::show`
- `Tree` cannot be directly accessed through `DockArea`
- Renamed `Style::border_size` to `Style::border_width`
- Renamed `Style::separator_size` to `Style::separator_width`
- Removed `Style::tab_text_color` as you can now set the tab text color of a tab by passing `RichText` for its title
- Removed the requirement of creating your own Context type

## Fixed

- Now selection color of the placing area for the tab isn't showing if the tab is targeted on its own node when the tab is the only member of  this node
- Dock vertical and horizontal separators are now displayed properly