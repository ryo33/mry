use std::rc::Rc;

#[mry::mry]
#[derive(Default)]
struct Cat {
    name: String,
}

#[mry::mry(skip_methods(skipped))]
impl Cat {
    fn meow(&self) -> String {
        format!("{} meows", self.name)
    }

    fn skipped(&self, rc: Rc<String>) -> String {
        rc.to_string()
    }
}

#[mry::mry(skip_methods(skipped))]
trait SkipTrait {
    fn not_skipped(&self) -> String;
    fn skipped(&self, rc: Rc<String>) -> String;
}

#[mry::mry(skip_methods(skipped))]
impl SkipTrait for Cat {
    fn not_skipped(&self) -> String {
        format!("{} meows", self.name)
    }

    fn skipped(&self, rc: Rc<String>) -> String {
        rc.to_string()
    }
}

#[test]
fn test_skip_in_impl() {
    let mut cat = mry::new!(Cat {
        name: "Tama".to_string()
    });
    cat.mock_meow().returns("mocked".to_string());
    assert_eq!(cat.meow(), "mocked");

    assert_eq!(cat.skipped(Rc::new("a".to_string())), "a");
}

#[test]
fn test_not_skipped_in_trait() {
    let mut mock = MockSkipTrait::default();
    mock.mock_not_skipped().returns("mocked".to_string());
    assert_eq!(mock.not_skipped(), "mocked");
}

#[test]
#[should_panic(expected = "this method is skipped with `#[mry::mry(skip_methods(...))]` attribute")]
fn test_skipped_in_trait() {
    let mock = MockSkipTrait::default();
    mock.skipped(Rc::new("a".to_string()));
}

#[test]
fn test_not_skipped_in_trait_impl() {
    let mut mock = Cat::default();
    mock.mock_not_skipped().returns("mocked".to_string());

    assert_eq!(mock.not_skipped(), "mocked");
}

#[test]
fn test_skipped_in_trait_impl() {
    let mock = Cat::default();
    assert_eq!(mock.skipped(Rc::new("a".to_string())), "a");
}
