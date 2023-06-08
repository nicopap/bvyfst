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

## Limitations

- This doesn't handle assets
- You have to define `ArchiveProxy`s yourself.
- The proxied components still need to implement `Reflect` and be registered.
  It would be otherwise impossible to save them.
- A `Scene` may contain at most 65535 instances of any given `Table`-storage component.

## Why do I need to provide scene loaders to the plugin?

From my understanding, the current bevy asset processing implementation requires
a unified loader (loads N formats)

```
  png
  bmp
  jpeg   --> ImageLoader --> Image --> ImageSaver ->-
->basis                                             |
| ktx2                                              |
|                                                   |
-----------------------------------------------------
```

This means that the `SceneLoader` exposed by this crate — in addition to our
`.hollow_bvyfst` format — needs to be able to read all scene format possible,
such as `.fbx`, `.glb`, `.gtlf` and bevy native's `.scn.ron` (though renamed
to `.myscn.ron`, otherwise the native loader would overrule ours).

However, the only format it needs to _write_ is `.hollow_bvyfst` scenes.

## What's up with the `Plugin!` macro

Hollow scene handles component registrations with two "hlists".
A hlist, or "heterogenous list" is a `struct` which fields can be specified
dynamically.

Let's take the info message printed at the plugin's startup:

```
Loader<
  ( Table<proxy::Id<basic_scene::ComponentB>>, ()),
  (
    Inline<proxy::Id<basic_scene::ComponentA>>,
    (Inline<basic_scene::MyTransform>, ())
  )
>
```

`Loader` has two type parameters:

- `Ts: Tables`: List of table-stored components
- `Is: Inlines`: List of inline-stored componets

The `Tables` and `Inlines` trait specify how components in the list are read
from and written to the ECS, they also allow the scene format to store the
components in the specified way.

The lists in question are defined as a series of nested tuples terminated by
an empty tuple:

- `Ts` = `(Table<Id<CompoentB>>, ())`
- `Is` = `(Inline<Id<ComponentA>>, (Inline<MyTransform>, ()))`

`Table` and `Inline` are newtypes that really don't need to exist, but makes
easier defining the traits implementations for `Inlines` and `Tables`.

The `Plugin!` macro does nothing else than converting two flat lists into the
nested tuple variants. In this case, I can guess it was called as follow:

```rust
Plugin!(
   Inline[Id<ComponentA>, MyTransform] // This is `Is`
   Table[Id<ComponentB>] // This is `Ts`
   Extras[]
);
```

An advantage of this over the classical `n_tuples!` macro used by bevy, is that
I'm not generating an insane amount of code through macros. The compilation cost
is strictly proportional to how large your type list will be. You are still
paying a fairly large monomorphization cost, but rustc doesn't seem like it
minds.
