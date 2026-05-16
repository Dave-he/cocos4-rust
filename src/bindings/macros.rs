#[macro_export]
macro_rules! wasm_export {
    ($name:ident) => {
        pub fn $name() {}
    };
}

#[macro_export]
macro_rules! native_export {
    ($(#[$attr:meta])* $vis:vis fn $name:ident($($arg:ident: $type:ty),*) -> $ret:ty $body:block) => {
        $vis fn $name($($arg: $type),*) -> $ret $body
    };
}
