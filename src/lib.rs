use std::ops::Deref;

#[macro_export]
macro_rules! argue {
    ($name: ident may repeat $ty:path) => {
        $name.iter().filter_map(|arg| match arg {
            $ty(ident, val) => Some((ident, val)),
            _ => None,
        })

    };

    ($name: ident may have $ty:path) => {{
        let args: Vec<_> = argue!($name may repeat $ty).collect();
        match args.len() {
            0 => Ok(None),
            1 => Ok(args.get(1).cloned()),
            _ => {
                let mut errors = args.iter().skip(1).map(|(ident, ..)|
                    ::syn::Error::new_spanned(ident, concat!(stringify!($ty), " may only appear once"))
                );
                let mut error = errors.nth(0).expect("match gurantees at least 2");
                error.extend(errors);
                Err(error)
            }
        }
    }};

    ($name: ident must have $ty:path) => {
        argue!($name may have $ty).and_then(|arg|arg.ok_or_else(|| {
            ::syn::Error::new(
                ::proc_macro::Span::call_site().into(),
                format!("Missing Required argument {}", stringify!($ty)),
            )
        }))
    };

    //generate enum for nested argument
    ($name: ident {$($arg: ident: $ty:ty),*$(,)?}) => {
        #[allow(unused)]
        enum $name {
            $($arg(::syn::Ident, $ty)),*
        }

        // #[allow(unused)]
        // use $name::*;

        impl ::syn::parse::Parse for $name {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                let meta: ::syn::MetaList = input.parse()?;
                let ident: ::syn::Ident = meta.path.require_ident()?.clone();
                use $name::*;
                match ident.to_string().as_str() {
                    $(stringify!($arg) => ::syn::parse2(meta.tokens).map(|r| $arg(ident, r)),)*
                    _ => Err(syn::Error::new_spanned(ident, "Invalid Argument")),
                }
            }
        }
    };

    //generate struct for argument parameters
    ($name: ident ($($ty:ty),*$(,)?)) => {
        struct $name($($ty),*);
        impl ::syn::parse::Parse for $name {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                Ok($name($(input.parse::<$ty>()?),*))
            }
        }
    };

    //allow nested and direct declarations in one invokation
    ($($name: ident $decl:tt)*) => {$(
        argue!($name $decl);
    )*};
}

pub struct ArgumentList<A, D = syn::token::Comma>(syn::punctuated::Punctuated<A, D>)
where
    A: syn::parse::Parse,
    D: syn::parse::Parse;

impl<A, D> syn::parse::Parse for ArgumentList<A, D>
where
    A: syn::parse::Parse,
    D: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        syn::punctuated::Punctuated::parse_terminated_with(input, A::parse).map(|p| ArgumentList(p))
    }
}
impl<A, D> Deref for ArgumentList<A, D>
where
    A: syn::parse::Parse,
    D: syn::parse::Parse,
{
    type Target = syn::punctuated::Punctuated<A, D>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, A, D> IntoIterator for &'a ArgumentList<A, D>
where
    A: syn::parse::Parse,
    D: syn::parse::Parse,
{
    type Item = &'a A;
    type IntoIter = syn::punctuated::Iter<'a, A>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
