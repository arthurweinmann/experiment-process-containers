

https://www.reddit.com/r/rust/comments/7megb9/returning_a_string_from_a_function_string_or_str/: 

It's actually fairly simple. There are two regions of memory, the stack and the heap. The size of all values on the stack have to be known at compile time. Local variables are on a "stack frame" that gets popped and pushed. When a function returns, the stack frame for that function is destructed. To make a value "outlive" the function that it was created in, you have the heap. Heap memory is not freed when the function returns. The stack is fast, the heap is slow. Memory is cleaned up automatically on the stack, on the heap you have to do it manually (RAII). Variables on the stack cannot be dynamically changed in size, variable on the heap can. So far, so good.

You cannot return a &str if that reference was created inside the function. Why? Because & is a pointer into the stack frame. When the function returns the stack frame goes poof and where would your pointer now point to? Garbage memory. However, if the & points to memory that was in the parent function (the parent stack frame), then it would be ok to return a &str.

Rust automatically creates "invisible lifetimes" on each function, which correspond to one stack frame.

Example:

```
fn something(input: &str) -> &str { return input; }
```

What would be the "invisible lifetimes" on this? Rust knows that you cannot return a pointer into the stack frame, so it has to assume this:

```
fn something<'a>(input: &'a str) -> &'a str { return input; }
```

This is the only thing that would make sense. The input has to live as long as the function itself. The output cannot point to a function-local variable, because when the function returns, that value is destructed (this is a common bug in C++, for example). So it has to assume that the output lives at least as long as the input.

But what if we pass a String? Here you have to know about RAII. As I said, heap memory is not automatically freed. In C you would do this:

```
char* my_string = (char *) malloc(10 * sizeof (char)); // allocate memory on heap
free(my_string); // free memory on heap
// also: you can still access my_string (points to garbage)
```

This is dangerous, because these two lines can be hundreds of lines of code apart from each other, if something goes wrong between them, the free() function may never called (bad). In Rust (and C++), the "free" happens automatically when the scope (i.e. the curly braces) end. So:

```
{
     let my_string = String::with_capacity(10); // allocate
} // my_string goes out of scope here, automatic free() called
```

This is called "RAII" - an object (in this case String) contains a pointer to memory on the heap. Meanwhile the String is passed around on the stack. When it goes out of scope, the heap memory is freed. You can never forget to free memory this way. But let's look at this:

```
fn something<'a>(input: String) -> &'a str { 
    return &input; 
 } // memory of input is free'd! return value would point to garbage!
 ```

This will never compile. Why? Well, where should the &str point to? It can only point to something passed into the function, in this case the String, which is destructed when the function returns, it would point to garbage. Rust catches this because the "&input has to live as least as long as the 'a on the function" (the invisible lifetime).

So TLDR:

- if you created the string from inside the function, return String

- if the function borrowed the string, you can use &str. You can copy the value in &str and make a new String, via copying the contents. This is possibly not performant.

- &String can be coerced to &str. The compiler can accept a &String, even though the function wants a &str. But not the other way around.

But there is no general rule. You need to understand the stack / heap memory model, then you'll understand the difference between String and &str and when to use which. Otherwise, Rust (in general) won't make sense to you.

## See also

- https://stackoverflow.com/questions/43079077/proper-way-to-return-a-new-string-in-rust
