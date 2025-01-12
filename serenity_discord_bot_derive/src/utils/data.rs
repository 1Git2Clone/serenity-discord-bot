#[allow(dead_code)]
#[derive(Clone)]
pub struct DeriveEnum {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub data: syn::DataEnum,
}

impl syn::parse::Parse for DeriveEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ast = input.parse::<syn::DeriveInput>()?;

        let syn::Data::Enum(enum_item) = ast.data else {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Only works on enums!",
            ));
        };

        Ok(DeriveEnum {
            attrs: ast.attrs,
            vis: ast.vis,
            ident: ast.ident,
            generics: ast.generics,
            data: enum_item,
        })
    }
}

pub trait StrValuesByName {
    fn get_variant_str_values_by_name(&self, name: &str) -> Vec<String>;
}

impl StrValuesByName for syn::DataEnum {
    fn get_variant_str_values_by_name(&self, name: &str) -> Vec<String> {
        self.variants
            .iter()
            .flat_map(|v| {
                let attr = v.attrs.iter().find(|attr| attr.path().is_ident(name))?;

                let syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(str),
                            ..
                        }),
                    ..
                }) = &attr.meta
                else {
                    return None;
                };

                Some(str.value())
            })
            .collect()
    }
}
