use std::time::Duration;

use async_std::task::sleep;

#[mry::mry]
#[derive(Default, PartialEq)]
struct Cat {
    name: String,
}

#[mry::mry]
impl<ToString: Into<String>> Into<ToString> for Cat {
    fn into(self) -> String {
        todo!()
    }
}

#[async_std::test]
async fn meow_behaves() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_into().behaves(|| format!("Called"));

    assert_eq!(<Cat as Into<String>>::into(cat), "Called".to_string());
}
