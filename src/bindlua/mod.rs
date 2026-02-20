mod field;
mod function;
mod sig;

use proc_macro2::{Ident, Span};
use quote::{quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Attribute, Token};
use syn::__private::{CustomToken};
use syn::buffer::Cursor;
use syn::token::{Brace};
use paste::paste;
use crate::bindlua::field::FieldDefinition;
use crate::bindlua::function::FunctionDefinition;
use crate::imported::keyword;
use crate::TokStream;

/*
  Thank you for reading my code <3
  I wish you have a good day, and
  May your beloved ones be with you forever
*/

macro_rules! define_custom_keywords {
    ($($keyword:ident)*) => {
        $(
            paste! {
                #[allow(unused)]
                struct [<Keyword $keyword:camel>] {
                    span: Span
                }

                impl Parse for [<Keyword $keyword:camel>] {
                    fn parse(input: ParseStream) -> syn::Result<Self> {
                        Ok(Self {
                            span: keyword(input, stringify!($keyword))?,
                        })
                    }
                }

                impl CustomToken for [<Keyword $keyword:camel>] {
                    fn peek(cursor: Cursor) -> bool {
                        let data = cursor.ident();

                            if data.is_none() {
                            return false;
                        }

                        return data.unwrap().0.to_string() == stringify!($keyword);
                    }

                    fn display() -> &'static str {
                        return stringify!($keyword);
                    }
                }
            }
        )*

        paste! {
            #[allow(unused)]
            enum Qualifier {
                $(
                    [<$keyword:camel>]([<Keyword $keyword:camel>]),
                )*
                Reference(Token![ref]),
                Mutable(Token![mut]),
                Public(Token![pub]),
                Static(Token![static]),
            }

            impl Parse for Qualifier {
                fn parse(input: ParseStream) -> syn::Result<Self> {
                    if input.peek(Token![ref]) {
                        return Ok(Self::Reference(input.parse()?))
                    } else if input.peek(Token![mut]) {
                        return Ok(Self::Mutable(input.parse()?))
                    } else if input.peek(Token![pub]) {
                        return Ok(Self::Public(input.parse()?))
                    } else if input.peek(Token![static]) {
                        return Ok(Self::Static(input.parse()?))
                    } $(
                    else if [<Keyword $keyword:camel>]::peek(input.cursor()) {
                        return Ok(Self::[<$keyword:camel>](input.parse()?))
                    }
                    )*

                    return Err(syn::Error::new(Span::call_site(), "unrecognized token"))
                }
            }

            impl Qualifier {
                fn peek(input: &ParseStream) -> bool {
                    return input.peek(Token![ref]) ||
                    input.peek(Token![mut]) ||
                    input.peek(Token![pub]) ||
                    input.peek(Token![static])
                    $(|| [<Keyword $keyword:camel>]::peek(input.cursor()) )*
                }

                fn is_qualifier(ident: &Ident) -> bool {
                    let str = ident.to_string();
                    return  str == "ref" || str == "mut" || str == "pub" || str == "static" $(|| str == stringify!($keyword))*;
                }

                #[allow(unused)]
                fn to_string(&self) -> &'static str {
                    match self {
                        Qualifier::Reference(_) => { "ref" }
                        Qualifier::Mutable(_) => { "mut" }
                        Qualifier::Public(_) => { "pub" }
                        Qualifier::Static(_) => { "static" }
                        $(Qualifier::[<$keyword:camel>](_) => { stringify!($keyword) })*,
                    }
                }

                fn parse_all(input: ParseStream) -> syn::Result<Vec<Qualifier>> {
                    let mut qualifier = Vec::new();
                    while Self::peek(&input) {
                        let val = Self::parse(input);
                        if val.is_err() {
                            return Ok(qualifier);
                        }
                        qualifier.push(val?);
                    }
                    return Ok(qualifier);
                }

                fn gen_qualifier(qualifier: &Vec<Qualifier>) -> TokStream {
                    return qualifier.iter()
                    .filter(|it| {
                        matches!(it, Qualifier::Reference(_) | Qualifier::Mutable(_) | Qualifier::Public(_))
                    })
                    .map(|it| {
                        match it {
                            Qualifier::Mutable(q) => quote! { #q },
                            Qualifier::Public(q) => quote! { #q },
                            Qualifier::Reference(q) => quote! { #q },
                            Qualifier::Static(q) => quote! { #q },
                            _ => { unreachable!() }
                        }
                    })
                    .collect::<TokStream>()
                }
            }
        }


    };
}


define_custom_keywords!(
    lua /* Marks a field should be included in UserData */
    get /* get/set style of defining a field, applies only to functions */
    set /* get/set style of defining a field, applies only to functions */
    operator /* Marks a function to be an operator function */
);



#[allow(unused)]
pub struct BindLuaBlock {
    apple: Vec<Attribute>,
    qualifiers: Vec<Qualifier>,
    name: Ident,
    brace: Brace,

    fields: Vec<FieldDefinition>,
    functions: Vec<FunctionDefinition>,
}

impl BindLuaBlock {
    fn gen_struct_def(&self) -> TokStream {
        let name = &self.name;

        let fields = &self.fields.iter().map(|it| {
            it.gen_toks()
        }).collect::<TokStream>();

        let stt_attr_macro = &self.apple.iter().map(|it| {
            quote! { #it }
        }).collect::<TokStream>();
        
        let stt_qualifier = Qualifier::gen_qualifier(&self.qualifiers);

        return quote! {
            #[allow(non_snake_case)]
            #stt_attr_macro
            #stt_qualifier
            struct #name {
                #fields
            }
        }
    }

    fn gen_impl_def(&self) -> TokStream {
        let name = &self.name;

        let funcs = &self.functions.iter().map(|it| {
            it.gen_toks()
        }).collect::<TokStream>();

        return quote! {
            #[allow(non_snake_case)]
            impl #name {
                #funcs
            }
        }
    }

    fn gen_userdata(&self) -> TokStream {
        let stt_name = &self.name;

        let fns: TokStream = self.functions.iter()
            .filter(|it| it.is_lua())
            .filter(|it| !it.is_functional_field())
            .map(|it| {
                let name = &it.sig.ident;

                if it.is_static() {
                    if it.is_operator() {
                        quote! {
                            methods.add_meta_function(stringify!(#name), Self::#name);
                        }
                    } else {
                        quote! {
                            methods.add_function(stringify!(#name), Self::#name);
                        }
                    }
                } else {
                    if it.is_operator() {
                        quote! {
                            methods.add_meta_method(stringify!(#name), Self::#name);
                        }
                    } else {
                        quote! {
                            methods.add_method(stringify!(#name), Self::#name);
                        }
                    }
                }


            })
            .collect();

        let functional_fields: TokStream = self.functions.iter()
            .filter(|it| it.is_lua())
            .filter(|it| it.is_functional_field())
            .map(|it| {
                let fn_name = &it.sig.ident;
                let field_name = it.field_name.clone().unwrap();



                return if it.is_getter() {
                    quote! {
                        fields.add_field_method_get(stringify!(#field_name), Self::#fn_name);
                    }
                } else if it.is_setter() {
                    quote! {
                        fields.add_field_method_set(stringify!(#field_name), Self::#fn_name);
                    }
                } else {
                    unreachable!("Impossible to not have either get or set qualifier")
                }
            })
            .collect();


        // TODO: Support const (how?) Qualifier
        let fields: TokStream = self.fields.iter()
            .filter(|it| it.is_lua())
            .map(|it| {
                let name = &it.name;

                let mut toks = if it.is_ref() {
                    quote! {
                        fields.add_field_method_get(stringify!(#name), |lua, this| {
                            return this.#name.into_lua(lua);
                        });
                    }
                } else {
                    quote! {
                        fields.add_field_method_get(stringify!(#name), |lua, this| {
                            return this.#name.clone().into_lua(lua);
                        });
                    }
                };

                if it.is_mut() {
                    toks = quote! {
                        #toks

                        fields.add_field_method_set(stringify!(#name), |lua, this, val| {
                            this.#name = val;

                            return Ok(())
                        });
                    };
                }

                return toks;
            })
            .collect();

        return quote! {
            #[allow(non_snake_case)]
            impl mlua::UserData for #stt_name {
                fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
                    #fields

                    #functional_fields
                }

                fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
                    #fns
                }
            }
        }
    }

    fn gen_toks(&self) -> TokStream {
        let stt = self.gen_struct_def();
        let impl_def = self.gen_impl_def();
        let userdata = self.gen_userdata();

        return quote! {
            #stt

            #impl_def

            #userdata
        }
    }
}


impl Parse for BindLuaBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let maybe_macro = input.call(Attribute::parse_outer)?;
        let qualifiers = input.call(Qualifier::parse_all)?;

        let name = input.parse()?;

        let body;
        let brace = braced!(body in input);
        let mut fields: Vec<FieldDefinition> = vec!();
        let mut functions: Vec<FunctionDefinition> = vec!();

        while !body.is_empty() {
            if FunctionDefinition::peek_fn(body.cursor()) {
                functions.push(body.parse::<FunctionDefinition>()?);
            } else {
                fields.push(body.parse::<FieldDefinition>()?);
            }
        }

        return Ok(Self {
            apple: maybe_macro,
            qualifiers,
            name,
            brace,
            fields,
            functions,
        })

    }
}

pub struct BindLua {
    blocks: Vec<BindLuaBlock>,
}

///
/// Syntax for bindlua is:
///
/// bindlua! {
///     lua $name {
///         $([lua] $field:FieldDefinition [,])*
///         $([lua] $functionDefinition)*
///     }
/// }
///
/// Generate into:
///
/// struct $name {
///     $field* |> DropOptionalKeywordPrefix(lua)
/// }
///
/// impl $name {
///     $functionDefinition* |> DropOptionalKeywordPrefix(lua)
/// }
///
/// impl UserData for $name {
///     ...
/// }
///
impl Parse for BindLua {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut blocks: Vec<BindLuaBlock> = vec!();

        while !input.is_empty() {
            blocks.push(input.parse::<BindLuaBlock>()?);
        }

        return Ok(Self { blocks })
    }
}

pub fn do_bindlua(bindlua: BindLua) -> anyhow::Result<TokStream> {
    let generated: &TokStream = &bindlua.blocks.iter().map(BindLuaBlock::gen_toks).collect();

    return Ok(generated.clone());
}