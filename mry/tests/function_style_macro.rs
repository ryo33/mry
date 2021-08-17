mry::m! {
    #[derive(Default, PartialEq)]
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
fn meow_behaves() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow()
        .behaves(|count| format!("Called with {}", count));

    assert_eq!(cat.meow(2), "Called with 2".to_string());
}

#[test]
fn assert_called() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow()
        .behaves(|count| format!("Called with {}", count));

    cat.meow(2);

    cat.mock_meow().assert_called();
}
