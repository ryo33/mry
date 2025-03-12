use mry::send_wrapper::SendWrapper;

#[mry::mry]
struct Test {}

#[mry::mry]
impl Test {
    // fn get_raw_pointer(&self) -> *mut String {
    //     Box::into_raw(Box::new(String::from("Hello, world!")))
    // }

    fn use_ptr(&self, ptr: *const String, data: u8) -> u8 {
        unsafe {
            println!("{} {}", *ptr, data);
        }
        data
    }

    fn use_mut_ptr(&self, ptr: *mut String, data: u8) -> u8 {
        unsafe {
            println!("{} {}", *ptr, data);
        }
        data
    }
}

#[test]
fn test_use_pointer() {
    let mut test = Test {
        mry: Default::default(),
    };
    let ptr1 = Box::into_raw(Box::new(String::from("Hello, world!")));
    test.mock_use_ptr(ptr1 as *const String, 1)
        .returns_with(|_: SendWrapper<*const String>, value| value * 10);
    test.mock_use_ptr(mry::Any, mry::Any).calls_real_impl();
    test.mock_use_mut_ptr(ptr1, 2)
        .returns_with(|_: SendWrapper<*mut String>, value| value * 10);
    test.mock_use_mut_ptr(mry::Any, mry::Any).calls_real_impl();
    let ptr2 = Box::into_raw(Box::new(String::from("Hello, world!"))); // same contents

    assert_eq!(test.use_ptr(ptr1, 1), 10);
    assert_eq!(test.use_mut_ptr(ptr1, 2), 20);
    assert_eq!(test.use_ptr(ptr2, 5), 5);
    assert_eq!(test.use_mut_ptr(ptr2, 6), 6);
}
