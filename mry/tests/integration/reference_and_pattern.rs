#[mry::mry]
#[derive(Default, PartialEq)]
struct Cat {
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
struct A<T>(T);

#[mry::mry]
impl Cat {
    fn meow(&self, base: &'static str, A(count): A<usize>, _: String) -> String {
        format!("{}: {}", self.name, base.repeat(count))
    }
}

#[test]
fn meow_returns_with() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow("aaa", A(2), "bbb")
        .returns_with(|base, count, string| format!("Called with {base} {count:?} {string}"));

    assert_eq!(
        cat.meow("aaa", A(2), "bbb".into()),
        "Called with aaa A(2) bbb".to_string()
    );
}
