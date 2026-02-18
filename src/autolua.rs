use proc_macro::TokenStream;
use anyhow::anyhow;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Fields, ItemStruct, Token, Type};
use syn::parse::{Parse, ParseStream};
use crate::{TokStream};

/*
  Thank you for reading my code <3
  I wish you have a good day, and
  May your beloved ones be with you forever
*/

///
/// *Imported from viator:viator-utils*
///
/// Deduce a enum into a specific variant.
/// Used when you are sure about that enum is the correct variant.
/// Will panic if variant mismatch.
///
/// Example:
/// ```
/// use viator_utils::deduce_enum;
///
/// let opt: Option<String> = Some("Some text".to_string());
///
/// let str: String = deduce_enum!(opt, Option::Some); // Works fine
/// let str: () = deduce_enum!(opt, Option::None); // this won't compile and will panic
/// ```
///
/// Currently, does not support deducing complex structs or tuples.
/// Only tuple1 is supported.
///
macro_rules! deduce_enum {
    ($var:expr, $enum_name:path) => {
        if let $enum_name(body) = $var {
            body
        } else {
            unreachable!()
        }
    };
}

macro_rules! field_has_attr {
    (PureTag, $field: ident, $text: literal) => {
        $field.attrs.iter().any(|x| {
            if let syn::Meta::Path(path) = &x.meta {
                return path.is_ident($text)
            }

            return false
        })
    };

    (WithParam, $field: ident, $text: literal) => {
        $field.attrs.iter().any(|x| {
            if let syn::Meta::List(list) = &x.meta {
                return list.path.is_ident($text)
            }

            return false
        })
    };
}

macro_rules! drop_attr {
    ($field: ident, $text: literal) => {
        $field.attrs = $field.attrs.clone().into_iter().filter(|attr| {
            match &attr.meta {
                syn::Meta::Path(path) => !path.is_ident($text),
                syn::Meta::List(list) => !list.path.is_ident($text),
                _ => true,
            }
        }).collect();
    };
}

#[allow(unused)]
pub struct MatrixLikeField {
    pub(crate) ident: Ident,
    pub(crate) set_ident: Option<Ident>,
    pub(crate) get_ident: Option<Ident>,

    ///
    /// Not supported for now. Preserved for future use.
    ///
    pub(crate) new_ident: Option<Ident>
}

#[allow(unused)]
pub struct MatrixTagArg {
    ident: Ident,
    eq_tok: Token![=],
    expr: Ident
}

impl Parse for MatrixTagArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let eq_tok = input.parse()?;
        let expr = input.parse()?;

        Ok(Self { ident, eq_tok, expr })
    }
}

pub struct MatrixTag {
    args: Vec<MatrixTagArg>,
}

impl Parse for MatrixTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut params: Vec<MatrixTagArg> = Vec::new();

        while !input.is_empty() {
            let param = input.parse::<MatrixTagArg>();

            if let Ok(param) = param {
                params.push(param);
            }

            if let Err(_) = input.parse::<Token![,]>() {
                break;
            }
        }

        return Ok(Self {
            args: params
        })
    }
}

#[allow(unused)]
pub struct StructInfo {
    pub(crate) stt: ItemStruct,
    pub(crate) target_fields: Vec<Ident>,
    pub(crate) skipped_fields: Vec<Ident>,
    pub(crate) mat_like_fields: Vec<MatrixLikeField>
}

pub struct AutoLuaArgs {
    params: Vec<String>
}

impl Parse for AutoLuaArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut params = Vec::new();

        while !input.is_empty() {
            let param = input.parse::<Ident>();
            if let Ok(param) = param {
                params.push(param.to_string());
            }

            if let Err(_) = input.parse::<Token![,]>() {
                break;
            }
        }

        Ok(Self { params })
    }
}

pub fn transform_struct(mut target: ItemStruct) -> anyhow::Result<StructInfo> {
    let input_fields = if let Fields::Named(named) = &mut target.fields {
        named
    } else {
        return Err(anyhow!("Only full struct is supported!"))
    };

    let mut regular_fields: Vec<Ident> = Vec::new();
    let mut skipped_fields: Vec<Ident> = Vec::new();
    let mut mat_like_fields: Vec<MatrixLikeField> = Vec::new();

    for field in &mut input_fields.named {
        if field_has_attr!(PureTag, field, "skip") {

            // Add MaybeValue wrapper to original type
            field.ty = Type::Verbatim(gen_maybe_wrapper(field.ty.clone().into_token_stream().into()));

            // Drop processed attribute (which is undefined to rustc)
            drop_attr!(field, "skip");
            drop_attr!(field, "hidden_彩蛋哦");

            // Random quote from quote.lama.icu
            //
            // English (Translated by Gemini 3 Fast):
            //      This unfamiliar place is but a fragment of the scenery,
            //      a fleeting station in the soul's long pilgrimage;
            //      the road that lies ahead remains, as it ever was, an interrogation of the infinite.
            //
            // for whoever wish to include this in your prog:
            // #[此一处陌生的地方_不过是心魂之旅中的一处景观_一次际遇_未来的路途一样还是无限之问]
            drop_attr!(field, "此一处陌生的地方_不过是心魂之旅中的一处景观_一次际遇_未来的路途一样还是无限之问");

            skipped_fields.push(field.ident.clone().unwrap());
        } else if field_has_attr!(WithParam, field, "matrix") {
            let attr = field.attrs.iter().find(|x| {
                if let syn::Meta::List(list) = &x.meta {
                    return list.path.is_ident("matrix");
                }
                return false
            }).unwrap();

            let matrix_attr = deduce_enum!(&attr.meta, syn::Meta::List);
            let args = syn::parse::<MatrixTag>(matrix_attr.tokens.clone().into())?;

            let mut get_ident = Option::None;
            let mut set_ident = Option::None;
            let mut new_ident = Option::None;

            args.args.iter().for_each(|it| {
                match it.ident.to_string().as_str() {
                    "get" => get_ident = Some(it.expr.clone()),
                    "set" => set_ident = Some(it.expr.clone()),
                    "new" => new_ident = Some(it.expr.clone()),

                    _ => { }
                }
            });

            mat_like_fields.push(MatrixLikeField {
                ident: field.ident.clone().unwrap(),
                set_ident,
                get_ident,
                new_ident
            })
        } else {
            regular_fields.push(field.ident.clone().unwrap());
        }
    }

    return Ok(StructInfo {
        stt: target,
        target_fields: regular_fields,
        skipped_fields,
        mat_like_fields,
    })
}

pub fn do_autolua(args: AutoLuaArgs, input: TokenStream) -> syn::Result<TokStream> {
    // We do nothing if nothing to auto impl
    if args.params.is_empty() {
        return Ok(input.into());
    }

    let mut into = false;
    let mut from = false;
    let mut ref_into = false;

    for param in &args.params {
        match param.as_str() {
            "Into" => into = true,
            "From" => from = true,
            "RefInto" => ref_into = true,

            _ => {}
        }
    }

    let input = syn::parse::<ItemStruct>(input)?;
    let info = transform_struct(input).unwrap();
    let mut tok_stream: TokStream = recreate_struct(&info).unwrap();

    if from {
        let from_toks = gen_from_lua(&info).unwrap();

        tok_stream = quote! {
            #tok_stream
            #from_toks
        }
    }

    if into {
        let into_toks = gen_into_lua(&info).unwrap();

        tok_stream = quote! {
            #tok_stream
            #into_toks
        }
    }

    if ref_into {
        let ref_to_toks = gen_into_lua_ref(&info).unwrap();

        tok_stream = quote! {
            #tok_stream
            #ref_to_toks
        }
    }

    return Ok(tok_stream);
}

fn gen_maybe_wrapper(ttype: TokenStream) -> TokStream {
    let ttype: TokStream = ttype.into();

    return quote! { viator_utils::maybe_value::MaybeValue<#ttype> };
}

fn recreate_struct(target: &StructInfo) -> anyhow::Result<TokStream> {
    let stt = &target.stt;

    return Ok(quote! {
        #[allow(non_snake_case)]
        #stt
    })
}

fn gen_from_lua(target: &StructInfo) -> anyhow::Result<TokStream> {
    let name = target.stt.ident.clone();

    let regular_fields = target.target_fields.iter().map(|it| {
        quote! {
            #it: table.get(stringify!(#it))?,
        }
    }).collect::<TokStream>();

    let skipped_fields = target.skipped_fields.iter().map(|it| {
        quote! {
            #it: viator_utils::maybe!(null),
        }
    }).collect::<TokStream>();

    let implementation: TokStream = quote! {
        #[allow(non_snake_case)]
        impl mlua::FromLua for #name {
            fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
                return match value {
                    mlua::Value::Table(table) => {
                        Ok(
                            Self {
                                #regular_fields
                                #skipped_fields
                            }
                        )
                    }

                    _ => {
                        Err(anyhow::anyhow!("Unable to convert such value into {} struct", stringify!(#name)).into())
                    }
                }
            }
        }
    }.into();

    Ok(quote! {
        #implementation
    })
}

fn gen_into_lua(target: &StructInfo) -> anyhow::Result<TokStream> {
    let name = target.stt.ident.clone();

    let combined_fields = target.target_fields.iter().map(|it| {
        quote! {
            tbl.set(stringify!(#it), self.#it)?;
        }
    }).collect::<TokStream>();

    return Ok(quote! {
        #[allow(non_snake_case)]
        impl mlua::IntoLua for #name {
            fn into_lua(self, lua: &mlua::Lua) -> Result<mlua::Value, mlua::Error> {
                let tbl = lua.create_table()?;

                #combined_fields

                return Ok(mlua::Value::Table(tbl));
            }
        }
    })
}

fn gen_into_lua_ref(target: &StructInfo) -> anyhow::Result<TokStream> {
    let name = target.stt.ident.clone();

    let combined_fields = target.target_fields.iter().map(|it| {
        quote! {
            tbl.set(stringify!(#it), self.#it.clone())?;
        }
    }).collect::<TokStream>();

    return Ok(quote! {
        #[allow(non_snake_case)]
        impl mlua::IntoLua for &#name {
            fn into_lua(self, lua: &mlua::Lua) -> Result<mlua::Value, mlua::Error> {
                let tbl = lua.create_table()?;

                #combined_fields

                return Ok(mlua::Value::Table(tbl));
            }
        }
    })
}