use std::ops::Deref;

#[macro_export]
macro_rules! argue {
    ($name: ident may have $ty:path) => {
        $name.iter().find_map(|arg| match arg {
            $ty(ident, val) => Some((ident, val)),
            _ => None,
        })
    };
    ($name: ident must have $ty:path) => {
        argue!($name may have $ty).ok_or_else(|| {
                ::syn::Error::new(
                    ::proc_macro::Span::call_site().into(),
                    format!("Missing Required argument {}", stringify!($ty)),
                )
            })
    };
    ($($name:ident: {$($arg: ident: $ty:ty),*})*) => ($(
        #[allow(unused)]
        enum $name {
            $($arg(syn::Ident, $ty)),*
        }
        #[allow(unused)]
        use $name::*;

        impl syn::parse::Parse for $name {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let meta: syn::MetaList = input.parse()?;
                let ident: syn::Ident = meta.path.require_ident()?.clone();
                use $name::*;
                match ident.to_string().as_str() {
                    $(stringify!($arg) => syn::parse2(meta.tokens).map(|r| $arg(ident, r)),)*
                    _ => Err(syn::Error::new_spanned(ident, "Invalid Argument")),
                }
            }
        }
    )*);
}
pub struct List<T: syn::parse::Parse>(syn::punctuated::Punctuated<T, syn::token::Comma>);
impl<T: syn::parse::Parse> syn::parse::Parse for List<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input
            .parse_terminated(T::parse, syn::Token![,])
            .map(|args| List(args))
    }
}
impl<T: syn::parse::Parse> Deref for List<T> {
    type Target = syn::punctuated::Punctuated<T, syn::token::Comma>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
