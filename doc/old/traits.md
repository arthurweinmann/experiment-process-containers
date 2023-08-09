

## Blanket implementation

- https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods:

"We can also conditionally implement a trait for any type that implements another trait. Implementations of a trait on any type that satisfies the trait bounds are called blanket implementations and are extensively used in the Rust standard library. For example, the standard library implements the ToString trait on any type that implements the Display trait. The impl block in the standard library looks similar to this code:

```RUST
impl<T: Display> ToString for T {
    // --snip--
}
```

Because the standard library has this blanket implementation, we can call the to_string method defined by the ToString trait on any type that implements the Display trait. For example, we can turn integers into their corresponding String values like this because integers implement Display:


```RUST
let s = 3.to_string();
```

Blanket implementations appear in the documentation for the trait in the “Implementors” section."