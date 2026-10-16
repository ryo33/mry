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
fn dummy_is_not_equal_to_another_dummy() {
    assert_ne!(mry::dummy!(Results), mry::dummy!(Results));
}

#[test]
fn dummy_is_equal_to_its_clone() {
    let dummy = mry::dummy!(Results);

    assert_eq!(dummy.clone(), dummy);
}

#[test]
fn dummy_is_not_equal_to_new() {
    assert_ne!(mry::dummy!(Results), mry::new!(Results));
}

#[test]
fn new_is_still_equal_to_new() {
    assert_eq!(mry::new!(Results), mry::new!(Results));
}

#[test]
fn dummy_fills_fields_with_default() {
    let dummy = mry::dummy!(Results);

    assert!(dummy.values.is_empty());
}

#[test]
fn dummy_with_fields() {
    let dummy = mry::dummy!(Results {
        values: vec!["a".into()],
    });

    assert_eq!(dummy.values, vec!["a".to_string()]);
    assert_ne!(
        dummy,
        mry::dummy!(Results {
            values: vec!["a".into()],
        })
    );
}

#[test]
fn dummy_is_mockable() {
    let mut left = mry::dummy!(Results);
    let right = mry::dummy!(Results);
    let expected = mry::dummy!(Results);

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
fn dummy_with_generic_path() {
    let generic = mry::dummy!(Generic::<u32>);

    assert_eq!(generic.value, 0);
    assert_ne!(generic, mry::dummy!(Generic::<u32>));
}
