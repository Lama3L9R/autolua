use std::collections::HashMap;
use std::sync::LazyLock;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Expr, FnArg, ReturnType, Stmt, Pat};
use crate::bindlua::function::FunctionDefinition;

macro_rules! simple_type {
    ($ty:ty) => {
        Box::new(syn::Type::Verbatim(quote! { $ty }))
    };
}

macro_rules! simple_fn_arg {
    ($name:ident: $ty:ty) => {
        FnArg::Typed(syn::PatType {
            attrs: Vec::new(),
            pat: Box::new(syn::Pat::Ident(syn::PatIdent {
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

static META_FUNCTIONS: LazyLock<HashMap<String, (String, i8)>> = LazyLock::new(|| {
    // https://gist.github.com/oatmealine/655c9e64599d0f0dd47687c1186de99f
    HashMap::from([
        // Calculation operators
        // Name                LuaName           Params
        ("add".to_owned(),    ("__add".to_owned(),  2)),
        ("sub".to_owned(),    ("__sub".to_owned(),  2)),
        ("mul".to_owned(),    ("__mul".to_owned(),  2)),
        ("div".to_owned(),    ("__div".to_owned(),  2)),
        ("negate".to_owned(), ("__unm".to_owned(),  1)),
        ("mod".to_owned(),    ("__mod".to_owned(),  2)),
        ("pow".to_owned(),    ("__pow".to_owned(),  2)),
        ("idiv".to_owned(),   ("__idiv".to_owned(), 2)),

        // Bitwise operators
        //                         Name               LuaName            Params
        #[cfg(feature = "lua53")] ("and".to_owned(), ("__band".to_owned(), 2)),
        #[cfg(feature = "lua53")] ("or".to_owned(),  ("__bor".to_owned(),  2)),
        #[cfg(feature = "lua53")] ("xor".to_owned(), ("__bxor".to_owned(), 2)),
        #[cfg(feature = "lua53")] ("not".to_owned(), ("__bnot".to_owned(), 1)),
        #[cfg(feature = "lua53")] ("shl".to_owned(), ("__shl".to_owned(),  2)),
        #[cfg(feature = "lua53")] ("shr".to_owned(), ("__shr".to_owned(),  2)),

        // Equation operators
        // Name            LuaName           Params
        ("eq".to_owned(), ("__eq".to_owned(), 2)),
        ("lt".to_owned(), ("__lt".to_owned(), 2)),
        ("le".to_owned(), ("__le".to_owned(), 2)),

        // Misc operators
        ("concat".to_owned(), ("__concat".to_owned(), 2)),
        ("len".to_owned(),    ("__len".to_owned(),    1)),

        // Indexing
        ("get".to_owned(), ("__index".to_owned(),    2)),
        ("set".to_owned(), ("__newindex".to_owned(), 3)),

        // Function Call
        ("invoke".to_owned(), ("__call".to_owned(), -1)),

        // GC
        ("onGarbageCollect".to_owned(), ("__gc".to_owned(), 1)),

        // Misc
        ("toString".to_owned(), ("__tostring".to_owned(), 1)),
        ("pairs".to_owned(),    ("__pairs".to_owned(),    1)),
        ("ipairs".to_owned(),   ("__ipairs".to_owned(),   1)),

    ])
});

static META_METHODS: LazyLock<HashMap<String, (String, i8)>> = LazyLock::new(|| {
    // https://gist.github.com/oatmealine/655c9e64599d0f0dd47687c1186de99f
    HashMap::from([

        // Calculation operators
        // Name                LuaName           Params
        ("add".to_owned(),    ("__add".to_owned(),  1)),
        ("sub".to_owned(),    ("__sub".to_owned(),  1)),
        ("mul".to_owned(),    ("__mul".to_owned(),  1)),
        ("div".to_owned(),    ("__div".to_owned(),  1)),
        ("negate".to_owned(), ("__unm".to_owned(),  0)),
        ("mod".to_owned(),    ("__mod".to_owned(),  1)),
        ("pow".to_owned(),    ("__pow".to_owned(),  1)),
        ("idiv".to_owned(),   ("__idiv".to_owned(), 1)),

        // Bitwise operators
        //                         Name               LuaName            Params
        #[cfg(feature = "lua53")] ("and".to_owned(), ("__band".to_owned(), 1)),
        #[cfg(feature = "lua53")] ("or".to_owned(),  ("__bor".to_owned(),  1)),
        #[cfg(feature = "lua53")] ("xor".to_owned(), ("__bxor".to_owned(), 1)),
        #[cfg(feature = "lua53")] ("not".to_owned(), ("__bnot".to_owned(), 0)),
        #[cfg(feature = "lua53")] ("shl".to_owned(), ("__shl".to_owned(),  1)),
        #[cfg(feature = "lua53")] ("shr".to_owned(), ("__shr".to_owned(),  1)),

        // Equation operators
        // Name            LuaName           Params
        ("eq".to_owned(), ("__eq".to_owned(), 1)),
        ("lt".to_owned(), ("__lt".to_owned(), 1)),
        ("le".to_owned(), ("__le".to_owned(), 1)),

        // Misc operators
        // Name                LuaName               Params
        ("concat".to_owned(), ("__concat".to_owned(), 1)),
        ("len".to_owned(),    ("__len".to_owned(),    0)),

        // Indexing
        // Name              LuaName               Params
        ("get".to_owned(), ("__index".to_owned(),    1)),
        ("set".to_owned(), ("__newindex".to_owned(), 2)),

        // Function Call
        ("invoke".to_owned(), ("__call".to_owned(), -1)),

        // GC
        ("onGarbageCollect".to_owned(), ("__gc".to_owned(), 0)),

        // Misc
        ("toString".to_owned(), ("__tostring".to_owned(), 0)),
        ("pairs".to_owned(),    ("__pairs".to_owned(),    0)),
        ("ipairs".to_owned(),   ("__ipairs".to_owned(),   0)),

    ])
});

///
/// Used to complete definitions
///
pub trait CompleteFnSignature {
    fn complete_signature_getter(target: &mut FunctionDefinition);
    fn complete_signature_setter(target: &mut FunctionDefinition);
    fn complete_signature_static(target: &mut FunctionDefinition);
    fn complete_signature_lua(target: &mut FunctionDefinition);
    fn complete_signature_operator(target: &mut FunctionDefinition);
    fn complete_signature_rust(_: &mut FunctionDefinition) { /* We do not complete for non-lua functions */ }

    fn complete_signature(target: &mut FunctionDefinition) {
        if !target.is_lua() {
            return Self::complete_signature_rust(target);
        }

        if target.is_setter() {
            target.field_name = Some(target.sig.ident.clone());

            Self::complete_signature_setter(target);
        } else if target.is_getter() {
            target.field_name = Some(target.sig.ident.clone());

            Self::complete_signature_getter(target);
        } else if target.is_operator() { /* Operator has higher priority than static since we can `lua static operator fn` */
            Self::complete_signature_operator(target);
        } else if target.is_static() {
            Self::complete_signature_static(target);
        } else {
            Self::complete_signature_lua(target);
        }
    }
}

#[cfg(not(feature = "simple"))]
impl CompleteFnSignature for FunctionDefinition {
    fn complete_signature_getter(target: &mut FunctionDefinition) {
        if matches!(target.sig.output, ReturnType::Default) {
            let fn_name = &target.sig.ident;
            target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                    compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Expected a return type to auto complete required generics on add_field_method_get"));
                }), Default::default()))
        }

        if target.sig.inputs.len() == 1 {
            target.sig.inputs.push(simple_fn_arg!(this: &Self));
        }

        target.field_name = Some(target.sig.ident.clone());
        target.sig.ident = Ident::new(&*format!("__lua_get_{}", target.sig.ident), Span::call_site());
    }

    fn complete_signature_setter(target: &mut FunctionDefinition) {
        let fn_name = &target.sig.ident;

        if target.sig.inputs.len() == 0 {
            target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                    compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! A setter must has at least one argument."));
            }), Default::default()));
            return;
        }

        if target.sig.inputs.len() == 1 {
            let ty = target.sig.inputs.pop().unwrap();

            if let FnArg::Typed(_) = ty.value() {
                target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
                target.sig.inputs.push(simple_fn_arg!(this: &mut Self));
                target.sig.inputs.push(ty.value().clone())
            }
        }

        if target.sig.inputs.len() == 2 {
            /* Default behavior for setter is that the first arg is default to the value being set */
            let ty = target.sig.inputs.pop().unwrap();

            if let FnArg::Typed(_) = ty.value() {
                target.sig.inputs.push(simple_fn_arg!(this: &mut Self));
                target.sig.inputs.push(ty.value().clone())
            }

        }

        if matches!(target.sig.output, ReturnType::Default) {
            target.sig.output = ReturnType::Type(
                Default::default(),
                simple_type!(mlua::Result<()>)
            );
        }

        target.field_name = Some(target.sig.ident.clone());
        target.sig.ident = Ident::new(&*format!("__lua_set_{}", target.sig.ident), Span::call_site());
    }

    fn complete_signature_static(target: &mut FunctionDefinition) {
        if target.sig.inputs.len() == 0 {
            target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
        }

        if target.sig.inputs.len() == 1 {
            target.sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));
        }

        if matches!(target.sig.output, ReturnType::Default) {
            target.sig.output = ReturnType::Type(
                Default::default(),
                simple_type!(mlua::Result<()>)
            );

            target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! { return Ok(()) }), Default::default()))
        }
    }

    fn complete_signature_lua(target: &mut FunctionDefinition) {
        if target.sig.inputs.len() == 0 {
            target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
        }

        if target.sig.inputs.len() == 1 {
            target.sig.inputs.push(simple_fn_arg!(this: &Self));
        }

        if target.sig.inputs.len() == 2 {
            target.sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));
        }

        if matches!(target.sig.output, ReturnType::Default) {
            target.sig.output = ReturnType::Type(
                Default::default(),
                simple_type!(mlua::Result<()>)
            );

            target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! { return Ok(()) }), Default::default()))
        }
    }

    fn complete_signature_operator(target: &mut FunctionDefinition) {
        let fn_name = &target.sig.ident;

        if target.is_static() {
            if target.sig.inputs.len() == 0 {
                target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
            }

            if target.sig.inputs.len() == 1 {
                target.sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));
            }

            let real_name = META_FUNCTIONS.get(&fn_name.to_string());

            if real_name.is_none() {
                target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                    compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Unknown operator!"));
                }), Default::default()));
            }

            target.sig.ident = Ident::new(&*real_name.unwrap().0, Span::call_site());
        } else {
            if target.sig.inputs.len() == 0 {
                target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
            }

            if target.sig.inputs.len() == 1 {
                target.sig.inputs.push(simple_fn_arg!(this: &Self));
            }

            if target.sig.inputs.len() == 2 {
                target.sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));
            }

            let real_name = META_METHODS.get(&fn_name.to_string());

            if real_name.is_none() {
                target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                    compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Unknown operator!"));
                }), Default::default()));

                return;
            }

            target.sig.ident = Ident::new(&*real_name.unwrap().0, Span::call_site());
        }
    }
}

#[cfg(feature = "simple")]
impl FunctionDefinition {
    fn wrap_return_val(&mut self) {
        let rtn_raw = self.sig.output.clone();

        if let ReturnType::Default = self.sig.output {
            self.sig.output = ReturnType::Type(Default::default(), simple_type!(mlua::Result<()>));

            self.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                return Ok(())
            }), Default::default()))
        } else if let ReturnType::Type(_, ty) = rtn_raw {
            let ty = Box::new(syn::Type::Verbatim(quote! {
                mlua::Result<#ty>
            }));

            self.sig.output = ReturnType::Type(Default::default(), ty);
        } else {
            unreachable!("Impossible code reached! syn::ReturnType has only two variants at the time this code was written.")
        }
    }

    fn wrap_multi_args(&mut self) {
        let fn_name = self.sig.ident.clone();

        let mut index = 0usize;
        for arg in self.sig.inputs.clone() {
            if let FnArg::Typed(arg) = arg {

                let ty = &arg.ty;
                let mat = arg.pat;

                match *mat {
                    Pat::Ident(param) => {
                        let name = param.ident;

                        self.body.stmts.insert(0, Stmt::Expr(Expr::Verbatim(quote! {
                            let #name: #ty = mlua::FromLua::from_lua((args.get(#index).ok_or(Error::BadArgument {
                                    to: Some(stringify!(#fn_name).to_string()),
                                    pos: #index,
                                    name: Some(stringify!(#name).to_string()),
                                    cause: std::sync::Arc::new(Error::RuntimeError("Missing argument".to_string())),
                                })?).clone(), lua)?;
                        }), Default::default()));

                        index += 1;
                    },
                    _ => {
                        self.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                            compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! `self` is not supported! Please use implicit declared `this` as `self`"));
                        }), Default::default()))
                    }
                }

            } else {
                self.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                    compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Unsupported syntax!"));
                }), Default::default()))
            }
        }
    }
}

#[cfg(feature = "simple")]
impl CompleteFnSignature for FunctionDefinition {
    fn complete_signature_getter(target: &mut FunctionDefinition) {
        if target.sig.inputs.len() != 0 {
            let fn_name = target.sig.ident.clone();

            target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Too much arguments (expected 0)!"));
            }), Default::default()));

            return;
        }
        target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
        target.sig.inputs.push(simple_fn_arg!(this: &Self));

        target.wrap_return_val();

        target.sig.ident = Ident::new(&*format!("__lua_get_{}", target.sig.ident), Span::call_site());
    }

    fn complete_signature_setter(target: &mut FunctionDefinition) {
        if target.sig.inputs.len() == 0 {
            target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
            target.sig.inputs.push(simple_fn_arg!(this: &mut Self));
            target.sig.inputs.push(simple_fn_arg!(args: mlua::Value));
        } else if target.sig.inputs.len() == 1 {
            target.sig.inputs.insert(0, simple_fn_arg!(lua: &mlua::Lua));
            target.sig.inputs.insert(1, simple_fn_arg!(this: &mut Self));
        } else {
            let fn_name = target.sig.ident.clone();

            target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Too much arguments (expected 1)!"));
            }), Default::default()));

            return;
        }

        target.wrap_return_val();

        target.sig.ident = Ident::new(&*format!("__lua_set_{}", target.sig.ident), Span::call_site());
    }

    fn complete_signature_static(target: &mut FunctionDefinition) {
        target.wrap_multi_args();

        target.sig.inputs.clear();
        target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
        target.sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));

        target.wrap_return_val();
    }

    fn complete_signature_lua(target: &mut FunctionDefinition) {
        target.wrap_multi_args();

        target.sig.inputs.clear();
        target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
        target.sig.inputs.push(simple_fn_arg!(this: &Self));
        target.sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));

        target.wrap_return_val();
    }

    fn complete_signature_operator(target: &mut FunctionDefinition) {
        let fn_name = target.sig.ident.clone();

        let real_name = if target.is_static() {
            META_FUNCTIONS.get(&fn_name.to_string())
        } else {
            META_METHODS.get(&fn_name.to_string())
        };

        if real_name.is_none() {
            target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Unknown operator!"));
            }), Default::default()));
        }

        let (real_name, params_allowed) = real_name.unwrap();

        if *params_allowed != -1 {
            if target.sig.inputs.len() != *params_allowed as usize {
                target.body.stmts.push(Stmt::Expr(Expr::Verbatim(quote! {
                    compile_error!(concat!("Error while attempt to complete function signature for ", stringify!(#fn_name), "! Illegal operation arguments! Expected ", stringify!(#params_allowed), "!"));
                }), Default::default()));
            }
        }

        target.sig.ident = Ident::new(real_name, Span::call_site());

        target.wrap_multi_args();

        target.sig.inputs.clear();
        target.sig.inputs.push(simple_fn_arg!(lua: &mlua::Lua));
        if !target.is_static() {
            target.sig.inputs.push(simple_fn_arg!(this: &Self));
        }
        target.sig.inputs.push(simple_fn_arg!(args: mlua::MultiValue));

        target.wrap_return_val();
    }
}