#[mry::mry]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
fn cat_can_serialize() {
    let cat: Cat = mry::new!(Cat {
        name: "Tama".into(),
        ..Default::default()
    });
    assert_eq!(cat.meow(2), "Tama: meowmeow".to_string());

    let serialized = serde_json::to_string(&cat);
    assert!(serialized.is_ok());
    assert_eq!(serialized.unwrap(), r#"{"name":"Tama"}"#)
}