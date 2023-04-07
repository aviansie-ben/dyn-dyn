# Changelog

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
