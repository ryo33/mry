#[mry::mry]
#[derive(Debug, PartialEq)]
struct Cat {
    name: String,
    age: u8,
}

#[mry::mry]
impl Cat {
    fn meow(&self, count: usize) -> String {
        format!("{}: {}", self.name, "meow".repeat(count))
    }
}

#[test]
fn new_with_path_fills_fields_with_default() {
    let cat = mry::new!(Cat);

    assert_eq!(cat.name, "");
    assert_eq!(cat.age, 0);
}

#[test]
fn new_with_path_is_mockable() {
    let mut cat = mry::new!(Cat);

    let mock_meow = cat.mock_meow(mry::Any).returns("Called".to_string());

    assert_eq!(cat.meow(2), "Called".to_string());
    mock_meow.assert_called(1);
}

#[mry::mry]
struct Generic<T> {
    value: T,
}

#[test]
fn new_with_generic_path() {
    let generic = mry::new!(Generic::<u32>);

    assert_eq!(generic.value, 0);
}

struct NotDefault;

// #[mry::mry] must not fail for structs with non-Default fields.
// Calling `mry::new!(HasNotDefault)` fails to compile; see the doc test of `mry::MockDefault`.
#[mry::mry]
#[allow(dead_code)]
struct HasNotDefault {
    name: String,
    not_default: NotDefault,
}

#[test]
fn struct_with_not_default_field_is_constructible() {
    let _value = mry::new!(HasNotDefault {
        name: "Tama".into(),
        not_default: NotDefault,
    });
}
