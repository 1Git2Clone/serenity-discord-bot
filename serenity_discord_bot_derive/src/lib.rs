extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

// Credit:
// https://stackoverflow.com/questions/68025264/how-to-get-all-the-variants-of-an-enum-in-a-vect-with-a-proc-macro/69812881#69812881
#[proc_macro_derive(IterateVariants)]
pub fn derive_all_variants(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let syn::Data::Enum(enum_item) = ast.data else {
        return quote!(compile_error!("AllVariants only works on enums")).into();
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
