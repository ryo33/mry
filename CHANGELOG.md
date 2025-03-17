# Changelog

## 0.13

- Added `#[mry::mry(skip_args(A, B, ...))]` attribute to skip types.
- Added `#[mry::mry(skip_fns(method_name))]` attribute to skip some methods in trait or impl block.

## 0.12

- Now raw pointer types in mock arg or output are wrapped automatically in `SendWrapper` in the background. This makes mry to support raw pointer types.
- Added `#[mry::mry(non_send(A, B, ...)]` attribute to specify which types should be wrapped in `SendWrapper`.
- Previously, calling the original static function in the `returns_with` for the static function leads deadlock. Since `0.12` with the same situation, it calls the real function.
