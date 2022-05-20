# Guerrilla
[![Crates.io](https://img.shields.io/crates/v/guerrilla.svg?style=popout-square)](https://crates.io/crates/guerrilla)
[![Travis (.com)](https://img.shields.io/travis/com/mehcode/guerrilla.svg?label=Travis%20CI&style=flat-square)](https://travis-ci.com/mehcode/guerrilla)
[![AppVeyor](https://img.shields.io/appveyor/ci/mehcode/guerrilla.svg?label=AppVeyor&style=flat-square)](https://ci.appveyor.com/project/mehcode/guerrilla)
![License](https://img.shields.io/crates/l/guerrilla.svg?style=popout-square)
> Guerrilla (or Monkey) Patching in Rust for (unsafe) fun and profit.

Provides aribtrary monkey patching in Rust. Please do not use this crate for anything outside of testing.
It will not end well.

Can patch (almost) any function in Rust (free, associated, instance, generic, etc.).

## Usage

```rust
extern crate guerrilla;

#[inline(never)]
fn say_hi(name: &str) {
    println!("hello, {}", name);
}

fn main() {
    // Variadic generics would be wondeful so we could have a [guerrilla::patch]
    let _guard = guerrilla::patch1(say_hi, |name| {
        // So sneaky... like a sneaky sneaky snek
        println!("bye, {}", name);
    });

    // [...]
    // Thousands of lines codes further in the project
    // [...]

    say_hi("Steve");

    // Once the guard is dropped the patch reverts the function to its original
    // behavior.
    drop(_guard);

    say_hi("Bob");
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Credits

Inspired (and derived) from [monkey-patching-in-go](https://bou.ke/blog/monkey-patching-in-go/).
