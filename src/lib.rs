use std::ops::Deref;

//IDEA: default values for arguments

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
            1 => Ok(args.get(0).cloned()),
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
    ($name: ident {$($arg: ident$(: $ty:ty)*),*$(,)?}) => {


        #[allow(unused)]
        enum $name {
            $($arg(::syn::Ident, ::proc_macro_argue::argue_optional!($($ty,)* ::syn::Path))),*
        }

        // #[allow(unused)]
        // use $name::*;

        impl ::syn::parse::Parse for $name {
            fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                let meta: ::syn::Meta = input.parse()?;
                let path = match &meta {
                    ::syn::Meta::Path(path) => path,
                    ::syn::Meta::List(meta_list) => &meta_list.path,
                    ::syn::Meta::NameValue(meta_name_value) => &meta_name_value.path,
                };
                use $name::*;
                let ident: ::syn::Ident = path.require_ident()?.clone();

                match ident.to_string().as_str() {
                    $(
                        stringify!($arg) => ::proc_macro_argue::argue_parse!(meta $(as $ty)*).map(|a|$arg(ident, a)),
                    )*
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
    ($($name: ident $decl:tt);*$(;)?) => {$(
        argue!($name $decl);
    )*};

}

//for resolving path meta args
#[doc(hidden)]
#[macro_export]
macro_rules! argue_optional {
    ($ty1:path $(, $ty2:ty)?) => {
        $ty1
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! argue_parse {
    //generating the parsing for path arguments
    ($meta:ident) => {
        ::proc_macro_argue::Expect::<::syn::Path>::expect($meta)
    };

    //generating the parsing for list arguments
    ($meta:ident as $ty:ty) => {
        ::proc_macro_argue::Expect::<::syn::MetaList>::expect($meta)
            .map(|list| list.tokens)
            .and_then(::syn::parse2::<$ty>)
    };
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

pub trait Expect<T> {
    fn expect(self) -> Result<T, syn::Error>;
}

impl Expect<syn::MetaList> for syn::Meta {
    fn expect(self) -> Result<syn::MetaList, syn::Error> {
        match self {
            syn::Meta::List(meta_list) => Ok(meta_list),
            meta => Err(syn::Error::new_spanned(meta, "Expected a List argument")),
        }
    }
}

impl Expect<syn::Path> for syn::Meta {
    fn expect(self) -> Result<syn::Path, syn::Error> {
        match self {
            syn::Meta::Path(path) => Ok(path),
            meta => Err(syn::Error::new_spanned(meta, "Expected a Path argument")),
        }
    }
}
impl Expect<syn::MetaNameValue> for syn::Meta {
    fn expect(self) -> Result<syn::MetaNameValue, syn::Error> {
        match self {
            syn::Meta::NameValue(meta_name_value) => Ok(meta_name_value),
            meta => Err(syn::Error::new_spanned(
                meta,
                "Expected a Name Value argument",
            )),
        }
    }
}
