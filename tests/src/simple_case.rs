#[mry::mry]
#[derive(Default)]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    fn meow(&self, count: usize) -> String {
        "meow".repeat(count)
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
fn assert_called_with() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow()
        .behaves(|count| format!("Called with {}", count));

    cat.meow(2);

    cat.mock_meow().assert_called_with(2);
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

#[test]
#[should_panic]
fn assert_called_with_fails() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow()
        .behaves(|count| format!("Called with {}", count));

    cat.meow(3);

    cat.mock_meow().assert_called_with(2);
}

#[test]
fn meow_behaves_when() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow()
        .behaves_when(3, |count| format!("Called with {}", count));

    assert_eq!(cat.meow(3), "Called with 3".to_string())
}
