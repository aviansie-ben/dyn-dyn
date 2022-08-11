# `dyn-dyn`

`dyn-dyn` allows for flexible downcasting of dynamic trait objects into other dynamic trait objects using the unstable `ptr_metadata` feature. Unlike many other crates providing similar functionality, `dyn-dyn` does not rely on any linker tricks or global registries to do its magic, making it safe to use in `#![no_std]` crates and even without `alloc`. `dyn-dyn` also does not require the base trait to in any way list what traits it may be downcast to: the implementing type has full control to select any set of valid traits to expose.

## Usage

`dyn-dyn` is used by declaring a "base trait" annotated with the `#[dyn_dyn_base]` attribute macro and annotating any `impl` blocks for that trait using the `#[dyn_dyn_derived(...)]` attribute macro, like so:

```rust
#[dyn_dyn_base]
trait BaseTrait {}
trait ExposedTrait {}

struct Struct;

impl ExposedTrait for Struct {}

#[dyn_dyn_derived(ExposedTrait)]
impl BaseTrait for Struct {}
```

A reference to the trait object `&dyn BaseTrait` can then be downcast to any other trait object listed in the arguments of the `#[dyn_dyn_derived(...)]` attribute by using the `dyn_dyn_cast!` macro:

```rust
let _: Option<&dyn ExposedTrait> = dyn_dyn_cast!(BaseTrait => ExposedTrait, &TestStruct as &dyn BaseTrait);
let _: Option<&mut dyn ExposedTrait> = dyn_dyn_cast!(mut BaseTrait => ExposedTrait, &mut TestStruct as &mut dyn BaseTrait);
```

## Using `dyn-dyn` with auto traits

Unlike many other downcasting libraries, `dyn-dyn` allows downcasting operations to preserve extra marker traits on the trait object. For instance, it is possible to perform a downcast using `dyn_dyn_cast!(BaseTrait + Send => ExposedTrait + Send, ...)` to preserve an extra `Send` bound. Additionally, these marker traits can also be derived from supertraits of the base trait: so if `BaseTrait: Send`, then it is also possible to do `dyn_dyn_cast!(BaseTrait => ExposedTrait + Send, ...)`.

This works for any auto traits, including those declared by other crates using the unstable `auto_traits` feature.

## Limitations

Currently, `dyn-dyn` only works in nightly versions of Rust due to its use of the unstable `ptr_metadata` and `unsize` features, as well as due to its use of several standard library features in `const` contexts.

Due to limitations of `TypeId`, `dyn-dyn` can only currently work with types and traits that are `'static`.

In order to be able to construct a way of downcasting into every possible derived trait that a concrete type wishes to expose, the set of traits exposed using the `#[dyn_dyn_derived(...)]` attribute must be finite. That is, it is not possible to expose some generic trait `Trait<T>` for an arbitrary value of `T` (although it is possible to do so if `T` is constrained by a generic argument to the concrete type itself).

## How it works

`dyn-dyn` works by creating a table that maps various `TypeId`s corresponding to traits into the vtable pointer they use for a particular concrete type. This table is then exposed via a hidden supertrait of the base trait, allowing the `dyn_dyn_cast!` macro to dynamically look up the metadata corresponding to a particular trait object. This metadata is then reattached to the pointer using the unstable `ptr_metadata` feature to create a reference to the derived trait object type.

Extra bounds are handled by using the unstable `unsize` feature to cast from the type without this extra bound to the type with it, along with the macro emitting some extra code that causes compile failures if the provided base type does not imply one of the marker traits in question.
