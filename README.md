[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Crates.io](https://img.shields.io/crates/v/case_iterable.svg)](https://crates.io/crates/case_iterable)

# `case_iterable`

## Installation

`case_iterable` can be installed with `cargo`:

```
$ cargo add case_iterable
```

or by manually adding it to your `Cargo.toml`:

```toml
[dependencies]
case_iterable = "0.1.0"
```

## Usage

```rust
use case_iterable::CaseIterable;

#[derive(CaseIterable)]
enum Foo {
    A,
    Bar,
    Chocolate,
}

for variant in Foo::all_cases() {
    // Foo::A, Foo::Bar, Foo::Chocolate ...
}
```
