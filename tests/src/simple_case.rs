use mry::Mry;

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

    fn new_by_mry_into() -> Self {
        MryCat {
            name: "Tama".into(),
        }
        .into()
    }

    fn new_by_mry_new() -> Self {
        mry::new!(Cat {
            name: "Tama".into(),
        })
    }
}

#[test]
fn mry_cat() {
    let cat: Cat = MryCat {
        name: "Tama".into(),
    }
    .into();
    assert_eq!(cat.mry.id(), None);
}

#[test]
fn mry_new() {
    let cat = mry::new!(Cat {
        name: "Tama".into(),
    });
    assert_eq!(cat.mry.id(), None);
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

#[test]
fn just_meow_behaves() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_just_meow().behaves(|()| "Called".into());

    assert_eq!(cat.just_meow(), "Called".to_string());
}
