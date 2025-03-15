use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
struct A(*const ());
#[derive(Debug, Clone, PartialEq)]
struct B;

#[mry::mry]
#[derive(Default)]
struct Cat {
    #[expect(dead_code)]
    name: String,
}

#[mry::mry(not_send(A, Rc))]
impl Cat {
    fn meow_a(&self, a: A) -> String {
        "meow".to_string()
    }

    fn meow_rc(&self) -> Rc<String> {
        Rc::new("meow".to_string())
    }

    fn meow_b(&self, b: B) -> B {
        b
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::null;

    use mry::send_wrapper::SendWrapper;

    use super::*;

    #[test]
    fn test_meow_a() {
        let mut cat = Cat {
            name: "meow".to_string(),
            ..Default::default()
        };
        cat.mock_meow_a(A(null()))
            .returns_with(|_: SendWrapper<A>| "mocked".to_string());
        assert_eq!(cat.meow_a(A(null())), "mocked");
    }

    #[test]
    fn test_meow_rc() {
        let mut cat = Cat {
            name: "meow".to_string(),
            ..Default::default()
        };
        cat.mock_meow_rc()
            .returns_with(|| Rc::new("mocked".to_string()));
        assert_eq!(cat.meow_rc(), Rc::new("mocked".to_string()));
    }

    #[test]
    fn test_meow_b() {
        let mut cat = Cat {
            name: "meow".to_string(),
            ..Default::default()
        };
        cat.mock_meow_b(B).returns_with(|_: B| B);
        assert_eq!(cat.meow_b(B), B);
    }
}
