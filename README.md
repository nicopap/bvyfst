# Fast Bevy Scene

A bevy fast scene format.

This crates uses

- [`tmf`] for mesh storage
- [`basis-universal`] for image storage
- [`rkyv`] for the scene representation
- [`ar`] to put all of those into a single file.

[`tmf`]: https://github.com/fractalfir/tmf
[`basis-universal`]: https://lib.rs/crates/basis-universal
[`rkyv`]: https://lib.rs/crates/rkyv
[`ar`]: https://lib.rs/crates/ar

### File format

A bevy "Fast scene" has the `.bvyfst` extension (we leave out the vowels, 40
decades of C programming have taught us that no vowels = extremely fast)

It is designed to load ~~bevy~~ very fast.

It is nothing more than an old schoold unix `.a` archive file. It contains the
following files:

1. `scene_vXXXXX`: the bevy scene description, which contains:
   - **FUTURE**: may be delta-encoded with low lz4 compression
   - `vXXXXX` is a digit representing the scene format version. (currently `v00001`, I know,
     accounting for a hundred thousand versions is mad, but it costs nothing)
   - Description of entities: Parent/Child Hierarchy, their StandardMaterial, Mesh
     and Transform, 448 bits per entity
   - Image metadata, 16 bits per image
   - All Material descriptions. 512 bits per material
2. `meshes`: a `tmf`-formatted file containing the collection of meshes used in
   `scene_vx`. They are ordered by index as well.
3. `imgages`: a `basis universal` file containing the collection of images used in
   `scene_vx` materials. They are ordered by index as well.

Note that the list order is important, otherwise the reader will treat the archive
as invalid.

#### Limitations

Individual files in the archive cannot be larger than 8GB, meaning that none
of the following can be true:

- `scene_vx` is larger than 8GB (154M entities **OR** 125M materials **OR** 4G images)
- `images`, all images used in the scene (compressed), is larger than 8GB
- `meshes`, all meshes used in the scene (compressed), is larger than 8GB.
- If somehow several entities share the same children (this should be impossible),
  they will be duplicated on load 
