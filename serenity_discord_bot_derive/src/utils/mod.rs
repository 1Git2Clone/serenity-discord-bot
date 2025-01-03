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
    enum_name: syn::Ident,
    variant_idents: Vec<&'a syn::Ident>,
    display_pat: fn(ident: &'a syn::Ident) -> String,
) -> proc_macro2::TokenStream {
    let res = variant_idents
        .iter()
        .map(|i| display_pat(i))
        .collect::<Vec<_>>();

    quote_display_impl(enum_name, variant_idents, &res)
}

pub fn impl_display_with_vals<'a>(
    enum_name: syn::Ident,
    variant_idents: Vec<&'a syn::Ident>,
    variants_values: Vec<String>,
    display_pat: fn(ident: &'a syn::Ident, val: &str) -> String,
) -> proc_macro2::TokenStream {
    let res = variant_idents
        .iter()
        .zip(variants_values.iter())
        .map(|(i, v)| display_pat(i, v))
        .collect::<Vec<_>>();

    quote_display_impl(enum_name, variant_idents, &res)
}

pub fn get_variant_str_values_by_name(enum_item: syn::DataEnum, name: &str) -> Vec<String> {
    enum_item
        .variants
        .iter()
        .filter_map(|v| {
            if !v.attrs.iter().any(|attr| attr.path().is_ident(name)) {
                return None;
            }

            v.attrs
                .iter()
                .find(|attr| attr.path().is_ident(name))
                .map(|attr| match &attr.meta {
                    syn::Meta::NameValue(nv) => match &nv.value {
                        syn::Expr::Lit(lit_expr) => match &lit_expr.lit {
                            syn::Lit::Str(str) => Some(str.value()),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                })
        })
        .map(|x| x.unwrap())
        .collect()
}
