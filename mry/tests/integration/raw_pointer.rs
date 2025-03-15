use mry::send_wrapper::SendWrapper;

#[mry::mry]
struct Test {}

#[mry::mry]
impl Test {
    fn get_raw_pointer(&self) -> *mut String {
        Box::into_raw(Box::new(String::from("Hello, world!")))
    }

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

#[test]
fn test_raw_pointer_return() {
    let mut test = Test {
        mry: Default::default(),
    };

    // Create a mock string to return
    let mock_ptr = Box::into_raw(Box::new("Mocked value".to_string()));

    // Mock the get_raw_pointer method to return our mock pointer
    // The mock system should handle wrapping it in SendWrapper internally
    test.mock_get_raw_pointer().returns(mock_ptr);

    // Call the method and verify the result
    let result_ptr = test.get_raw_pointer();

    unsafe {
        assert_eq!(*result_ptr, "Mocked value");

        // Clean up to avoid memory leaks
        let _ = Box::from_raw(result_ptr);
    }
}

#[test]
fn test_raw_pointer_return_once() {
    let mut test = Test {
        mry: Default::default(),
    };

    let mock_ptr = Box::into_raw(Box::new("Mocked value".to_string()));
    test.mock_get_raw_pointer().returns_once(mock_ptr);

    let result_ptr = test.get_raw_pointer();

    unsafe {
        assert_eq!(*result_ptr, "Mocked value");

        // Clean up to avoid memory leaks
        let _ = Box::from_raw(result_ptr);
    }
}

#[test]
fn test_raw_pointer_return_with() {
    let mut test = Test {
        mry: Default::default(),
    };

    // Mock the get_raw_pointer method with a dynamic implementation
    test.mock_get_raw_pointer()
        .returns_with(|| Box::into_raw(Box::new("Dynamically created value".to_string())));

    // Call the method and verify the result
    let result_ptr = test.get_raw_pointer();

    unsafe {
        assert_eq!(*result_ptr, "Dynamically created value");

        // Clean up to avoid memory leaks
        let _ = Box::from_raw(result_ptr);
    }
}
