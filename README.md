# maps

**Inspect, compare and align multiple grid maps in an intuitive & fast GUI**

## Audience

`maps` can be useful for you if ...

* ...you work on mobile robot SLAM or navigation.
* ...your SLAM or navigation system supports exporting maps as grid map images.
* ...you want to quickly work with the map files, but other tools are either...
  * ...better suited for live data streams (e.g. RViz, Rerun, Foxglove etc)
  * ...not supporting grid coordinates (most image viewers)
* ...you want to display in a shared coordinate system, take measurements etc.
* ...you want to align multiple, potentially very large maps.

The features are best summarized in a video:

https://github.com/user-attachments/assets/391e8e02-77da-42ac-991d-02578a3cdaa0

## Goals

At its core, `maps` is an image viewer that is aware of the metric properties of the grid maps.

### Intuitive
  * Maps of different resolutions can be displayed in a shared coordinate system with correct scale and position.
  * Details of large maps can be quickly inspected using a lens tool without zooming & dragging.
  * Several keybindings make it fast to use, e.g. `W A S D` for moving and `Q E` for rotating.
  * Files can be loaded both via GUI and CLI.
  * Sessions can be saved and loaded at a later point to continue working.
  * Settings are autosaved by default.

### Fast
  * Interaction should be fast and responsive, also with very large high resolution maps.
  * maps is optimized to allow dragging / rotating images also at high zoom levels in real-time, with efficient resource usage.
  * Built with [Rust](https://www.rust-lang.org/) using [egui](https://github.com/emilk/egui) + [wgpu](https://github.com/gfx-rs/wgpu).

## Input

The supported grid map file format is ROS map_server files, i.e. a pair of image and metadata per map:

* YAML metadata file containing information about the origin, resolution and other properties.
* Image file containing the grid cells.

See the [ROS documentation](http://wiki.ros.org/map_server#Map_format) for all details.

> 💡 `maps` does not require a ROS installation, it just uses this data format as convention.

## User Interface

### Views

`maps` provides three different main view modes.

* `Aligned`:
  * Maps are shown in a metric grid, with their origin at zero.
  * The grid can be dragged and zoomed, grid lines can be shown etc.
* `Tiles`:
  * Map images are shown in separate tab tiles.
  * The tab tiles can be freely rearranged, for example to view images side by side.
* `Stacked`: Map images are shown stacked in a scrollable view.

### Measurements

Does exactly what you think: activate the tool and click two points in the aligned grid view to measure their distance.

### Lens

The lens tool magnifies a region below the mouse cursor with a magnification factor (in Aligned view) or to the original image size (in Tiles / Stacked view). This makes it fast to inspect details of large maps in selected regions without tedious zooming and/or dragging.

* Right-click the mouse on a map to enable/disable the lens (or press L).
* Adjust it...
 * in Aligned view: use the options side bar to set the magnification factor.
 * in Tiles/Stacked view: scroll to adjust the size of the lens.

### Fixed Lens

In the Aligned view, you can add multiple lenses that are looking at a fixed coordinate each. They stay centered at the coordinate that was clicked, even if the main grid is moved.

This can be useful when aligning large maps, where you need to watch different areas in detail to check how well they fit while moving the map.

### Pose Alignment

You can change the pose of a map relative to the global origin in the aligned grid view.

* Select the map that you want to move in the menu sidebar.
* Enter values or move the map with the keyboard.
* Poses can be exported to YAML files.

> 👉 maps doesn't touch the `origin` of your `map.yaml` file, but writes a separate file.
> Many ROS tools don't support rotations in the map yaml file, and it's anyway cleaner to separate the alignment pose from the map origin.

### Menu & Settings

* Click `☰` to open the sidebar to manage maps and their visibility.
* Click `⚙` to open the sidebar for settings.
* Click the ℹ️ button in the lower right corner to display version & keybindings.

### Session files

You can save your session and reload it later using the menu. maps also asks you if you want to save before quitting or when there are unsaved changes.

> ⚠️ The files that are written are not self-contained. They just contain the relevant config and point to the map file paths.

## Install

First, you need to [install the Rust toolchain](https://www.rust-lang.org/tools/install) if you don't have it already.

Then you can install the latest release from [crates.io](https://crates.io/):
> ⚠️ TODO

---

You can also build completely from source if you want.
<details>
<summary>Details</summary>

Clone this repository, then:
```bash
cargo install --path maps/
```

(or `cargo build --release` if you just want to build)

</details>

## Run

Start the app with:

```bash
maps
```
Use the `Load Maps` button to add your map files.


You can also already pass map file paths from the command line:

```bash
maps some/map.yaml some/other/map.yaml
```

See `maps --help` for all command line options.

## License

Apache 2.0
