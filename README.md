# \#\[hackfn\]

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/hackfn-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/hackfn)
[<img alt="crates.io" src="https://img.shields.io/crates/v/hackfn.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/hackfn)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-hackfn-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/hackfn)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dtolnay/hackfn/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/hackfn/actions?query=branch%3Amaster)

Fake implementation of `std::ops::Fn` for user-defined data types.

Place a `#[hackfn]` attribute on an impl block containing a single method to use
that method as the implementation of the function call operator.

```toml
[dependencies]
hackfn = "0.1"
```

*Version requirement: \#\[hackfn\] supports rustc 1.31+*

## Limitations

- The function must receive `&self`. Functions that receive `&mut self` or
  `self` are not supported.

- The function may not have generic parameters or where-clause.

- The `Self` type must implement `Sized`.

## Examples

```rust
use hackfn::hackfn;

/// Function object that adds some number to its input.
struct Plus(u32);

#[hackfn]
impl Plus {
    fn call(&self, other: u32) -> u32 {
        self.0 + other
    }
}

fn main() {
    let plus_one = Plus(1);
    let sum = plus_one(2);
    assert_eq!(sum, 3);
}
```

The next example is somewhat more elaborate:

- Interior mutability can be used to approximate a `FnMut` impl.

- Generic parameters and where-clause are permitted on the impl block (though
  not on the function).

- The function may take any number of arguments.

```rust
use hackfn::hackfn;

use std::cell::Cell;
use std::ops::Add;

/// Function object that accumulates a pair of values per call.
#[derive(Default)]
struct AccumulatePairs<T> {
    first: Cell<T>,
    second: Cell<T>,
}

#[hackfn]
impl<T> AccumulatePairs<T> where T: Copy + Add<Output = T> {
    fn call(&self, first: T, second: T) {
        self.first.set(self.first.get() + first);
        self.second.set(self.second.get() + second);
    }
}

fn main() {
    let accumulate = AccumulatePairs::default();
    accumulate(30, 1);
    accumulate(20, 2);
    accumulate(10, 3);
    assert_eq!(accumulate.first.get(), 60);
    assert_eq!(accumulate.second.get(), 6);
}
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
