extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

// Credit:
// https://stackoverflow.com/questions/68025264/how-to-get-all-the-variants-of-an-enum-in-a-vect-with-a-proc-macro/69812881#69812881
#[proc_macro_derive(IterateVariants)]
pub fn derive_all_variants(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let syn::Data::Enum(enum_item) = ast.data else {
        return quote!(compile_error!("IterateVariants only works on enums")).into();
    };

    let enum_name = ast.ident;
    let variant_idents = enum_item.variants.into_iter().map(|v| v.ident);

    quote! {
        impl #enum_name {
            pub fn variants() -> &'static[Self] {
                &[ #(#enum_name::#variant_idents),* ]
            }
            pub fn iter_variants() -> impl Iterator<Item = &'static Self> {
                #enum_name::variants().iter()
            }
        }
    }
    .into()
}

#[proc_macro_derive(DiscordEmoji, attributes(emoji_id))]
pub fn derive_discord_emoji(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let syn::Data::Enum(enum_item) = ast.data else {
        return quote!(compile_error!("DiscordEmoji only works on enums")).into();
    };

    let enum_name = ast.ident;
    let variant_idents = enum_item.variants.iter().map(|v| &v.ident);
    let variants_ids = enum_item
        .variants
        .iter()
        .filter_map(|v| {
            if !v.attrs.iter().any(|attr| attr.path().is_ident("emoji_id")) {
                return None;
            }

            v.attrs
                .iter()
                .find(|attr| attr.path().is_ident("emoji_id"))
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
        .map(|x| x.unwrap());

    let display_quote = {
        let variant_idents = variant_idents.clone();
        let variants_ids = variants_ids.clone();
        quote! {
            impl std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(Self::#variant_idents => {
                            write!(
                                f,
                                concat!(
                                    "<:", stringify!(#variant_idents), ":", #variants_ids, ">"
                                )
                            )
                        })*
                    }
                }
            }
        }
    };

    let id_quote = {
        let variant_idents = variant_idents.clone();
        let variants_ids = variants_ids.clone();
        quote! {
            impl #enum_name {
                pub fn get_id(&self) -> &'static str {
                    match self {
                        #(#enum_name::#variant_idents => #variants_ids,)*
                    }
                }
            }
        }
    };

    let variant_str_quote = {
        let variant_idents = variant_idents.clone();
        quote! {
            impl #enum_name {
                pub fn get_variant_str(&self) -> &'static str {
                    match self {
                        #(#enum_name::#variant_idents => stringify!(#variant_idents),)*
                    }
                }
            }
        }
    };

    quote! {
        #display_quote
        #id_quote
        #variant_str_quote
    }
    .into()
}
