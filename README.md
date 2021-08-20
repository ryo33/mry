# Mry

[![GitHub](https://img.shields.io/badge/GitHub-ryo33/mry-222222)](https://github.com/ryo33/mry)
![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/mry)](https://crates.io/crates/mry)
[![docs.rs](https://img.shields.io/docsrs/mry)](https://docs.rs/mry)

Super simple mocking library that supports **structs** and **traits**.

Issues and pull requests are welcome!

## Features

* Completely no need of switching a mock object and a real object such as in a way using `#[cfg(test)]`.
* Supports mocking for `impl for YourStruct`, `impl SomeTrait for YourStruct`, and `trait YourTrait`.
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

#[mry::mry] // Also mocking of impl trait is supported!
impl Into<&'static str> for Cat {
    fn into(self) -> &'static str {
        self.name
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

// Also a helper struct MryCat is generated
Cat::from(MryCat { name: "Tama" }); // From/Into trait
MryCat { name: "Tama" }.mry(); // or mry(self) -> Cat

// If you derive or impl Default trait.
Cat::default();
Cat { name: "Tama", ..Default::default() };
```

Now you can mock it.

```rust
// mock it
cat.mock_meow().returns("Called".into()); // the shortest
cat.mock_meow().returns_when(3, format!("Called with 3")); // matches by value
cat.mock_meow().behaves(|count| format!("Called with {}", count));
cat.mock_meow().behaves_when(3, |count| format!("Called with {}", count)); // the longest

// call it
assert_eq!(cat.meow(2), "Called with 2".to_string());

// assert it
cat.mock_meow().assert_called(); // the shortest
cat.mock_meow().assert_called_with(2); // matches by value
cat.mock_meow().assert_called_with().times(1); // exactly called 1 time
cat.mock_meow().assert_called().times_within(0..100); // or within the range
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

    cat.mock_meow().calls_real_impl();

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

Now you can use `MockCat` as a mock object.

```rust
// You can construct it by Default trait
let mut cat = MockCat::default();

// API's are the same as struct mock
cat.mock_meow().returns("meow".into());

assert_eq!(cat.meow(2), "Called with 2".to_string());
```

Or you can mock a trait by manually creating a mock struct.

```rust
#[mry::mry]
#[derive(Default)]
struct MockIterator {
}

#[mry::mry]
impl Iterator for MockIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
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
