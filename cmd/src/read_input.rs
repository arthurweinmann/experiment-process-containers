use std::io;

// we can return variable created inside the function in a box, this is to say as a pointer
// to an allocated region in heap memory. Here it is not necessary but in somecase it may
// avoid a copy when returning a value (since we cannot return references to variable created inside the function
// except &'static str).

// String are a tuples of (str, capacity, len), no need for a box: https://users.rust-lang.org/t/function-returning-a-string-does-it-copy-the-value/10108
// line here will be moved not copied, but we could have written the function with Box as:
/*
pub fn read_input_line(ini_cap: usize) -> Result< Box<String>, String>{
    let mut line = Box::new(String::with_capacity(ini_cap));
    match io::stdin().read_line(&mut line) {
        Ok(_) => Ok(line),
        Err(e) => Err(format!("{}", e)),
    }
}
*/

// See also: https://www.reddit.com/r/rust/comments/7megb9/returning_a_string_from_a_function_string_or_str/
pub fn read_input_line(ini_cap: usize) -> Result<String, String>{
    let mut line = String::with_capacity(ini_cap);
    match io::stdin().read_line(&mut line) {
        Ok(_) => Ok(line),
        Err(e) => Err(format!("{}", e)),
    }
}