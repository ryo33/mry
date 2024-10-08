#[mry::mry]
struct Struct {}

#[mry::mry]
impl Struct {
    fn takes_slice(&self, value: &[String]) -> usize {
        value.iter().map(|i| i.len()).sum()
    }

    fn takes_slice_str(&self, value: &[&str]) -> usize {
        value.iter().map(|i| i.len()).sum()
    }
}
#[test]
fn takes_slice_any() {
    let mut target = mry::new!(Struct {});
    let mock = target.mock_takes_slice(mry::Any).returns(1);

    assert_eq!(target.takes_slice(&["first arg".to_string()]), 1);
    mock.assert_called(1);
}
#[test]
fn takes_slice_original_type_ok() {
    let mut target = mry::new!(Struct {});
    let mock = target
        .mock_takes_slice(&["first arg".to_string(), "second arg".to_string()][..])
        .returns(1);

    assert_eq!(
        target.takes_slice(&["first arg".to_string(), "second arg".to_string()]),
        1
    );
    mock.assert_called(1);
}

#[should_panic]
#[test]
fn takes_slice_original_type_ko() {
    let mut target = mry::new!(Struct {});
    let mock = target
        .mock_takes_slice(&["first arg".to_string(), "second arg".to_string()][..])
        .returns(1);

    assert_eq!(
        target.takes_slice(&["first arg".to_string(), "wrong value".to_string()]),
        1
    );
    mock.assert_called(1);
}

#[test]
fn takes_slice_str_any() {
    let mut target = mry::new!(Struct {});
    let mock = target.mock_takes_slice_str(mry::Any).returns(1);

    assert_eq!(target.takes_slice_str(&["first arg"]), 1);
    mock.assert_called(1);
}
#[test]
fn takes_slice_str_original_type_ok() {
    let mut target = mry::new!(Struct {});
    let mock = target
        .mock_takes_slice_str(&["first arg", "second arg"][..])
        .returns(1);

    assert_eq!(target.takes_slice_str(&["first arg", "second arg"]), 1);
    mock.assert_called(1);
}

#[should_panic]
#[test]
fn takes_slice_str_original_type_ko() {
    let mut target = mry::new!(Struct {});
    let mock = target
        .mock_takes_slice_str(&["first arg", "second arg"][..])
        .returns(1);

    assert_eq!(target.takes_slice_str(&["first arg", "wrong value"]), 1);
    mock.assert_called(1);
}

#[should_panic]
#[test]
fn takes_slice_str_original_type_wrong_length() {
    let mut target = mry::new!(Struct {});
    let mock = target
        .mock_takes_slice_str(&["first arg", "second arg"][..])
        .returns(1);

    assert_eq!(target.takes_slice_str(&["first arg"]), 1);
    mock.assert_called(1);
}
