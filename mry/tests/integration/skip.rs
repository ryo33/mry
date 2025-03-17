use std::rc::Rc;

#[mry::mry]
struct Cat {}

struct NonCloneable;

#[derive(Clone, PartialEq, Debug)]
struct A;

#[mry::mry(skip(NonCloneable, Rc, A))]
impl Cat {
    fn many_args(&self, non_clonable: NonCloneable, rc: Rc<String>, count: usize) -> String {
        let _ = non_clonable;
        rc.to_string().repeat(count)
    }

    fn skip_return(&self) -> A {
        A
    }
}

#[mry::mry(skip(Rc))]
fn skip_rc(rc: Rc<String>) -> String {
    rc.to_string()
}

#[mry::mry(skip(Rc))]
fn hello(rc: Rc<String>, count: usize) -> String {
    rc.to_string().repeat(count)
}

#[test]
fn test_many_args() {
    let mut cat = mry::new!(Cat {});
    cat.mock_many_args(2).returns("mocked".to_string());

    assert_eq!(
        cat.many_args(NonCloneable, Rc::new("a".to_string()), 2),
        "mocked"
    );
}

#[test]
#[mry::lock(skip_rc)]
fn test_skip_rc() {
    mock_skip_rc().returns("mocked".to_string());
}

#[test]
#[mry::lock(hello)]
fn test_hello() {
    mock_hello(2).returns_with(|num| "mocked".repeat(num));
    assert_eq!(hello(Rc::new("aaa".into()), 2), "mockedmocked");
}

#[test]
fn test_skip_return_no_effect() {
    let mut cat = mry::new!(Cat {});
    cat.mock_skip_return().returns(A);
    assert_eq!(cat.skip_return(), A);
}
