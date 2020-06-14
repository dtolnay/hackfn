//! [![github]](https://github.com/dtolnay/hackfn)&ensp;[![crates-io]](https://crates.io/crates/hackfn)&ensp;[![docs-rs]](https://docs.rs/hackfn)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! <br>
//!
//! Fake implementation of `std::ops::Fn` for user-defined data types.
//!
//! Place a `#[hackfn]` attribute on an impl block containing a single method to
//! use that method as the implementation of the function call operator.
//!
//! # Limitations
//!
//! - The function must receive `&self`. Functions that receive `&mut self` or
//!   `self` are not supported.
//!
//! - The function may not have generic parameters or where-clause.
//!
//! - The `Self` type must implement `Sized`.
//!
//! # Examples
//!
//! ```
//! use hackfn::hackfn;
//!
//! /// Function object that adds some number to its input.
//! struct Plus(u32);
//!
//! #[hackfn]
//! impl Plus {
//!     fn call(&self, other: u32) -> u32 {
//!         self.0 + other
//!     }
//! }
//!
//! fn main() {
//!     let plus_one = Plus(1);
//!     let sum = plus_one(2);
//!     assert_eq!(sum, 3);
//! }
//! ```
//!
//! The next example is somewhat more elaborate:
//!
//! - Interior mutability can be used to approximate a `FnMut` impl.
//!
//! - Generic parameters and where-clause are permitted on the impl block
//!   (though not on the function).
//!
//! - The function may take any number of arguments.
//!
//! ```
//! use hackfn::hackfn;
//!
//! use std::cell::Cell;
//! use std::ops::Add;
//!
//! /// Function object that accumulates a pair of values per call.
//! #[derive(Default)]
//! struct AccumulatePairs<T> {
//!     first: Cell<T>,
//!     second: Cell<T>,
//! }
//!
//! #[hackfn]
//! impl<T> AccumulatePairs<T> where T: Copy + Add<Output = T> {
//!     fn call(&self, first: T, second: T) {
//!         self.first.set(self.first.get() + first);
//!         self.second.set(self.second.get() + second);
//!     }
//! }
//!
//! fn main() {
//!     let accumulate = AccumulatePairs::default();
//!     accumulate(30, 1);
//!     accumulate(20, 2);
//!     accumulate(10, 3);
//!     assert_eq!(accumulate.first.get(), 60);
//!     assert_eq!(accumulate.second.get(), 6);
//! }
//! ```

#![cfg_attr(docs_rs_workaround, feature(proc_macro))]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    braced, parenthesized, parse_macro_input, Attribute, Generics, Ident, Token, Type, Visibility,
};

struct Nothing;

impl Parse for Nothing {
    fn parse(_input: ParseStream) -> Result<Self> {
        Ok(Nothing)
    }
}

struct FnArg {
    ident: Ident,
    ty: Type,
}

impl Parse for FnArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(FnArg { ident, ty })
    }
}

struct HackFn {
    impl_attrs: Vec<Attribute>,
    generics: Generics,
    self_ty: Type,
    fn_attrs: Vec<Attribute>,
    vis: Visibility,
    method: Ident,
    args: Vec<FnArg>,
    ret_ty: Option<Type>,
    body: TokenStream2,
}

impl Parse for HackFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let impl_attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![impl]>()?;
        let mut generics: Generics = input.parse()?;
        let self_ty: Type = input.parse()?;
        generics.where_clause = input.parse()?;

        let impl_block;
        braced!(impl_block in input);

        let fn_attrs = impl_block.call(Attribute::parse_outer)?;
        let vis: Visibility = impl_block.parse()?;
        impl_block.parse::<Token![fn]>()?;
        let method: Ident = impl_block.parse()?;

        let argument_list;
        parenthesized!(argument_list in impl_block);
        argument_list.parse::<Token![&]>()?;
        argument_list.parse::<Token![self]>()?;

        let mut args = Vec::new();
        while !argument_list.is_empty() {
            argument_list.parse::<Token![,]>()?;
            if argument_list.is_empty() {
                break;
            }
            args.push(argument_list.parse::<FnArg>()?);
        }

        let ret_ty = if impl_block.parse::<Option<Token![->]>>()?.is_some() {
            Some(impl_block.parse::<Type>()?)
        } else {
            None
        };

        let body;
        braced!(body in impl_block);
        let body: TokenStream2 = body.parse()?;

        Ok(HackFn {
            impl_attrs,
            generics,
            self_ty,
            fn_attrs,
            vis,
            method,
            args,
            ret_ty,
            body,
        })
    }
}

#[proc_macro_attribute]
pub fn hackfn(args: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(args as Nothing);

    let HackFn {
        impl_attrs,
        generics,
        self_ty,
        fn_attrs,
        vis,
        method,
        args,
        ret_ty,
        body,
    } = parse_macro_input!(input as HackFn);

    let impl_attrs = &impl_attrs;
    let where_clause = &generics.where_clause;
    let arg_names = args.iter().map(|fn_arg| &fn_arg.ident).collect::<Vec<_>>();
    let arg_types = args.iter().map(|fn_arg| &fn_arg.ty).collect::<Vec<_>>();
    let ret_ty = ret_ty.map(|ret| quote!(-> #ret));

    let target = quote! {
        ::std::ops::Fn(#(#arg_types),*) #ret_ty
    };

    let expanded = quote! {
        #(#impl_attrs)*
        impl #generics #self_ty #where_clause {
            #(#fn_attrs)*
            #vis fn #method(&self #(, #arg_names: #arg_types)*) #ret_ty {
                #body
            }
        }

        #(#impl_attrs)*
        impl #generics ::std::ops::Deref for #self_ty #where_clause {
            type Target = #target;

            // This implementation assumes that a closure that captures a type T
            // by move has the same layout as T.
            fn deref(&self) -> &Self::Target {
                let __this = ::std::mem::MaybeUninit::<Self>::uninit();
                let __closure = move |#(#arg_names : #arg_types),*| #ret_ty {
                    Self::#method(
                        unsafe { &*__this.as_ptr() }
                        #(, #arg_names)*
                    )
                };
                let __size_of_closure = ::std::mem::size_of_val(&__closure);
                fn __second<'__a, __T>(__first: &__T, __second: &'__a __T) -> &'__a __T {
                    __second
                }
                let __ret = __second(&__closure, unsafe { ::std::mem::transmute(self) });
                ::std::mem::forget(__closure);
                assert_eq!(__size_of_closure, ::std::mem::size_of::<Self>());
                unsafe { ::std::mem::transmute(__ret as &#target) }
            }
        }
    };

    TokenStream::from(expanded)
}
