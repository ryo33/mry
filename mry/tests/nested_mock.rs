#[mry::mry]
struct Cat {
    name: String,
}

#[mry::mry]
#[derive(Clone, Debug)]
struct Name {
    name: String,
}

#[mry::mry]
impl Cat {
    fn name(&self) -> Name {
        mry::new!(Name {
            name: self.name.clone()
        })
    }
}

#[mry::mry]
impl Name {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[test]
fn nested_mock() {
    let mut cat: Cat = mry::new!(Cat {
        name: "Tama".into(),
    });
    let mut name: Name = mry::new!(Name {
        name: "Name".into(),
    });

    name.mock_name().returns("Called".into());
    cat.mock_name().returns(name);

    assert_eq!(cat.name().name(), "Called".to_string());
}
