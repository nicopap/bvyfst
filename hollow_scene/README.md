# Bevy fast Hollow Scene

(not to be confused with [Holocene])

An **asset-less** scene format based on [`rkyv`].
This stores a scene hierarchy and a subset of components relevant to a 3d scene.
The `Handle<A>` components are stored as file paths.

bvyfst_hollow_scene is a bevy scene representation especially designed load fast.

## Difference with `bvyfst_scene`

Unlike `bvyfst_hollow_scene`, `bvyfst_scene` stores assets within the scene file.

`bvyfst_hollow_scene` only stores file name of assets.

[Holocene]: https://en.wikipedia.org/wiki/Holocene
[`rkyv`]: https://lib.rs/crates/rkyv
