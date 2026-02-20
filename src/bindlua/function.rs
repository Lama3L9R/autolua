use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, Block, Signature};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use crate::bindlua::{sig, Qualifier};
use crate::TokStream;

pub struct FunctionDefinition {
    pub attrs: Vec<Attribute>,
    pub qualifiers: Vec<Qualifier>,
    pub sig: Signature,
    pub body: Block,
    pub field_name: Option<Ident>,
}

impl FunctionDefinition {
    pub fn gen_toks(&self) -> TokStream {
        let attrs = &self.attrs.iter().map(|it| {
            quote! { #it }
        }).collect::<TokStream>();

        let qualifiers = Qualifier::gen_qualifier(&self.qualifiers);

        let sig = &self.sig;

        let body = self.body.clone();

        return quote! {
            #attrs
            #qualifiers
            #sig
            #body
        }
    }

    pub fn is_lua(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Lua(_)));
    }

    pub fn is_functional_field(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Get(_) | Qualifier::Set(_)));
    }

    pub fn is_getter(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Get(_)));
    }

    pub fn is_setter(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Set(_)));
    }

    pub fn is_static(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Static(_)));
    }

    pub fn is_operator(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Operator(_)));
    }

    pub fn peek_fn(input: Cursor) -> bool {
        let mut opt = input.ident();

        while opt.is_some() {
            let (ident, cursor) = opt.unwrap();

            if Qualifier::is_qualifier(&ident) {
                opt = cursor.ident();

                continue;
            }

            return if ident.to_string() == "fn" {
                true
            } else {
                false
            }
        }

        return false;
    }
}

impl Parse for FunctionDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let mut result = FunctionDefinition {
            attrs: input.call(Attribute::parse_outer)?,
            qualifiers: input.call(Qualifier::parse_all)?,
            sig: input.parse()?,
            body: input.parse()?,
            field_name: None
        };

        <FunctionDefinition as sig::CompleteFnSignature>::complete_signature(&mut result);

        return Ok(result)
    }
}