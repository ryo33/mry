# Mry

[![GitHub](https://img.shields.io/badge/GitHub-ryo33/mry-222222)](https://github.com/ryo33/mry)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/mry)](https://crates.io/crates/mry)
[![docs.rs](https://img.shields.io/docsrs/mry)](https://docs.rs/mry)

A cfg-free mocking library for **structs** and **traits**, which supports **partial mocks**.

## Features

* No need of switching between mock objects and real objects such as the way using `#[cfg(test)]`.
* Supports mocking for `impl Struct`, `impl Trait for Struct`, and `trait Trait`.
* Supports partial mocking.

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

`#[mry::mry]` adds a visible but ghostly field `mry`  your struct, so your struct must be constructed by the following ways.

```rust
// An easy way
mry::new!(Cat { name: "Tama" })

// is equivalent to:
Cat {
    name: "Tama",
    #[cfg(test)]
    mry: Default::default(),
};

// If you derive or impl Default trait.
Cat::default();
Cat { name: "Tama", ..Default::default() };
```

Now you can mock it by using following functions:

- `returns(...)`: Makes a mock to returns a constant value.
- `ruturns_with(|arg| ...)`: Makes a mock to returns a value with closure (This allows returning `!Clone` unlike `returns`.
- `assert_called()`: Assert that a mock was called with correct arguments and times.

### Examples

```rust
cat.mock_meow(3).returns("Returns this string when called with 3".into());
cat.mock_meow(mry::Any).returns("This string is returned for any value".into());
cat.mock_meow(mry::Any).returns_with(|count| format!("Called with {}", count)); // return a dynamic value
```

```rust
cat.mock_meow(3).assert_called(); // Assert called with 3
cat.mock_meow(mry::Any).assert_called(); // Assert called with any value
cat.mock_meow(3).assert_called().times(1); // exactly called 1 time with 3
cat.mock_meow(3).assert_called().times_within(0..100); // or within the range
```

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

Just add `#[mry::mry]` as we did with mocking a struct,

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

// API's are the same as struct mock
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

## Rust Analyzer

Currently comprehensive support of proc macros is not available in rust-analyzer,
so above examples are not fully recognized by rust-analyzer and completions and type hints are inconvenient.

You can support them via [GitHub Sponsors](https://github.com/sponsors/rust-analyzer) or [Open Collective](https://opencollective.com/rust-analyzer).

Also, we can contribute to it on [GitHub](https://github.com/rust-analyzer/rust-analyzer).
