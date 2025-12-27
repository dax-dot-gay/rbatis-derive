use manyhow::manyhow;
use proc_macro2::TokenStream;

mod derive_schema;

#[manyhow]
#[proc_macro_derive(Schema, attributes(schema, field))]
pub fn macro_derive_schema(input: syn::DeriveInput) -> manyhow::Result<TokenStream> {
    derive_schema::derive_schema(input)
}