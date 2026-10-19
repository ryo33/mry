#[mry::mry]
#[derive(Debug, Clone, PartialEq)]
struct Results {
    values: Vec<String>,
}

#[mry::mry]
impl Results {
    fn merge(self, other: Results) -> Results {
        let mut values = self.values;
        values.extend(other.values);
        mry::new!(Results { values })
    }
}

#[test]
fn unique_is_not_equal_to_another_unique() {
    assert_ne!(mry::unique!(Results), mry::unique!(Results));
}

#[test]
fn unique_is_equal_to_its_clone() {
    let unique = mry::unique!(Results);

    assert_eq!(unique.clone(), unique);
}

#[test]
fn unique_is_not_equal_to_new() {
    assert_ne!(mry::unique!(Results), mry::new!(Results));
}

#[test]
fn new_is_still_equal_to_new() {
    assert_eq!(mry::new!(Results), mry::new!(Results));
}

#[test]
fn unique_fills_fields_with_default() {
    let unique = mry::unique!(Results);

    assert!(unique.values.is_empty());
}

#[test]
fn unique_with_fields() {
    let unique = mry::unique!(Results {
        values: vec!["a".into()],
    });

    assert_eq!(unique.values, vec!["a".to_string()]);
    assert_ne!(
        unique,
        mry::unique!(Results {
            values: vec!["a".into()],
        })
    );
}

#[test]
fn unique_is_mockable() {
    let mut left = mry::unique!(Results);
    let right = mry::unique!(Results);
    let expected = mry::unique!(Results);

    let mock_merge = left.mock_merge(right.clone()).returns(expected.clone());

    assert_eq!(left.clone().merge(right), expected);
    mock_merge.assert_called(1);
}

#[mry::mry]
#[derive(Debug, Clone, PartialEq)]
struct Generic<T> {
    value: T,
}

#[test]
fn unique_with_generic_path() {
    let generic = mry::unique!(Generic::<u32>);

    assert_eq!(generic.value, 0);
    assert_ne!(generic, mry::unique!(Generic::<u32>));
}
