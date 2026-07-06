# proc_macro_argue

Declarative argument parsing for procedural macros in rust

```rust

argue!{
  //Enum argument
  MyArgument {
    Foo: syn::LitStr,
    Bar: syn::LitInt,
    Baz: BazArgument
  };
  //Tuple argument
  BazArgument(syn::LitStr, syn::token::Comma, syn::LitInt)
}


use MyArgument::*;
let args:ArgumentList<MyArgument> = syn::parse(token_stream)?;

let foo = argue!(args may have Foo)?;
let baz = argue!(args must have Bar)?;
let bar = argue!(args may repeat Baz)?;

#[my_macro(Foo("foo"), Bar(1), Baz("baz", 1), Baz("baz", 2))]
struct MyStruct;

```
