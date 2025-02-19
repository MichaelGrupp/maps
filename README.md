# maps

**Inspect, compare and align multiple grid maps in an intuitive & fast GUI**

| <img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/screenshot_0.png" width="250" />  | <img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/screenshot_1.png" width="250" />  | <img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/screenshot_2.png" width="250" /> |
|---|---|---|

```
cargo install maps
```

## Audience

`maps` can be useful for you if ...

* ...you work on mobile robot SLAM or navigation.
* ...your SLAM or navigation system supports exporting maps as 2D grid map images.
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
  * Sessions can be saved and loaded at a later point to continue working, settings are autosaved by default.
  * No context menus or other hidden UI.

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

### Menu & Settings

* Click `☰` to open the sidebar to manage maps and their visibility.
* Click `⚙` to open the sidebar for settings.
* Click the ℹ️ button in the lower right corner to display version & keybindings.

### Views

`maps` provides three different main view modes.

<img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/view_selection.png" width="150" />

* `Aligned`:
  * Maps are shown in a metric grid, with their origin at zero.
  * The grid can be dragged and zoomed, grid lines can be shown etc.
* `Tiles`:
  * Map images are shown in separate tab tiles.
  * The tab tiles can be freely rearranged, for example to view images side by side.
* `Stacked`: Map images are shown stacked in a scrollable view.

### Measurements

<img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/tool_measure.png" width="125" />

Does exactly what you think: activate the tool and click two points in the aligned grid view to measure their distance.

### Lens

<img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/tool_hover_lens.png" width="125" />

The lens tool magnifies a region below the mouse cursor with a magnification factor (in Aligned view) or to the original image size (in Tiles / Stacked view). This makes it fast to inspect details of large maps in selected regions without tedious zooming and/or dragging.

* Right-click the mouse on a map to enable/disable the lens (or press L).
* Adjust it...
  * in Aligned view: use the options side bar to set the magnification factor.
  * in Tiles/Stacked view: scroll to adjust the size of the lens.

### Fixed Lens

<img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/tool_fixed_lens.png" width="125" />

In the Aligned view, you can add multiple lenses that are looking at a fixed coordinate each. They stay centered at the coordinate that was clicked, even if the main grid is moved.

This can be useful when aligning large maps, where you need to watch different areas in detail to check how well they fit while moving the map.

### Pose Alignment

You can change the pose of a map relative to the global origin in the aligned grid view.

This can be used when you have multiple maps with different origins (e.g. different floors, different origin due to remapping),
or to align with a fixed layout, etc.

* Select the map that you want to move in the menu sidebar.
* Enter values or move the map with the keyboard (after enabling "Move Map")
* Poses can be exported to YAML files.
* Optionally, use the tools to make alignment easier, for example:
   * make the texture of the maps transparent/colored using the blend settings
   * add fixed lenses in different areas of a large map
   * adjust the movement step size for the WASD/QE keybindings to the sensitivity you need

<img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/alignment.gif" height="150" />

> 👉 maps doesn't touch the `origin` of your `map.yaml` file, but writes a separate file.
> Many ROS tools don't support rotations in the map yaml file, and it's anyway cleaner to separate the alignment pose from the map origin.

### Draw order

Also with transparency enabled, you might want to reorder the map layers.

Changing the draw is just a matter of using drag & drop in the map list of the menu panel:

https://github.com/user-attachments/assets/0d68e18b-2980-4ffa-bf93-470fe7a9612d

### Value interpretation & colormap

You can display maps the same way as you would see them in RViz when using an occupancy grid publisher like map_server together with RViz.

`maps` simulates this by first applying a [value interpretation](http://wiki.ros.org/map_server#Value_Interpretation) followed by a colormap.
Just enable value interpretation and choose the options you want.
This allows for example to tune the free/occupied thresholds for your application, since `maps` directly shows the effect when the corresponding slider is moved.

<img width="600" alt="value_interpretation_examples" src="https://github.com/user-attachments/assets/a5882a4d-0a06-4bdb-8bf8-ebc32041ea26" />
<img width="340" alt="value_interpretation_ui" src="https://github.com/user-attachments/assets/738843eb-9d15-4906-8a5b-eb273e5805e4" />


> 💡 If the map metadata YAML already contains the optional `mode` parameter, value interpretation is enabled automatically for that map.

> ⚠️ The implementation in map_server is not consistent with the documentation.
> This is a "standard" that most likely will stay, but worth to consider that there are slight differences in case you rely on the documentation.
> Hence `maps` defaults to a reimplementation of that map_server quirk, but an implementation following the Wiki docs can be chosen as alternative.

### Session files

You can save your session and reload it later using the menu. maps also asks you if you want to save before quitting or when there are unsaved changes.

Note that `maps` never overwrites your input map files.

> ⚠️ The files that are written are not self-contained. They just contain the relevant config and point to the map file paths.

### Configuration files

#### Autosave

The general options are autosaved by default.
So for example, if you change the grid color and spacing, you will automatically have the same setting the next time you open maps.

If you don't want this, you can disable this for the active session in the options side bar of the app.
All options can be reset with the respective reset buttons in the UI.

#### Custom

If you want to use a custom configuration file path instead of the autosaved default for a session:

```
maps --config my_custom_config.toml
```

If the file doesn't exist yet, it opens `maps` with default options, creates the file and saves your changes there on exit.
You can then reload these settings again anytime you want using the same command.

## Install

First, you need to [install the Rust toolchain](https://www.rust-lang.org/tools/install) if you don't have it already.

Then you can install the latest release from [crates.io](https://crates.io/):

```
cargo install maps
```

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

## Development / Testing

There are integration tests in the `tests/` directory that are run in pull requests.
These include kit tests that check if the UI stays consistent, using the snapshot diff feature of `egui_kittest`.

<details>
<summary>How to run locally</summary>

First, install [Git LFS](https://docs.github.com/en/repositories/working-with-files/managing-large-files/installing-git-large-file-storage) and `git lfs pull`.
The snapshot image blobs are versioned with it instead of normal Git.

Run with snapshot diff check enabled (default feature `kittest_snapshots` of this crate):
```
RUST_LOG=maps=info cargo test --profile kittest --verbose -- --show-output
```

To update the baseline snapshots (e.g. when changing the UI intentionally as part of the UI), set the `UPDATE_SNAPSHOTS=1` environment variable before the test command.

Run without snapshot checks:
```
RUST_LOG=maps=info cargo test --profile kittest --verbose --no-default-features -- --show-output
```

</details>

---

With `-log-level debug` or `trace`, a debug window can be opened through a ⚒️ button in the footer panel.

<img src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/data/doc/debug_button.png" width="125" />

## License

Apache 2.0

<details>
  <summary>
    Cite
  </summary>
  <p>
If you want to cite this repository, you can either use a footnote linking <code>https://github.com/MichaelGrupp/maps</code> or:
  </p>

<pre><code>@misc{grupp2025maps,
  title={maps: Inspect, compare and align multiple grid maps in an intuitive & fast GUI.},
  author={Grupp, Michael},
  howpublished={\url{https://github.com/MichaelGrupp/maps}},
  year={2025}
}</code></pre>

</details>
