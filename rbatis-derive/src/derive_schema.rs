use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromField, FromMeta, ast::Data};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Arm, DeriveInput, Expr, Ident, Path, Token, Type, punctuated::Punctuated};

#[derive(FromField, Clone, Debug)]
#[darling(attributes(field))]
struct FieldReciever {
    pub ident: Option<Ident>,
    pub ty: Type,

    #[darling(default)]
    pub unique: bool,

    #[darling(default)]
    pub not_null: bool,

    #[darling(default)]
    pub select: bool,

    #[darling(default)]
    pub sql_type: Option<String>,
}

#[derive(FromMeta, Clone, Debug, Default)]
struct TableDefinition {
    #[darling(default)]
    pub name: Option<String>,
}

#[derive(FromDeriveInput, Clone, Debug)]
#[darling(attributes(schema), supports(struct_named))]
struct DeriveSchemaInput {
    pub ident: Ident,

    #[darling(default)]
    pub table: TableDefinition,

    #[darling(default)]
    pub rbatis: Option<Path>,

    #[darling(default)]
    pub rbs: Option<Path>,
    pub data: Data<(), FieldReciever>,
}

fn process_field(field: FieldReciever, rbs: Path) -> manyhow::Result<(Arm, Arm)> {
    let FieldReciever {
        ident,
        ty,
        unique,
        not_null,
        sql_type,
        ..
    } = field;
    let ident = ident.unwrap();
    let ident_str = ident.to_string().to_case(Case::Snake);
    let sql_type: Expr = if let Some(sqlt) = sql_type {
        syn::parse2(quote! {Some(String::from(#sqlt))})?
    } else {
        syn::parse2(quote! {None})?
    };

    let field_type_arm = syn::parse2::<Arm>(quote! {
        #ident_str => {
            let type_override: Option<String> = #sql_type;
            let default_value: #ty = Default::default();
            Some(type_override.unwrap_or(mapper.get_column_type(#ident_str, &#rbs::value!(default_value))))
        }
    })?;

    let mut constraints = Vec::<String>::new();
    if unique {
        constraints.push("UNIQUE".to_string());
    }
    if not_null {
        constraints.push("NOT NULL".to_string());
    }

    let constraint_str = constraints.join(" ");

    let constraint_arm = syn::parse2::<Arm>(quote! {
        #ident_str => {
            Some(format!("{} {}", Self::field_type(field.as_str(), mapper).unwrap(), #constraint_str).trim().to_string())
        }
    })?;

    Ok((field_type_arm, constraint_arm))
}

pub fn derive_schema(input: DeriveInput) -> manyhow::Result<TokenStream> {
    let DeriveSchemaInput {
        ident: schema_ident,
        table,
        rbatis,
        rbs,
        data: schema_data,
    } = DeriveSchemaInput::from_derive_input(&input).or_else(|e| Err(syn::Error::from(e)))?;
    let rbatis = rbatis.unwrap_or(Path::from(Ident::new("rbatis", Span::call_site())));
    let rbs = rbs.unwrap_or(Path::from(Ident::new("rbs", Span::call_site())));
    let table_name = table
        .name
        .clone()
        .unwrap_or(schema_ident.to_string().to_case(Case::Snake));

    let fields = if let Data::Struct(fields) = schema_data {
        fields
    } else {
        unreachable!("Enums not supported!")
    };

    if !fields.is_struct() {
        unreachable!("Should only support struct_named!");
    }

    let field_keys = fields
        .clone()
        .into_iter()
        .map(|v| v.ident.unwrap().to_string().to_case(Case::Snake))
        .collect::<Punctuated<String, Token![,]>>();

    let mut field_type_arms = Punctuated::<Arm, Token![,]>::new();
    let mut constraint_arms = Punctuated::<Arm, Token![,]>::new();
    for field in fields.clone() {
        let (ft, c) = process_field(field, rbs.clone())?;
        field_type_arms.push(ft);
        constraint_arms.push(c);
    }

    Ok(quote! {
        #rbatis::crud!(#schema_ident{}, #table_name);

        impl #schema_ident {
            pub fn fields() -> Vec<String> {
                vec![#field_keys].into_iter().map(|v| v.to_string()).collect()
            }

            pub fn field_type(field: impl Into<String>, mapper: &dyn #rbatis::table_sync::ColumnMapper) -> Option<String> {
                let field = field.into();
                if !Self::fields().contains(&field) {
                    return None;
                }

                match field.as_str() {
                    #field_type_arms,
                    _ => None
                }
            }

            pub fn field_constraints(field: impl Into<String>, mapper: &dyn #rbatis::table_sync::ColumnMapper) -> Option<String> {
                let field = field.into();
                if !Self::fields().contains(&field) {
                    return None;
                }

                match field.as_str() {
                    #constraint_arms,
                    _ => None
                }
            }

            pub async fn sync(rb: &#rbatis::rbatis::RBatis, mapper: &dyn #rbatis::table_sync::ColumnMapper) -> #rbatis::error::Result<()> {
                let mut columns: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                for field in Self::fields() {
                    let _ = columns.insert(field.clone(), Self::field_constraints(field.clone(), mapper).unwrap());
                }

                let map = #rbs::value!(columns);
                #rbatis::rbatis::RBatis::sync(&rb.acquire().await.unwrap(), mapper, &map, #table_name).await?;
                Ok(())
            }
        }
    })
}
