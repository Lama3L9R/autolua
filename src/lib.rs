//!
//! ## Welcome!
//!
//! autolua is a crate to generate `IntoLua`, `FromLua`, and `UserData` implementations **QUICK**!
//!
//! Please check out the only two macros below for detailed explanations.
//!
//! TLDR? NP!
//! - `#[autolua(Into, From)]`: Used on structs, to generate IntoLua and FromLua.
//! - `bindlua! { ... }`: Generate UserData based on your needs.
//!     + `feature = ["simple"]` makes your life even easier.
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
/// # bindlua
///
/// Generate UserData used for pure rust binding to lua.
/// You declare structs with almost the same syntax!
/// Except, we have new keywords like `lua`.
/// The following example is the best tutorial for this macro, so go take a look!
///
/// **As starting from 0.2.0, the simple syntax is recommended. Please check out Simple Syntax section for more information**
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
///
///         lua get fn fancyVar() -> mlua::Result<String> {
///             return Ok("Hi! Lua".to_string());
///         }
///
///         lua set fn fancyVar(str: String) {
///             println!("set variable defined by getter/setter: fancyVar set to {}", str);
///             return Ok(())
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
///     pub refImplIntoLua: HasRefIntoLuaStruct
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
/// impl mlua::UserData for MyStruct {
///     fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
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
///
///         fields.add_field_method_get("fancyVar", Self::__lua_get_fancyVar);
///         fields.add_field_method_set("fancyVar", Self::__lua_set_fancyVar);
///     }
///
///     fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
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
/// - mut: make the field mutable
/// - get: functions only; mark a lua function as a getter, similar to kotlin
/// - set: functions only; mark a lua function as a setter, similar to kotlin
/// - static: functions only; mark a function as a static function, which does not assume the first param is this
/// - operator: functions only; mark a function as an operator function, see below for more details
///
/// ## Getter and Setter
///
/// You may have noticed that `get` and `set` functions takes 0 and 1 params.
/// This is because a special logic for function param completion is applied.
/// Where `get` behaves the same as regular `lua` functions, that the signature will be complete to
/// `(lua: &mlua::Lua, this: &Self)`.
/// `set` behaves differently, that if the signature has only 1 parameter, it will be complete to
/// `(lua: &mlua::Lua, this: &Self, <Your First Param Will Be Copied to Here>)`.
/// If you wish to have full control, you can explicitly define all three params.
///
/// Also, `get` and `set` function names will be mangled by concatenating `__lua_get_` and `__lua_set_` in the front.
///
/// ## Operators
/// To enhance readability, unlike original lua, where you name operator functions like `__add`,
/// instead, most of the operators has removed this `__` in front. Here is a list of all supported operators:
///
/// Unary operators (1 args for static, 0 for non-static)
/// - `negate` (`-a`)
/// - **(Lua 5.3+)** `not`
/// - `len` (`#`)
/// - `onGarbageCollect` (`__gc`)
/// - `toString`
/// - `pairs`
/// - **(Until Lua 5.4)** `ipairs`
///
/// Binary operators (2 args for static, 1 for non-static (self is the first operand))
/// - `add`, `sub`, `mul`, `div`, `mod`, `pow`, `idiv`
/// - **(Lua 5.3+)** `and`, `or`, `xor`, `shl`, `shr`
/// - `eq`, `lt`, `le`
/// - `concat` (`..`)
/// - `get` (`table[key]`)
///
/// Trinary operators (3 args for static, 2 for non-static (self is the first operand))
/// - `set` (`table[key] = value`)
///
/// Other
/// - `invoke` (`__call`)
///
/// ## Simple Syntax
///
/// To further simplify the boilerplate code, `0.2.0` introduced a brand-new alterative DSL syntax.
/// You can enable this syntax by enabling the `simple` feature.
/// Simple syntax only changes how `bindlua` completes your function, so there's no difference
/// for fields declarations etc.
///
/// Basically, with `simple` enabled, `bindlua` will automatically turn your raw rust types
/// into the type that mlua requires, which allows you to declare functions like this:
///
/// ```
/// // assume you have a field declared as: lua z: u32
///
/// lua fn myFunction(text: String, x: u32, y: u32) -> bool {
///     println!("text is {}, x is {}, y is {}, z is {}", text, x, y, this.z);
///
///     return Ok(x > y)
/// }
/// ```
///
/// Similar to regular syntax, `this` and `lua` argument is always accessible even when you didn't declare them,
/// which is known as "implicit arguments".
///
/// The third argument `args: MultiValue` is accessible as well.
/// Your explicit arguments will be retrieved from `args: MultiValue` automatically by
/// inserting `let text: String = <code that get text from args>;`, which will return
/// Err(mlua::Error::BadArgument) if `bindlua` failed to transform mlua::Value into the type you want,
/// or the value is missing.
///
/// The return value will automatically be wrapped by mlua::Result.
/// For functions that *implicitly* returns unit (a.k.a. `()`), similar to regular syntax,
/// `return Ok(())` is automatically inserted at the end of the function as well.
/// However, if you explicitly declare `()` as your return type, `return Ok(())` will not be generated.
///
/// ## FAQ
/// - Q: Does bindlua supports autolua?
/// - A: Yes it does. But since `skip` is currently still broken, `#[skip]` doesn't work.
///
#[proc_macro]
pub fn bindlua(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as BindLua);

    return do_bindlua(body).unwrap().into();
}
