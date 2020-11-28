use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;
use syn::GenericArgument;
use syn::Ident;
use syn::PathArguments;
use syn::{DeriveInput, Result, Type};

use crate::ast::{Input, Struct};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(node)?;
    input.validate()?;
    Ok(match input {
        Input::Struct(input) => input.impl_builder()?,
        _ => unreachable!(),
    })
}

impl<'a> Struct<'a> {
    pub fn impl_builder(self) -> Result<TokenStream> {
        let ident = &self.ident;

        let builder_ident = self.builder_ident();

        let builder_fields = self.builder_fields();

        let methods = self.methods();

        let builder_fields_empty = self.builder_fields_empty();

        let build_fields = self.build_fields();

        let each_methods = self.each_methods()?;

        let expanded = quote! {
            // create the builder struct with name builder_ident and optionized fields from the
            // original struct
            pub struct #builder_ident {
                #(#builder_fields,)*
            }

            // implement methods on to the builder struct that will set fields appropriately
            impl #builder_ident {
                // the regular methods that set individual fields
                #(#methods)*

                #(#each_methods)*

                // extend methods
                pub fn build(&self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
                    std::result::Result::Ok(#ident {
                        #(#build_fields,)*
                    })
                }
            }

            // implementations on the original struct (currently only the builder method)
            impl #ident {
                fn builder() -> #builder_ident {
                    #builder_ident {
                        #(#builder_fields_empty,)*
                    }
                }
            }
        };

        Ok(expanded.into())
    }

    fn each_methods(&'a self) -> Result<impl Iterator<Item = TokenStream> + 'a> {
        Ok(self
            .fields
            .iter()
            .filter(|f| f.attrs.each.is_some())
            .map(|f| {
                let field_ident = &f.ident;
                let ty = &f.ty;
                let inner_ty = inner_ty(ty, "Vec").ok_or_else(|| {
                    Error::new_spanned(ty, "The each key must only be use with a Vec")
                })?;

                let fn_ident: Ident = {
                    let fn_lit = f
                        .attrs
                        .each
                        .as_ref()
                        .expect("Must be some, the nones were filtered out above");
                    fn_lit.parse()?
                };

                let expanded = quote! {
                    pub fn #fn_ident(&mut self, #fn_ident: #inner_ty) -> &mut Self {
                        self.#field_ident.push(#fn_ident);
                        self
                    }
                };

                Ok(expanded)
            })
            .collect::<Result<Vec<TokenStream>>>()?
            .into_iter())
    }

    /// The fields for the `build` function
    fn build_fields(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields.iter().map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;
            if inner_ty(ty, "Option").is_some() || f.attrs.each.is_some() {
                quote! {
                    #ident: self.#ident.clone()
                }
            } else {
                quote! {
                    #ident: self.#ident.clone().ok_or(concat!(stringify!(#ident), " is not set"))?
                }
            }
        })
    }

    fn builder_ident(&self) -> Ident {
        let ident = &self.ident;
        let builder_name = format!("{}Builder", &self.ident);
        Ident::new(&builder_name, ident.span())
    }

    fn builder_fields(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields.iter().map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;
            if inner_ty(ty, "Option").is_some() || f.attrs.each.is_some() {
                quote! { #ident: #ty }
            } else {
                quote! { #ident: std::option::Option<#ty> }
            }
        })
    }

    fn methods(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields
            .iter()
            .filter(|f| f.attrs.each.is_none())
            .map(|f| {
                let ident = &f.ident;
                let ty = &f.ty;

                // if the field is optional
                if let Some(inner_ty) = inner_ty(ty, "Option") {
                    quote! {
                        pub fn #ident(&mut self, #ident: #inner_ty) -> &mut Self {
                            self.#ident = Some(#ident);
                            self
                        }
                    }
                } else {
                    quote! {
                        pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                            self.#ident = Some(#ident);
                            self
                        }
                    }
                }
            })
    }

    fn builder_fields_empty(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.fields.iter().map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;

            if inner_ty(ty, "Option").is_some() {
                quote! { #ident: None }
            } else if inner_ty(ty, "Vec").is_some() && f.attrs.each.is_some() {
                quote! { #ident: std::vec::Vec::new() }
            } else {
                quote! { #ident: None }
            }
        })
    }
}

/// Checks if the the type has the ident of the wrapper. If it does, give the inner type of the
/// wrapper.
fn inner_ty<'a>(ty: &'a Type, wrapper: &str) -> Option<&'a Type> {
    let last = match ty {
        Type::Path(type_path) => type_path.path.segments.last()?,
        _ => return None,
    };

    if last.ident != wrapper {
        return None;
    }

    let type_arg = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => {
            let args = &bracketed.args;

            if args.len() != 1 {
                return None;
            }

            args.last()
                .expect("Must be okay, len of args was checked above")
        }
        _ => return None,
    };

    match type_arg {
        GenericArgument::Type(type_arg) => Some(type_arg),
        _ => None,
    }
}

// /// Checks if the type is an option. If it is, will return Some(ty) or None
// fn ty_is_option(ty: &Type) -> bool {
//     let path = match ty {
//         Type::Path(ty) => &ty.path,
//         _ => return false,
//     };

//     let last = path.segments.last().unwrap();
//     if last.ident != "Option" {
//         return false;
//     }

//     match &last.arguments {
//         PathArguments::AngleBracketed(bracketed) => bracketed.args.len() == 1,
//         _ => false,
//     }
// }
