#[allow(dead_code)]
#[mry::mry]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    #[track_caller]
    fn meow(&self) -> String {
        panic!("meow");
    }

    fn meow_without_track(&self) -> String {
        panic!("meow");
    }
}

#[test]
#[should_panic(expected = "meow")]
fn meow_panics() {
    let mut cat = Cat {
        name: "Tama".to_string(),
        mry: Default::default(),
    };
    cat.mock_meow().calls_real_impl();
    cat.meow();
}

#[test]
#[should_panic(expected = "meow")]
fn meow_without_track_panics() {
    let mut cat = Cat {
        name: "Tama".to_string(),
        mry: Default::default(),
    };
    cat.mock_meow_without_track().calls_real_impl();
    cat.meow_without_track();
}
