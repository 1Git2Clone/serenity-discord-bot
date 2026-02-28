extern crate proc_macro;
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::Error;
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
    // need to use &Vec<T> because `into_iter` takes ownership of the vector (when we can get away with just
    // references) and [`quote::quote_token_with_context`] binds the variable names with into_iter,
    // which is expanded from the fourth arm of the [`quote`] macro:
    //
    // https://docs.rs/quote/latest/src/quote/lib.rs.html#894
    let variants_names = &derive_enum
        .data
        .variants
        .iter()
        .map(|v| &v.ident)
        .collect::<Vec<_>>();

    let get_id_and_variant = {
        quote! {
            impl #name {
                pub fn get_id(&self) -> &'static str {
                    match self {
                        #(#name::#variants_names => #variants_ids,)*
                    }
                }
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
        #get_id_and_variant
    }
    .into()
}

#[proc_macro_derive(Asset, attributes(base_url, src_path))]
pub fn derive_asset(input: TokenStream) -> TokenStream {
    let derive_enum = syn::parse_macro_input!(input as utils::data::DeriveEnum);

    let Some(base_url_attr) = derive_enum
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("base_url"))
    else {
        return Error::new_spanned(
            &derive_enum.ident,
            "Set a `base_url(\"<link>\")` attribute on the top of the Enum.",
        )
        .to_compile_error()
        .into();
    };

    let base_url: String = match &base_url_attr.meta {
        // base_url = "<link>"
        syn::Meta::NameValue(nv) => {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(litstr),
                ..
            }) = &nv.value
            {
                litstr.value()
            } else {
                return syn::Error::new_spanned(&nv.value, "Expected string literal for base_url.")
                    .to_compile_error()
                    .into();
            }
        }

        // base_url("<link>")
        syn::Meta::List(list) => {
            let parsed: syn::LitStr = syn::parse2(list.tokens.clone()).unwrap_or_else(|_| {
                panic!("Expected exactly one string literal inside base_url(...).")
            });
            parsed.value()
        }

        _ => {
            return syn::Error::new_spanned(
                &derive_enum.ident,
                "Invalid `base_url` attribute syntax.",
            )
            .to_compile_error()
            .into();
        }
    };

    let target_variant_name = "src_path";

    let variants_values = derive_enum
        .data
        .get_variant_str_values_by_name(target_variant_name)
        .iter()
        .map(|src_path| format!("{}/{}", base_url, src_path))
        .collect();

    impl_display_with_vals(&derive_enum, variants_values, |_ident, src_path| {
        String::from(src_path)
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
