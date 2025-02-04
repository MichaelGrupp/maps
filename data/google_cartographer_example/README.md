Example maps of Google's Cartographer mapping backpack [dataset](https://google-cartographer-ros.readthedocs.io/en/latest/data.html#d-cartographer-backpack-deutsches-museum), as shown in the README screenshots / video.

They are at 2.5cm/pixel resolution with relatively large extents, so rather big images.

> **Note 1**: these maps show different floor levels of the [Deutsches Museum](https://www.deutsches-museum.de/en/) and have some drift/distortions.
> The map pose files that align them are rough guesstimates. After all, they are just for demo purposes 🤷🏻‍♂️.

> **Note 2:** Images are stored using [Git LFS](https://docs.github.com/en/repositories/working-with-files/managing-large-files/installing-git-large-file-storage) instead of normal Git, so you might need to install that first & pull again if you cloned the repo.

To open it with `maps`, `cd` into this directory and either:

```sh
maps map_*.yaml

# ... and load the pose files for each map
```

or:
```
maps -s session.toml
```
