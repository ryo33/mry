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

pub struct NewType(String);

#[mry::mry]
impl From<String> for NewType {
    fn from(from: String) -> Self {
        NewType(from)
    }
}

#[test]
fn meow_returns_with() {
    let mut cat: Cat = Cat {
        name: "Tama",
        ..Default::default()
    };
    cat.mock_into().returns_with(|| "Called".to_string());

    assert_eq!(<Cat as Into<String>>::into(cat), "Called".to_string());
}

#[test]
#[mry::lock(<NewType as From<String>>::from)]
fn mock_from_trait() {
    NewType::mock_from(mry::Any).returns_once(NewType("Called".to_string()));
    let new_type: NewType = "Hello".to_string().into();
    assert_eq!(new_type.0, "Called".to_string());
}
