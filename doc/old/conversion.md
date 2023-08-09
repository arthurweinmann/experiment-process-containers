## String to u32

```RUST
let mut guess = String::new();

// [...]

// variable shadowing
//The parse method on strings parses a string into some kind of number. Because this method can parse a variety of number types, we need to tell Rust the exact number type we want by using let guess: u32. The colon (:) after guess tells Rust we’ll annotate the variable’s type.
let guess: u32 = guess.trim().parse()
    .expect("Please type a number!");
```