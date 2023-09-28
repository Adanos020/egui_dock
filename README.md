# `egui_dock`: docking support for [egui](https://github.com/emilk/egui)

[![egui_ver](https://img.shields.io/badge/egui-0.22-blue)](https://github.com/emilk/egui)
[![Crates.io](https://img.shields.io/crates/v/egui_dock)](https://crates.io/crates/egui_dock)
[![docs.rs](https://img.shields.io/docsrs/egui_dock)](https://docs.rs/egui_dock/)

Originally created by [@lain-dono](https://github.com/lain-dono), this library provides docking support for `egui`.

## Contributing

Before contributing, please read [the contribution guide](CONTRIBUTING.md).

This library is a collaborative project developed with direct involvement of its users.

Please feel free to open new issues and pull requests, and participate in discussions!
A lot of our discussions take place on [`egui`'s official Discord server](https://discord.gg/JFcEma9bJq),
in the `#egui_dock` channel.

## Features

- Opening and closing tabs.
- Moving tabs between nodes and resizing.
- Dragging tabs out into new `egui` windows.
- Highly customizable look and feel.
- High degree of control over behaviour of the whole dock area and of individual tabs.
- Manipulating tabs and dock layout from code.

## Quick start

Add `egui` and `egui_dock` to your project's dependencies.

```toml
[dependencies]
egui = "0.23"
egui_dock = "0.8"
```

Then proceed by setting up `egui`, following its [quick start guide](https://github.com/emilk/egui#quick-start).
Once that's done, you can start using `egui_dock` – more details on that can be found in the
[documentation](https://docs.rs/egui_dock/latest/egui_dock/).

## Demo

![demo](images/demo.gif "Demo")

## Alternatives

### [egui_tiles](https://github.com/rerun-io/egui_tiles) 

It's a library aiming to achieve similar goals in addition to being more flexible and customizable.

One feature it supports that `egui_dock` does not at the moment is the ability to divide nodes into more than two children,
enabling horizontal, vertical, and grid layouts.

> [!NOTE]
> `egui_tiles` is much earlier in development than `egui_dock` and doesn't yet support a lot of features.
