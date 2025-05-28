#[mry::mry]
#[derive(Default, PartialEq)]
struct Cat {
    name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
struct A<T>(T);

#[mry::mry]
impl Cat {
    fn meow(&self, string: &mut String) -> String {
        *string = self.name.to_string();
        string.clone()
    }
}

#[test]
fn meow_returns_with() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow("aaa".to_string())
        .returns_with(|string| format!("Called with {string}"));

    assert_eq!(
        cat.meow(&mut "aaa".to_string()),
        "Called with aaa".to_string()
    );
}
