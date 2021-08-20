#[mry::mry]
#[derive(Default)]
struct MyIterator {}

#[mry::mry]
impl Iterator for MyIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[test]
fn with_mock() {
    let mut cat = MyIterator::default();

    cat.mock_next().returns(Some(1));

    assert_eq!(cat.next(), Some(1));
}
