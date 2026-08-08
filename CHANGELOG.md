# Changelog

## Version 0.2.1

- Fixed to build properly with newer Nightlies that replace the `doc_auto_cfg` feature with `doc_cfg`
- Fixed an issue where `dyn-dyn` could select an incompatible version of the internal `dyn-dyn-macros` crate
- Removed support for marker traits on the base trait of a `dyn_dyn_cast!` call
- Removed support for explicit lifetimes in `dyn_dyn_cast!` since they're not useful and don't always work correctly

## Version 0.2.0

- Loosened bounds for `UncheckedDowncast` to remove its dependency on `DynDynTarget`
- Remove the unnecessary `B` generic parameter from `UncheckedDowncast`
- Fixed to build properly with newer Nightlies where the `effects` feature is being reworked
- Removed the ability to use marker traits in `dyn_dyn_cast!` since this is no longer sound when combined with the `arbitrary_self_types` feature

## Version 0.1.2

- Fixed to build properly with newer Nightlies that removed the `const_convert` feature
- Improved the macro hygiene of `dyn_dyn_impl` so it can now be used within macros

## Version 0.1.1

- Fixed to build properly with newer Nightlies that require `#[const_trait]`
- Implemented downcasting of DST types containing a base trait if a manual unsafe `DynDynBase` implementation is provided
- Improved error messages for common mistakes in `dyn_dyn_cast!`

## Version 0.1.0

- Implemented downcasting of smart pointers by reworking the `DynDyn` trait
- Switched numerous methods to taking/returning `DynMetadata<_>` instead of `AnyDynMetadata`
- Changed `dyn_dyn_cast!` to return a `Result` instead of an `Option`
- Exported the previously hidden `AnyDynMetadata` struct, and the `DynDynCastTarget` and `DynDynBase` traits
- Renamed the `#[dyn_dyn_derived]` attribute to `#[dyn_dyn_impl]`

## Version 0.1.0-alpha.2

- Fixed `dyn_dyn_cast!` not properly keeping temporaries live in the value passed in

## Version 0.1.0-alpha.1

- Initial release
