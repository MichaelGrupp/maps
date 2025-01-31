# maps

**Inspect, compare and align multiple grid maps in an intuitive & fast GUI**

## Audience

`maps` can be useful for you if ...

* ...you work on mobile robot SLAM or navigation.
* ...your SLAM or navigation system supports exporting maps as grid map files.
* ...you want to quickly view the maps, but other viewers are either...
  * ...tedious to spin up (e.g. ROS nodes/topics + RViz).
  * ...not supporting grid coordinates (most image viewers).
* ...you want to display in a shared coordinate system, take measurements etc.
* ...you want to align multiple, potentially very large maps.

The features are best summarized in a video:

https://github.com/user-attachments/assets/391e8e02-77da-42ac-991d-02578a3cdaa0

## Goals

At its core, `maps` is an image viewer that is aware of the metric properties of the grid maps.

### Intuitive
  * Maps of different resolutions can be displayed in a shared coordinate system with correct scale and position.
  * Details of large maps can be quickly inspected using a lens tool without zooming & dragging.
  * Files can be loaded both via GUI and CLI.
### Fast
  * Interaction should be fast and responsive, also with very large high resolution maps.
  * Built with [Rust](https://www.rust-lang.org/) using [egui](https://github.com/emilk/egui) + [wgpu](https://github.com/gfx-rs/wgpu).

> ⚠️ `maps` is ready to use, but also under active development. Some features may be added or changed in upcoming versions.

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

### Lens

The lens tool magnifies a region below the mouse cursor to the original image size. This makes it fast to inspect details of large maps in selected regions without tedious zooming and/or dragging.

* Right-click the mouse on a map to enable/disable the lens (or press L).
* Scroll to adjust the size of the lens.

### Menu & Settings

* Click `☰` to open the sidebar to manage maps and their visibility.
* Click `⚙` to open the sidebar for settings.

## Install

First, you need to [install the Rust toolchain](https://www.rust-lang.org/tools/install).

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

## License

Apache 2.0
