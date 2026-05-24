# speckle-macro

The `speckle` macro supports three modes:

## Function

```rust
/// Returns true when `bar` is 3
/// 
/// ```
/// assert_eq!(foo(2), false)
/// assert_eq!(foo(3), true)
/// assert_eq!(foo(4), false)
/// ```
#[speckle]
fn foo(bar: u8) -> bool { todo!() }
```

This triggers Speckle to implement the function asynchronously. When it is ready, the macro replaces the function body with the implementation.

## Trait

```rust
/// Gives the ability to say the struct's name
#[speckle]
trait SayName {
    /// Returns the name of the struct
    fn say_name(&self) -> &'static str;
}

#[speckle_derive(SayName)]
struct Foo;
```

This expands to

```rust
impl SayName for Foo {
    #[speckle]
    /// Returns the name of the struct
    fn say_name(&self) -> &'static str { todo!() }
}
```

## Interfaces

```rust
/// HTTP verbs such as GET
#[speckle]
enum HttpMethod {}

/// Makes web requests
#[speckle]
trait Http {}

/// Interface for making web requests
#[speckle]
#[speckle_derive(Http)]
struct HttpClient;
```

Speckle populates fields, variants, and methods.

## Modules

```rust
#[speckle]
mod http {
    pub enum Method {}
    pub struct Client;
    impl Client {
        fn fetch(&self) -> String { todo!() }
    }
}
```

Speckle does a rest-of-the-owl on the module contents.