use proc_macro2::{Ident, Span};
use quote::{quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, Attribute, Block, Expr, FnArg, Pat, PatIdent, PatType, ReturnType, Signature, Stmt, Token, Type};
use syn::__private::{CustomToken};
use syn::buffer::Cursor;
use syn::token::{Brace};
use paste::paste;
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
            }

            impl Parse for Qualifier {
                fn parse(input: ParseStream) -> syn::Result<Self> {
                    if input.peek(Token![ref]) {
                        return Ok(Self::Reference(input.parse()?))
                    } else if input.peek(Token![mut]) {
                        return Ok(Self::Mutable(input.parse()?))
                    } else if input.peek(Token![pub]) {
                        return Ok(Self::Public(input.parse()?))
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
                    input.peek(Token![pub])
                    $(|| [<Keyword $keyword:camel>]::peek(input.cursor()) )*
                }

                fn is_qualifier(ident: &Ident) -> bool {
                    let str = ident.to_string();
                    return  str == "ref" || str == "mut" || str == "pub" $(|| str == stringify!($keyword))*;
                }

                #[allow(unused)]
                fn to_string(&self) -> &'static str {
                    match self {
                        Qualifier::Reference(_) => { "ref" }
                        Qualifier::Mutable(_) => { "mut" }
                        Qualifier::Public(_) => { "pub" }
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
                            _ => { unreachable!() }
                        }
                    })
                    .collect::<TokStream>()
                }
            }
        }


    };
}

macro_rules! simple_type {
    ($ty:ty) => {
        Box::new(Type::Verbatim(quote! { $ty }))
    };
}

macro_rules! simple_fn_arg {
    ($name:ident: $ty:ty) => {
        FnArg::Typed(PatType {
            attrs: Vec::new(),
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: Ident::new(stringify!($name), Span::call_site()),
                subpat: None,
            })),
            colon_token: Default::default(),
            ty: simple_type!($ty),
        })
    };
}

define_custom_keywords!(
    lua /* Marks a field should be included in UserData */
    get /* get/set style of defining a field, applies only to functions */
    set /* get/set style of defining a field, applies only to functions */
);

#[allow(unused)]
struct FieldDefinition {
    qualifiers: Vec<Qualifier>,
    name: Ident,
    colon: Token![:],
    typ: Type,
    comma: Option<Token![,]>,
}

impl FieldDefinition {
    fn gen_toks(&self) -> TokStream {
        let name = self.name.clone();
        let typ = self.typ.clone();
        return quote! {
            #name: #typ,
        }
    }
}

impl FieldDefinition {
    fn is_lua(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Lua(_)));
    }

    fn is_ref(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Reference(_)));
    }

    fn is_mut(&self) -> bool {
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

struct FunctionDefinition {
    pub attrs: Vec<Attribute>,
    pub qualifiers: Vec<Qualifier>,
    pub sig: Signature,
    pub body: Block,
    pub field_name: Option<Ident>,
}

impl FunctionDefinition {
    fn gen_toks(&self) -> TokStream {
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

    fn is_lua(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Lua(_)));
    }

    fn is_functional_field(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Get(_) | Qualifier::Set(_)));
    }

    fn is_getter(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Get(_)));
    }

    fn is_setter(&self) -> bool {
        return self.qualifiers.iter().any(|it| matches!(it, Qualifier::Set(_)));
    }

    fn peek_fn(input: Cursor) -> bool {
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
        let attrs = input.call(Attribute::parse_outer)?;
        let qualifiers = input.call(Qualifier::parse_all)?;
        let mut sig: Signature = input.parse()?;
        let mut body: Block = input.parse()?;
        let mut field_name = None;

        let mut auto_completed_lua = false;
        if qualifiers.iter().any(|it| matches!(it, Qualifier::Lua(_))) {
            if sig.inputs.len() == 0 {
                sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
                auto_completed_lua = true;
            }
        }


        if qualifiers.iter().any(|it| matches!(it, Qualifier::Get(_))) {
            /* Complete function signature for getter */

            if matches!(sig.output, ReturnType::Default) {
                let fn_name = &sig.ident;
                body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                    compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Expected a return type to auto complete required generics on add_field_method_get", stringify!(#fn_name)));
                }), Default::default()))
            }

            if sig.inputs.len() == 1 {
                sig.inputs.push(simple_fn_arg!(this: &Self));
            }

            field_name = Some(sig.ident.clone());
            sig.ident = Ident::new(&*format!("__lua_get_{}", sig.ident), Span::call_site());
        } else if qualifiers.iter().any(|it| matches!(it, Qualifier::Set(_))) {
            /* Complete function signature for setter */

            if sig.inputs.len() == 1 && !auto_completed_lua {
                let ty = sig.inputs.pop().unwrap();

                if let FnArg::Typed(_) = ty.value() {
                    sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
                    sig.inputs.push(simple_fn_arg!(this: &mut Self));
                    sig.inputs.push(ty.value().clone())
                }
            }

            if sig.inputs.len() == 2 {
                /* Default behavior for setter is that the first arg is default to the value being set */
                let ty = sig.inputs.pop().unwrap();

                if let FnArg::Typed(_) = ty.value() {
                    sig.inputs.push(simple_fn_arg!(this: &mut Self));
                    sig.inputs.push(ty.value().clone())
                }

            }

            if matches!(sig.output, ReturnType::Default) {
                sig.output = ReturnType::Type(
                    Default::default(),
                    simple_type!(mlua::Result<()>)
                );
            }

            field_name = Some(sig.ident.clone());
            sig.ident = Ident::new(&*format!("__lua_set_{}", sig.ident), Span::call_site());
        } else {
            /* Complete function signature for regular lua function */

            if sig.inputs.len() == 1 {
                sig.inputs.push(simple_fn_arg!(this: &Self));
            }

            if sig.inputs.len() == 2 {
                sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));
            }

            if matches!(sig.output, ReturnType::Default) {
                sig.output = ReturnType::Type(
                    Default::default(),
                    simple_type!(mlua::Result<()>)
                );

                body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! { return Ok(()) }), Default::default()))
            }
        }

        return Ok(FunctionDefinition {
            attrs,
            qualifiers,
            sig,
            body,
            field_name
        })
    }
}

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

                quote! {
                    methods.add_method(stringify!(#name), Self::#name);
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