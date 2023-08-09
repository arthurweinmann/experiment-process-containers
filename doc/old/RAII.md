# RAII: Resource Acquisition is Initialisation

- https://doc.rust-lang.org/rust-by-example/scope/raii.html

### Cannot return reference to variable created inside func since it will be destroyed

https://stackoverflow.com/questions/39550758/is-it-possible-to-return-a-reference-created-inside-function-scope

Use a box if afraid of copy: https://doc.rust-lang.org/std/boxed/struct.Box.html

## How to cheat rust compiler

### std::mem::forget

Takes ownership and "forgets" about the value without running its destructor.

- https://doc.rust-lang.org/std/mem/fn.forget.html

### Leak memory

- https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak

### Get raw pointer to memory

- https://doc.rust-lang.org/std/boxed/struct.Box.html#method.into_raw

### Dispose of a value running its destructor

- https://doc.rust-lang.org/std/mem/fn.drop.html