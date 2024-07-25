# `dyn-dyn`

[![Tests](https://img.shields.io/github/actions/workflow/status/aviansie-ben/dyn-dyn/check.yml?branch=master)](https://github.com/aviansie-ben/dyn-dyn/actions/workflows/check.yml)
[![Crates.io](https://img.shields.io/crates/v/dyn-dyn)](https://crates.io/crates/dyn-dyn)
[![docs.rs](https://img.shields.io/docsrs/dyn-dyn)](https://docs.rs/dyn-dyn/)

`dyn-dyn` allows for flexible downcasting of dynamic trait objects into other dynamic trait objects using the unstable `ptr_metadata` feature. Unlike many other crates providing similar functionality, `dyn-dyn` does not rely on any linker tricks or global registries to do its magic, making it safe to use in `#![no_std]` crates and even without `alloc`. `dyn-dyn` also does not require the base trait to in any way list what traits it may be downcast to: the implementing type has full control to select any set of valid traits to expose.

**DISCLAIMER**: This code has not been thoroughly audited or tested yet and relies on a lot of unstable features and hacks with unsafe code, so it's liable to break at any time. While tests are run under Miri to try to catch any UB, it's probably best not to rely on this crate in production code in its current state.

## Usage

`dyn-dyn` is used by declaring a "base trait" annotated with the `#[dyn_dyn_base]` attribute macro and annotating any `impl` blocks for that trait using the `#[dyn_dyn_impl(...)]` attribute macro. Any reference to the base trait can then be downcast to a reference to the derived trait by using the `dyn_dyn_cast!` macro, like so:

```rust
use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_impl};

#[dyn_dyn_base]
trait BaseTrait {}
trait ExposedTrait {}

struct Struct;

impl ExposedTrait for Struct {}

#[dyn_dyn_impl(ExposedTrait)]
impl BaseTrait for Struct {}

let mut s = Struct;

assert!(dyn_dyn_cast!(BaseTrait => ExposedTrait, &s).is_ok());
assert!(dyn_dyn_cast!(mut BaseTrait => ExposedTrait, &mut s).is_ok());

#[cfg(feature = "alloc")]
assert!(dyn_dyn_cast!(move BaseTrait => ExposedTrait, Box::new(s)).is_ok());
```

## Limitations

Currently, `dyn-dyn` only works in nightly versions of Rust due to its use of the unstable `generic_associated_types`, `ptr_metadata`, and `unsize` features, as well as due to its use of several standard library features in `const` contexts.

Due to limitations of `TypeId`, `dyn-dyn` can only currently work with types and traits that are `'static`.

In order to be able to construct a way of downcasting into every possible derived trait that a concrete type wishes to expose, the set of traits exposed using the `#[dyn_dyn_impl(...)]` attribute must be finite. That is, it is not possible to expose some generic trait `Trait<T>` for an arbitrary value of `T` (although it is possible to do so if `T` is constrained by a generic argument to the concrete type or base trait).

## How it works

`dyn-dyn` works by creating a table that maps various `TypeId`s corresponding to traits into the vtable pointer they use for a particular concrete type. This table is then exposed via a hidden supertrait of the base trait, allowing the `dyn_dyn_cast!` macro to dynamically look up the metadata corresponding to a particular trait object. This metadata is then reattached to the pointer using the unstable `ptr_metadata` feature to create a reference to the derived trait object type.
