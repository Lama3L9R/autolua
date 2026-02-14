//!
//! ## Welcome!
//!
//! autolua is a crate to generate `IntoLua`, `FromLua`, and `UserData` implementations **QUICK**!
//!
//! Please check out the only two macros below for detailed explanations.
//!
//! TLDR? NP!
//! - `#[autolua(Into, From)]`: Used on structs, to generate IntoLua and FromLua.
//! - `bindlua! {  }`: Generate UserData based on your needs.
//!
//! ## FAQ
//!
//! - Why Not Serde?
//!     + Well, serde does not allow you to have fields with type `mlua::Value`.
//!     + This is originally why I wrote `autolua`.
//! - Is this project stable?
//!     + *NO!* Please expect breaking changes.
//!

mod bindlua;
mod imported;
mod autolua;

use proc_macro::TokenStream;
use syn::{parse_macro_input};
use crate::autolua::{do_autolua, AutoLuaArgs};
use crate::bindlua::{do_bindlua, BindLua};

type TokStream = proc_macro2::TokenStream;

///
/// Auto generate IntoLua and/or FromLua
///
/// Examples:
/// Generate IntoLua and FromLua
/// ```
/// #[autolua(Into, RefInto, From)]
/// struct MyLuaData {
///     number: u32,
///     str: String,
///
///     #[skip]
///     skip_me: SomeOtherStuff,
///
///     dont_deluaify: mlua::Value,
///     keep_the_original_taste: mlua::Table
/// }
/// ```
///
/// - `Into`: will generate IntoLua implementation on MyLuaData.
/// - `RefInto`: will generate IntoLua implementation on &MyLuaData with all fields Clone-ed
/// - `From`: will generate FromLua implementation strictly from a Table type
///
/// Note: if `#[skip]` is used on a field, the type of the field will
/// be transformed into `viator-utils::MaybeValue<T>`,
/// which impl Deref, and will panic if null is being used / deref-ed.
///
#[proc_macro_attribute]
pub fn autolua(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AutoLuaArgs);

    return do_autolua(args, input).unwrap().into();
}
///
/// Generate UserData used for pure rust binding to lua.
/// You declare structs with almost the same syntax!
/// Except, we have new keywords like `lua`.
/// The following example is the best tutorial for this macro, so go take a look!
///
/// Example
/// ```
/// bindlua! {
///     #[derive(Debug)]
///     lua pub MyStruct {
///         lua x: u32,
///         lua y: String,
///         lua pub z: Option<AnotherIntoLua>
///         lua pub ref refImplIntoLua: HasRefIntoLuaStruct
///         not_going_be_in_lua: NoIntoLuaStruct
///
///         lua pub fn doSomething(/* Optional args */) /*Optional Return Type*/ {
///             // Args will be auto completed to `lua: &Lua, this: &Self, args: mlua::MultiValue`
///             // If you only need &Lua, then declare only `lua: &Lua`, and
///             // bindlua will complete the remaining types for you.
///             // Return type is default to mlua::Result<()>
///
///             // bindlua will generate `return Ok(())` automatically for you at the end of this function.
///         }
///
///         pub fn regular_rust_fn() {
///             todo!()
///         }
///     }
/// }
/// ```
///
/// Will generate the following code:
/// ```
/// #[derive(Debug)]
/// pub MyStruct {
///     x: u32,
///     y: String,
///     pub z: Option<AnotherIntoLua>
///     pub ref refImplIntoLua: HasRefIntoLuaStruct
///     not_going_be_in_lua: NoIntoLuaStruct
/// }
///
/// impl MyStruct {
///     pub fn doSomething(lua: &Lua, this: &Self, args: mlua::MultiValue) -> mlua::Result<()> {
///         return Ok(());
///     }
///
///     pub fn regular_rust_fn() {
///         todo!()
///     }
/// }
///
/// impl UserData for MyStruct {
///     fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
///         add_field_method_get("x", |lua, this| {
///             return this.x.clone().into_lua(lua);
///         });
///
///         add_field_method_get("y", |lua, this| {
///             return this.y.clone().into_lua(lua);
///         });
///
///         add_field_method_get("z", |lua, this| {
///             return this.z.clone().into_lua(lua);
///         });
///
///         add_field_method_get("refImplIntoLua", |lua, this| {
///             // because we have ref qualifier
///             return this.refImplIntoLua.into_lua(lua);
///         });
///
///         // Fields like not_going_be_in_lua without a `lua` qualifier, will not be
///         // able to access in lua.
///     }
///
///     fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
///         methods.add_method("doSomething", Self::doSomething);
///     }
/// }
/// ```
///
/// Qualifiers:
///
/// - lua: marks a struct/field/function to be a part of lua binding
///        members without this qualifier, will not be added to UserData
/// - pub: same as rust pub
/// - ref: fields only; marks a field's type has implemented IntoLua for &Type
/// - mut: `[WIP]` `[MaybeRemoved]` make the field mutable
/// - get: `[WIP]` functions only; mark a lua function as a getter, similar to kotlin
/// - set: `[WIP]` functions only; mark a lua function as a setter, similar to kotlin
///
#[proc_macro]
pub fn bindlua(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as BindLua);

    return do_bindlua(body).unwrap().into();
}
