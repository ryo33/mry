/// Constructs a mock object with all fields set to `Default::default()`.
///
/// `#[mry::mry]` implements this for every struct, but it is callable only when all fields implement `Default`.
/// Use `mry::new!(Cat)` instead of calling this directly.
///
/// ```
/// #[mry::mry]
/// struct Cat {
///     name: String,
/// }
///
/// let cat = mry::new!(Cat);
/// assert_eq!(cat.name, "");
/// ```
///
/// A struct with a field that does not implement `Default` can still have `#[mry::mry]`,
/// but `mry::new!(Cat)` fails to compile.
///
/// ```compile_fail
/// struct NotDefault;
///
/// #[mry::mry]
/// struct Cat {
///     name: NotDefault,
/// }
///
/// let cat = mry::new!(Cat);
/// ```
pub trait MockDefault {
    fn mock_default() -> Self;
}
