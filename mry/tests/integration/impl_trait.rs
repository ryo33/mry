#[mry::mry]
#[derive(Default)]
struct Cat<'a> {
    name: &'a str,
}

#[mry::mry]
impl<'a> Into<String> for Cat<'a> {
    fn into(self) -> String {
        self.name.to_string()
    }
}

#[test]
fn meow_returns_with() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_into().returns_with(|| format!("Called"));

    assert_eq!(<Cat as Into<String>>::into(cat), "Called".to_string());
}
