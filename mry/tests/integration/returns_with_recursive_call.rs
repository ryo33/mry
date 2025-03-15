#[mry::mry]
#[derive(Default, Clone)]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    fn name_len(&self) -> usize {
        self.name.len()
    }

    fn name_len_static(name: &str) -> usize {
        name.len()
    }
}

#[test]
fn test_name_len_with_clone() {
    let mut cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    let cat2 = cat.clone();
    cat.mock_name_len().returns_with(move || cat2.name_len());
    assert_eq!(cat.name_len(), 4);
}

#[test]
#[mry::lock(Cat::name_len_static)]
fn test_name_len_static() {
    Cat::mock_name_len_static("Tama").returns_with(|name: String| Cat::name_len_static(&name));
    assert_eq!(Cat::name_len_static("Tama"), 4);
}
