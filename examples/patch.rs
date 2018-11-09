extern crate guerrilla;

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
