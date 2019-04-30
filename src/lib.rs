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
#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parenthesized, parse_macro_input, Attribute, Generics, Ident, Token, Type, Visibility};

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
        input.parse::<Token![impl ]>()?;
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
    let ret_ty = &ret_ty;
    let where_clause = &generics.where_clause;
    let arg_names = args.iter().map(|fn_arg| &fn_arg.ident);
    let arg_types = args.iter().map(|fn_arg| &fn_arg.ty);
    let arg_names2 = arg_names.clone();
    let arg_names3 = arg_names.clone();
    let arg_types2 = arg_types.clone();
    let arg_types3 = arg_types.clone();

    let target = quote! {
        ::std::ops::Fn(#(#arg_types2),*) #(-> #ret_ty)*
    };

    let expanded = quote! {
        #(#impl_attrs)*
        impl #generics #self_ty #where_clause {
            #(#fn_attrs)*
            #vis fn #method(&self #(, #arg_names: #arg_types)*) #(-> #ret_ty)* {
                #body
            }
        }

        #(#impl_attrs)*
        impl #generics ::std::ops::Deref for #self_ty #where_clause {
            type Target = #target;

            // This implementation assumes that a closure that captures a type T
            // by move has the same layout as T.
            fn deref(&self) -> &Self::Target {
                let __this: Self = unsafe { ::std::mem::uninitialized() };
                let __closure = move |#(#arg_names2 : #arg_types3),*| #(-> #ret_ty)* {
                    Self::#method(&__this #(, #arg_names3)*)
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
