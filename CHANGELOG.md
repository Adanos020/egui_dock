# Unreleased

## Added

- `StyleBuilder` for the `Style`
- New fields in `Style:` `separator_color`, `border_color`, and `border_size` (last two for the cases when used `Margin`)
- `TabBuilder` for the `Tab`
- Support for all implementations of `Into<WidgetText>` in tab titles
- Style editor in the `hello` example

## Changed

- Now when you drag a tab it has an outline along the entire length of the edges of it
- Bumped MSRV to `1.62`

## Breaking changes

- `Tab` is no longer a trait for the user to implement, instead it is now a customizable widget constructed with `TabBuilder`
- Renamed `Style::border_size` to `Style::border_width`
- Renamed `Style::separator_size` to `Style::separator_width`
- Removed `Style::tab_text_color` as you can now set the tab text color of a tab by passing `RichText` for its title
- Removed the requirement of creating your own Context type

## Fixed

- Now selection color of the placing area for the tab isn't showing if the tab is targeted on its own node when the tab is the only member of  this node
