extern crate proc_macro;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use utils::{
    get_variant_str_values_by_name, impl_display, impl_display_with_vals,
    string_manipulation::pascal_to_snake_case,
};

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
    let variant_idents = enum_item
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();
    let variants_ids = get_variant_str_values_by_name(enum_item.clone(), "emoji_id");

    let display_quote = impl_display_with_vals(
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

#[proc_macro_derive(Asset, attributes(src_path))]
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
    let variants_values = get_variant_str_values_by_name(enum_item.clone(), "src_path");

    impl_display_with_vals(enum_name, variant_idents, variants_values, |_ident, src_path| {
        format!(
            "https://raw.githubusercontent.com/1Git2Clone/serenity-discord-bot/main/src/assets/{src_path}",
        )
    })
    .into()
}

/// Implements `std::fmt::Display` for the enum by converting all the `PascalCase` variants to
/// `snake_case`.
///
/// NOTE: Also adds a `.as_str()` method.
#[proc_macro_derive(DatabaseEnum)]
pub fn derive_database_enum(input: TokenStream) -> TokenStream {
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

    impl_display(enum_name, variant_idents, |ident| {
        pascal_to_snake_case(&ident.to_string())
    })
    .into()
}
