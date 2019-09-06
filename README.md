# Multiple Default Trait Implementations (multi-default-trait-impl)

Define multiple default implementations for a trait.

[![Latest Version](https://img.shields.io/crates/v/multi-default-trait-impl.svg)](https://crates.io/crates/multi-default-trait-impl)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/multi-default-trait-impl)

This library contains two macros: `default_trait_impl` which defines a default trait
implementation, and `trait_impl` which uses a default trait implementation you've defined.

This is particularly useful in testing, when many of your mocked types will have very similar
trait implementations, but do not want the canonical default trait implementation to use mocked
values.

## Example

First, define a default trait implementation for the trait `Car`:

```rust
#[default_trait_impl]
impl Car for NewCar {
    fn get_mileage(&self) -> Option<usize> { Some(6000) }
    fn has_bluetooth(&self) -> bool { true }
}
```

`NewCar` does not need to be defined beforehand.

Next, implement the new default implementation for a type:

```rust
struct NewOldFashionedCar;

#[trait_impl]
impl NewCar for NewOldFashionedCar {
    fn has_bluetooth(&self) -> bool { false }
}

struct WellUsedNewCar;
impl NewCar for WellUsedNewCar {
    fn get_mileage(&self) -> Option<usize> { Some(100000) }
}
```

This will ensure that our structs use the `NewCar` defaults, without having to change the
canonical `Car` default implementation:

```rust
fn main() {
    assert_eq!(NewOldFashionedCar.get_mileage(), Some(6000));
    assert_eq!(NewOldFashionedCar.has_bluetooth(), false);
    assert_eq!(WellUsedNewCar.get_mileage(), Some(100000));
    assert_eq!(WellUsedNewCar.has_bluetooth(), true);
}
```


