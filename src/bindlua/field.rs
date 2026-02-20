use proc_macro2::Ident;
use quote::quote;
use syn::{Token, Type};
use syn::parse::{Parse, ParseStream};
use crate::bindlua::Qualifier;
use crate::TokStream;

#[allow(unused)]
pub struct FieldDefinition {
    pub qualifiers: Vec<Qualifier>,
    pub name: Ident,
    pub colon: Token![:],
    pub typ: Type,
    pub comma: Option<Token![,]>,
}

impl FieldDefinition {
    pub fn gen_toks(&self) -> TokStream {
        let name = self.name.clone();
        let typ = self.typ.clone();
        return quote! {
            #name: #typ,
        }
    }
}

impl FieldDefinition {
    pub fn is_lua(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Lua(_)));
    }

    pub fn is_ref(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Reference(_)));
    }

    pub fn is_mut(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Mutable(_)));
    }
}

impl Parse for FieldDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let qualifiers = input.call(Qualifier::parse_all)?;
        let name = input.parse()?;
        let colon = input.parse()?;
        let typ = input.parse()?;
        let comma = input.parse()?;

        return Ok(FieldDefinition {
            qualifiers,
            name,
            colon,
            typ,
            comma,
        })
    }
}