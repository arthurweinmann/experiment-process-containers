
// macro with #[macro_export] are exported at the root of the crate
#[macro_export]
macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Debug, Clone, PartialEq)] // ewww
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}