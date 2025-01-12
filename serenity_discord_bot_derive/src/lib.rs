extern crate proc_macro;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use utils::{
    data::StrValuesByName, impl_display, impl_display_with_vals,
    string_manipulation::pascal_to_snake_case,
};

// Credit:
// https://stackoverflow.com/questions/68025264/how-to-get-all-the-variants-of-an-enum-in-a-vect-with-a-proc-macro/69812881#69812881
#[proc_macro_derive(IterateVariants)]
pub fn derive_all_variants(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as utils::data::DeriveEnum);

    let enum_name = ast.ident;
    let variant_idents = ast.data.variants.into_iter().map(|v| v.ident);

    quote! {
        impl #enum_name {
            pub fn variants() -> &'static[Self] {
                &[ #(#enum_name::#variant_idents),* ]
            }
        }
    }
    .into()
}

#[proc_macro_derive(DiscordEmoji, attributes(emoji_id))]
pub fn derive_discord_emoji(input: TokenStream) -> TokenStream {
    let derive_enum = syn::parse_macro_input!(input as utils::data::DeriveEnum);

    let target_variant_id = "emoji_id";
    let variants_ids = derive_enum
        .data
        .get_variant_str_values_by_name(target_variant_id);

    let display_impl = impl_display_with_vals(&derive_enum, variants_ids.clone(), |ident, id| {
        format!("<:{ident}:{id}>")
    });

    let name = derive_enum.ident;
    let variants_names = derive_enum
        .data
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();

    let get_id = {
        let variants_names = variants_names.clone();
        quote! {
            impl #name {
                pub fn get_id(&self) -> &'static str {
                    match self {
                        #(#name::#variants_names => #variants_ids,)*
                    }
                }
            }
        }
    };
    let get_variant_str = {
        let variants_names = variants_names.clone();
        quote! {
            impl #name {
                pub fn get_variant_str(&self) -> &'static str {
                    match self {
                        #(#name::#variants_names => {
                            stringify!(#variants_names)
                        },)*
                    }
                }
            }
        }
    };

    quote! {
        #display_impl
        #get_id
        #get_variant_str
    }
    .into()
}

#[proc_macro_derive(Asset, attributes(src_path))]
pub fn derive_asset(input: TokenStream) -> TokenStream {
    let derive_enum = syn::parse_macro_input!(input as utils::data::DeriveEnum);

    let target_variant_name = "src_path";
    let variants_values = derive_enum
        .data
        .get_variant_str_values_by_name(target_variant_name);

    impl_display_with_vals(&derive_enum, variants_values, |_ident, src_path| {
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
    let derive_enum = syn::parse_macro_input!(input as utils::data::DeriveEnum);

    impl_display(&derive_enum, |ident| {
        pascal_to_snake_case(&ident.to_string())
    })
    .into()
}
