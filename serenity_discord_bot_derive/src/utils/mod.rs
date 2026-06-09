pub mod data;
pub mod string_manipulation;

use quote::quote;

pub fn quote_display_impl(
    enum_name: syn::Ident,
    variant_idents: Vec<&syn::Ident>,
    display_results: &[String],
) -> proc_macro2::TokenStream {
    let display = {
        let iter = display_results.iter();
        quote! {
            impl std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(Self::#variant_idents => {
                            write!(
                                f,
                                #iter
                            )
                        })*
                    }
                }
            }
        }
    };
    let as_str = {
        let iter = display_results.iter();
        quote! {
            impl #enum_name {
                pub fn as_str(&self) -> &'static str {
                    match self {
                        #(Self::#variant_idents => {
                                #iter
                        })*
                    }
                }
            }
        }
    };

    quote! {
        #display
        #as_str
    }
}

pub fn impl_display<'a>(
    derive_enum: &'a crate::utils::data::DeriveEnum,
    display_pat: fn(ident: &'a syn::Ident) -> String,
) -> proc_macro2::TokenStream {
    let enum_name = derive_enum.ident.clone();
    let variant_idents = derive_enum
        .data
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();

    let res = variant_idents
        .iter()
        .map(|i| display_pat(i))
        .collect::<Vec<_>>();

    quote_display_impl(enum_name, variant_idents, &res)
}

pub fn impl_display_with_vals<'a>(
    derive_enum: &'a crate::utils::data::DeriveEnum,
    variants_values: Vec<String>,
    display_pat: fn(ident: &'a syn::Ident, val: &str) -> String,
) -> proc_macro2::TokenStream {
    let enum_name = derive_enum.ident.clone();
    let variant_idents = derive_enum
        .data
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();
    let res = variant_idents
        .iter()
        .zip(variants_values.iter())
        .map(|(i, v)| display_pat(i, v))
        .collect::<Vec<_>>();

    quote_display_impl(enum_name, variant_idents, &res)
}
