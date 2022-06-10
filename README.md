# Mry

[![GitHub](https://img.shields.io/badge/GitHub-ryo33/mry-222222)](https://github.com/ryo33/mry)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/mry)](https://crates.io/crates/mry)
[![docs.rs](https://img.shields.io/docsrs/mry)](https://docs.rs/mry)

A simple but powerful mocking library for **structs**, **traits**, and **function**.

## Features

* A really simple and easy API
* No need of switching between mock objects and real objects.
* Supports mocking for structs, traits, and functions.
* Supports partial mocking.

## Example

```rust
#[mry::mry]
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
fn meow_returns() {
    let mut cat = mry::new!(Cat { name: "Tama".into() });

    cat.mock_meow(mry::Any).returns("Called".to_string());

    assert_eq!(cat.meow(2), "Called".to_string());
}
```

## Mocking a struct

We need to add an attribute `#[mry::mry]` in the front of struct definition and impl block to mock them.

```rust
#[mry::mry] // This
struct Cat {
    name: &'static str,
}

#[mry::mry] // And this
impl Cat {
    fn meow(&self, count: usize) -> String {
        format!("{}: {}", self.name, "meow".repeat(count))
    }
}
```

`#[mry::mry]` adds a visible but ghostly field `mry` to your struct, so your struct must be constructed by the following ways.

```rust
// An easy way
mry::new!(Cat { name: "Tama" })

// is equivalent to:
Cat {
    name: "Tama",
    mry: Default::default(),
};

// If you derive or impl Default trait.
Cat::default();
// or
Cat { name: "Tama", ..Default::default() };
```

Now you can mock it by using following functions:

- `mock_*(...).returns(...)`: Makes a mock to return a constant value.
- `mock_*(...).returns_with(|arg| ...)`: Makes a mock to return a value with a closure (This is allowed to return `!Clone` unlike `returns` cannot).
- `mock_*(...).assert_called(...)`: Asserts that a mock was called with correct arguments and times, and returns call logs.

### Examples

```rust
cat.mock_meow(3).returns("Returns this string when called with 3".into());
cat.mock_meow(mry::Any).returns("This string is returned for any value".into());
cat.mock_meow(mry::Any).returns_with(|count| format!("Called with {}", count)); // return a dynamic value
```

```rust
cat.mock_meow(3).assert_called(1); // Assert called exactly 1 time with 3
cat.mock_meow(mry::Any).assert_called(1); // Assert called with any value
cat.mock_meow(3).assert_called(0..100); // or within the range
```

## Release build

When release build, the `mry` field of your struct will be zero size, and `mock_*` functions will be unavailable.

## impl Trait for Struct

Also, mocking of impl trait is supported in the same API.

```rust
#[mry::mry]
impl Into<&'static str> for Cat {
    fn into(self) -> &'static str {
        self.name
    }
}
```

## Partial mocks

You can do partial mocking with using `calls_real_impl()`.

```rust
#[mry::mry]
impl Cat {
    fn meow(&self, count: usize) -> String {
        self.meow_single().repeat(count)
    }

    fn meow_single(&self) -> String {
        "meow".into()
    }
}

#[test]
fn partial_mock() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };

    cat.mock_meow_single().returns("hello".to_string());

    cat.mock_meow(mry::Any).calls_real_impl();

    // not "meowmeow"
    assert_eq!(cat.meow(2), "hellohello".to_string());
}
```

## Mocking a trait

Just add `#[mry::mry]` as before;

```rust
#[mry::mry]
pub trait Cat {
    fn meow(&self, count: usize) -> String;
}
```

Now we can use `MockCat` as a mock object.

```rust
// You can construct it by Default trait
let mut cat = MockCat::default();

// API's are the same as the struct mocks.
cat.mock_meow(2).returns("Called with 2".into());

assert_eq!(cat.meow(2), "Called with 2".to_string());
```

We can also mock a trait by manually creating a mock struct.
If the trait has a generics or associated type, we need to use this way.

```rust
#[mry::mry]
#[derive(Default)]
struct MockIterator {
}

#[mry::mry]
impl Iterator for MockIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
```

## async_trait

Add `#[mry::mry]` with the async_trait attribute underneath.

```rust
#[mry::mry]
#[async_trait::async_trait]
pub trait Cat {
    async fn meow(&self, count: usize) -> String;
}
```

## Mocking a function

Add `#[mry::mry]` to the function definition.

```rust
#[mry::mry]
fn hello(count: usize) -> String {
    "hello".repeat(count)
}
```

We need to acquire a lock of the function by using `#[mry::lock(hello)]` because mocking of static function uses global state.

```rust
#[test]
#[mry::lock(hello)] // This is required!
fn function_keeps_original_function() {
    // Usage is the same as the struct mocks.
    mock_hello(Any).calls_real_impl();

    assert_eq!(hello(3), "hellohellohello");
}
```

## Mocking a associated function (static function)

Include your associated function into the impl block with `#[mry::mry]`.

```rust
struct Cat {}

#[mry::mry]
impl Cat {
    fn meow(count: usize) -> String {
        "meow".repeat(count)
    }
}
```

We need to acquire a lock for the same reason in mocking function above.

```rust
#[test]
#[mry::lock(Cat::meow)] // This is required!
fn meow_returns() {
    // Usage is the same as the struct mocks.
    Cat::mock_meow(Any).returns("Called".to_string());

    assert_eq!(Cat::meow(2), "Called".to_string());
}
```

## Rust Analyzer

Currently comprehensive support of proc macros is not available in rust-analyzer,
so above examples are not fully recognized by rust-analyzer and completions and type hints are inconvenient.

You can support them via [GitHub Sponsors](https://github.com/sponsors/rust-analyzer) or [Open Collective](https://opencollective.com/rust-analyzer).

Also, we can contribute to it on [GitHub](https://github.com/rust-analyzer/rust-analyzer).
