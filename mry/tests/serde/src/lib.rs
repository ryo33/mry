use serde::{Deserialize, Serialize};

#[mry::mry]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    fn meow(&self, count: usize) -> String {
        format!("{}: {}", self.name, "meow".repeat(count))
    }
}

#[test]
fn serde() {
    let cat: Cat = mry::new!(Cat {
        name: "Tama".into(),
        ..Default::default()
    });
    assert_eq!(cat.meow(2), "Tama: meowmeow".to_string());

    let serialized = serde_json::to_string(&cat);
    assert!(serialized.is_ok());
    assert_eq!(serialized.unwrap(), r#"{"name":"Tama"}"#);

    let serialized = serde_json::to_string(&cat);
    let deserialized = serde_json::from_str::<Cat>(serialized.unwrap().as_str());
    assert!(deserialized.is_ok());
    assert_eq!(
        deserialized.unwrap(),
        mry::new!(Cat {
            name: "Tama".into(),
            ..Default::default()
        })
    );
}
