use mry::Any;

#[mry::mry]
#[derive(Default, PartialEq)]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    fn meow(&self, count: usize) -> String {
        format!("{}: {}", self.name, "meow".repeat(count))
    }

    fn just_meow(&self) -> String {
        format!("{}: meow", self.name)
    }
}

#[test]
fn keeps_original_function() {
    let cat: Cat = mry::new!(Cat {
        name: "Tama".into(),
        ..Default::default()
    });
    assert_eq!(cat.meow(2), "Tama: meowmeow".to_string());
}

#[test]
fn meow_returns() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow(Any).returns("Called".to_string());

    assert_eq!(cat.meow(2), "Called".to_string());
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
    let meow = cat.mock_meow(Any).returns("Called".into());

    cat.meow(2);

    meow.assert_called(1);
}

#[test]
fn assert_called_0_times() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    let meow = cat.mock_meow(Any).returns("Called".into());
    meow.assert_called(0);
}

#[test]
#[should_panic]
fn assert_called_fails() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow(3usize)
        .returns_with(|count| format!("Called with {}", count));
    let meow = cat.mock_meow(2).returns("Called".into());

    cat.meow(3);

    meow.assert_called(1);
}

#[test]
fn just_meow_returns_with() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_just_meow().returns_with(|| "Called".into());

    assert_eq!(cat.just_meow(), "Called".to_string());
}

#[test]
fn just_meow_returns() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_just_meow().returns("Called".into());

    assert_eq!(cat.just_meow(), "Called".to_string());
}

#[test]
fn times() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    let meow = cat.mock_meow(Any).returns("Called".into());
    cat.meow(2);
    cat.meow(2);
    meow.assert_called(2);
}

#[test]
fn times_within() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    let meow = cat.mock_just_meow().returns("Called".to_string());
    cat.just_meow();
    cat.just_meow();

    meow.assert_called(2..3);
}

#[test]
fn returns_once_not_clone_value() {
    #[mry::mry]
    #[derive(Default, PartialEq)]
    struct Cat {
        name: String,
    }

    #[mry::mry]
    impl Cat {
        fn meow(&self, count: usize) -> NotClone {
            todo!()
        }
    }

    pub struct NotClone;

    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    let meow = cat.mock_meow(0).returns_once(NotClone);

    cat.meow(0);
    meow.assert_called(1);
}

#[test]
fn reuse_locator() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };

    let locator = cat.mock_meow(Any).returns("Called".into());

    cat.meow(2);

    locator.assert_called(1);
}
