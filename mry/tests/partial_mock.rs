#[mry::mry]
#[derive(Default)]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    fn meow(&self, count: usize) -> String {
        self.meow_single().repeat(count)
    }

    fn meow_single(&self) -> String {
        "meow".into()
    }
}

#[test]
fn partial_mock() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };

    cat.mock_meow_single().returns("hello".to_string());

    cat.mock_meow(2).calls_real_impl();

    assert_eq!(cat.meow(2), "hellohello".to_string());
}
