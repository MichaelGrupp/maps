# maps_io_ros

`maps_io_ros` provides fundamental I/O for 2D ROS grid maps including: metadata parsing, value interpretation, colormaps and map poses.

See the [maps](https://crates.io/crates/maps) app crate for a full GUI application that builds on top of this I/O library.

This crate was split out of `maps` with minimal dependencies and can be used also in other robotics applications that work with ROS map files and/or display 2D occupancy grids. It does __not__ require ROS installation.

> ⚠️ While the `maps` app is stable, `maps_io_ros` doesn't have a stable API yet.
> Parts of the library might change in upcoming minor versions.

## Demo

See `examples/demo.rs` for a minimal example how this library can be used. The demo loads a ROS map file and saves a processed image that looks like what you would see when publishing that map with a `map_server` and displaying it in `RViz`.

You can run it from within the maps repository with:

```
cd maps_io_ros/
cargo run -r --example demo your_map.yaml
```
<img width=200 src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/maps_io_ros/doc/orig.png" />
<img width=200 src="https://raw.githubusercontent.com/MichaelGrupp/maps/refs/heads/master/maps_io_ros/doc/proc.png" /> |
