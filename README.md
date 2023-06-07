# Bevy Fast Assets

A collection of efficient asset loader/savers for bevy Asset v2 🧪.

## Crates

This repository contains several crates:

- [`bvyfst_mesh`]: Mesh loading based on [`tmf`]
- [`bvyfst_hollow_scene`]: An **asset-less** scene format based on [`rkyv`].
  This stores a scene hierarchy and a subset of components relevant to a 3d scene.
  The `Handle<A>` components are stored as file paths.
- [`bvyfst_scene`]: A scene format with embedded assets, storing scene assets
  in the same file.
  - Based on [`proto_async_ar`], a fork of the excellent [`ar`] crate.
  - [`rkyv`] similarly to `bvyfst_hollow_scene`
  - [`basis-universal`] for embedding texture files
  - [`tmf`] for meshes

[`bvyfst_mesh`]: ./mesh
[`bvyfst_scene`]: ./scene
[`bvyfst_hollow_scene`]: ./hollow_scene
[`tmf`]: https://github.com/fractalfir/tmf
[`basis-universal`]: https://lib.rs/crates/basis-universal
[`rkyv`]: https://lib.rs/crates/rkyv
[`ar`]: https://lib.rs/crates/ar
[`proto_async_ar`]: https://lib.rs/crates/proto_async_ar


