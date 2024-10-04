# `case_iterable`

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Crates.io](https://img.shields.io/crates/v/case_iterable.svg)](https://crates.io/crates/case_iterable)

## Installation

`case_iterable` can be installed with `cargo`:

```
$ cargo add case_iterable
```

or by manually adding it to your `Cargo.toml`:

```toml
[dependencies]
case_iterable = "0.2.0"
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
    // Foo::A
    // Foo::Bar
    // Foo::Chocolate
}

// also exposes the next function used for the iterator
let x = Foo::Bar;
let y = x.next();  // Some(Foo::Chocolate)
```

#### License

<sup>
Licensed under <a href="LICENSE">GNU General Public License, Version 3.0</a>
</sup>

<sub>
This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.
</sub>
