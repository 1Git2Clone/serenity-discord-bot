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

fn impl_display<'a>(
    enum_name: syn::Ident,
    variant_idents: Vec<&'a syn::Ident>,
    variants_values: Vec<String>,
    display_pat: fn(ident: &'a syn::Ident, val: &str) -> String,
) -> proc_macro2::TokenStream {
    let display_iter = variant_idents
        .iter()
        .zip(variants_values.iter())
        .map(|(i, v)| display_pat(i, v));

    quote! {
        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#variant_idents => {
                        write!(
                            f,
                            #display_iter
                        )
                    })*
                }
            }
        }
    }
}

fn get_variant_str_values_by_name(enum_item: syn::DataEnum, name: &str) -> Vec<String> {
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

#[proc_macro_derive(DiscordEmoji, attributes(emoji_id))]
pub fn derive_discord_emoji(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let syn::Data::Enum(enum_item) = ast.data else {
        return quote!(compile_error!("DiscordEmoji only works on enums")).into();
    };

    let enum_name = ast.ident;
    let variant_idents = enum_item
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();
    let variants_ids = get_variant_str_values_by_name(enum_item.clone(), "emoji_id");

    let display_quote = impl_display(
        enum_name.clone(),
        variant_idents.clone(),
        variants_ids.clone(),
        |ident, id| format!("<:{ident}:{id}>",),
    );

    let id_quote = {
        let variant_idents = variant_idents.clone().into_iter();
        let variants_ids = variants_ids.clone().into_iter();
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
        let variant_idents = variant_idents.clone().into_iter();
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

// TODO: Simplify the process of making these single attribute derive macros due to the current
// code duplication

#[proc_macro_derive(Asset, attributes(filename))]
pub fn derive_asset(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let syn::Data::Enum(enum_item) = ast.data else {
        return quote!(compile_error!("Only works on enums")).into();
    };

    let enum_name = ast.ident;
    let variant_idents = enum_item
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();
    let variants_values = get_variant_str_values_by_name(enum_item.clone(), "filename");

    impl_display(enum_name, variant_idents, variants_values, |_ident, filename| {
        format!(
            "https://raw.githubusercontent.com/1Git2Clone/serenity-discord-bot/main/src/assets/{filename}",
        )
    })
    .into()
}
