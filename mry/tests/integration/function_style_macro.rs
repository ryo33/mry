mry::m! {
    #[derive(Default)]
    struct Cat {
        name: String,
    }

    impl Cat {
        fn meow(&self, count: usize) -> String {
            format!("{}: {}", self.name, "meow".repeat(count))
        }
    }
}

#[test]
fn meow_returns_with() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow(2)
        .returns_with(|count| format!("Called with {}", count));

    assert_eq!(cat.meow(2), "Called with 2".to_string());
}

#[test]
fn assert_called() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow(2)
        .returns_with(|count| format!("Called with {}", count));

    cat.meow(2);

    cat.mock_meow(2).assert_called();
}
