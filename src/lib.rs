//! Define multiple default implementations for a trait.
//!
//! This library contains two attribute macros: `default_trait_impl` which defines a default trait
//! implementation, and `trait_impl` which uses a default trait implementation you've defined.
//!
//! This is particularly useful in testing, when many of your mocked types will have very similar
//! trait implementations, but do not want the canonical default trait implementation to use mocked
//! values.
//!
//! # Example
//!
//! First, define a default trait implementation for the trait `Car`:
//!
//! ```
//! #[default_trait_impl]
//! impl Car for NewCar {
//!     fn get_mileage(&self) -> Option<usize> { Some(6000) }
//!     fn has_bluetooth(&self) -> bool { true }
//! }
//! ```
//!
//! `NewCar` does not need to be defined beforehand.
//!
//! Next, implement the new default implementation for a type:
//!
//! ```
//! struct NewOldFashionedCar;
//!
//! #[trait_impl]
//! impl NewCar for NewOldFashionedCar {
//!     fn has_bluetooth(&self) -> bool { false }
//! }
//!
//!
//! struct WellUsedNewCar;
//!
//! #[trait_impl]
//! impl NewCar for WellUsedNewCar {
//!     fn get_mileage(&self) -> Option<usize> { Some(100000) }
//! }
//! ```
//!
//! This will ensure that our structs use the `NewCar` defaults, without having to change the
//! canonical `Car` default implementation:
//!
//! ```
//! fn main() {
//!     assert_eq!(NewOldFashionedCar.get_mileage(), Some(6000));
//!     assert_eq!(NewOldFashionedCar.has_bluetooth(), false);
//!     assert_eq!(WellUsedNewCar.get_mileage(), Some(100000));
//!     assert_eq!(WellUsedNewCar.has_bluetooth(), true);
//! }
//! ```

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use syn::{parse_macro_input, parse_str, Ident, ImplItem, ImplItemMethod, ItemImpl, Type};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref DEFAULT_TRAIT_IMPLS: Mutex<HashMap<String, DefaultTraitImpl>> =
        Mutex::new(HashMap::new());
}

struct DefaultTraitImpl {
    pub trait_name: String,
    pub items: Vec<String>,
}

#[proc_macro_attribute]
pub fn default_trait_impl(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);

    let pseudotrait = match *input.self_ty {
        Type::Path(type_path) => match type_path.path.get_ident() {
            Some(ident) => ident.to_string(),
            None => return syntax_invalid_error(),
        },
        _ => return syntax_invalid_error(),
    };

    let trait_name = match input.trait_ {
        Some(trait_tuple) => match trait_tuple.1.get_ident() {
            Some(ident) => ident.to_string(),
            None => return syntax_invalid_error(),
        },
        _ => return syntax_invalid_error(),
    };

    let items: Vec<String> = input
        .items
        .iter()
        .map(|item| {
            return quote! {
                #item
            }
            .to_string();
        })
        .collect();

    DEFAULT_TRAIT_IMPLS
        .lock()
        .unwrap()
        .insert(pseudotrait, DefaultTraitImpl { trait_name, items });

    TokenStream::new()
}

fn syntax_invalid_error() -> TokenStream {
    return quote! {
        compile_error!("`default_trait_impl` expects to be given a syntactially valid trait implementation");
    }.into();
}

#[proc_macro_attribute]
pub fn trait_impl(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemImpl);

    let trait_name = match &input.trait_ {
        Some(trait_tuple) => match trait_tuple.1.get_ident() {
            Some(ident) => ident.to_string(),
            None => return syntax_invalid_error(),
        },
        _ => return syntax_invalid_error(),
    };

    let mut idents = HashSet::new();
    for item in &input.items {
        match item {
            ImplItem::Method(method) => {
                idents.insert(method.sig.ident.to_string());
            }
            ImplItem::Const(constant) => {
                idents.insert(constant.ident.to_string());
            }
            ImplItem::Type(ty) => {
                idents.insert(ty.ident.to_string());
            }
            _ => (),
        };
    }

    match DEFAULT_TRAIT_IMPLS.lock().unwrap().get(&trait_name) {
        Some(default_impl) => {
            if let Some(trait_tuple) = &mut input.trait_ {
                trait_tuple.1.segments[0].ident = Ident::new(&default_impl.trait_name, Span::call_site());
            }

            for default_impl_item in &default_impl.items {
                let parsed_result: ImplItem = parse_str(default_impl_item).unwrap();
                match parsed_result{
                    ImplItem::Method(method) if !idents.contains(&method.sig.ident.to_string()) =>{
                        input.items.push(ImplItem::Method(method));
                    }
                    ImplItem::Const(constant) if !idents.contains(&constant.ident.to_string()) =>{
                        input.items.push(ImplItem::Const(constant));
                    }
                    ImplItem::Type(ty) if !idents.contains(&ty.ident.to_string()) =>{
                        input.items.push(ImplItem::Type(ty));
                    }
                    ImplItem::Macro(_) => {return quote! {
                        compile_error!("macros invocation within default impl blocks are not supported");
                    }.into();
                    }
                    _ => ()
                }
            }
        },
        _ => return quote! {
            compile_error!("`trait_impl` expects there to be a `default_trait_impl` for the trait it implements");
        }.into()
    }

    let res = quote! {
        #input
    };
    res.into()
}
