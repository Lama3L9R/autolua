# autolua

`autolua` is a collection of Rust macros designed for [mlua](https://github.com/mlua-rs/mlua), aimed at simplifying the binding process between Rust and Lua. It provides functionality to automatically generate `IntoLua`/`FromLua` traits and quickly define `UserData`, significantly reducing the workload of writing manual boilerplate code.

English | [简体中文](README.md)

---

## Table of Contents

- [Installation](#installation)
- [Core Features](#core-features)
    - [#[autolua] Attribute Macro](#autolua-attribute-macro)
        - [Arguments](#arguments)
        - [Field Skipping & Special Handling](#field-skipping--special-handling)
    - [bindlua! Macro](#bindlua-macro)
        - [Keywords](#keywords)
        - [Method Autocompletion](#method-autocompletion--injection)
- [About](#about)


## Installation

Add `autolua` and `mlua` to your `Cargo.toml`:

```toml
[dependencies]
autolua = "0.1"
mlua = { version = "0.11.5", features = ["lua54", "anyhow"] }

```

## Core Features

### `#[autolua]` Attribute Macro

This macro automatically generates implementations of `mlua::IntoLua` and `mlua::FromLua` for Rust structs, enabling easy value passing between Rust and Lua.

#### Example Usage

```rust
use mlua::Lua;
use autolua::autolua;

#[autolua(Into, RefInto, From)]
struct MyLuaData {
    number: u32,
    str: String,

    // Fields marked with #[skip] will not be converted to Lua.
    // When converting back from Lua to Rust, it will attempt to use a default value or special handling.
    #[skip]
    skip_me: SomeInternalType,

    // Directly keep mlua value types
    dont_deluaify: mlua::Value,
    keep_the_original_taste: mlua::Table
}


```

#### Arguments

`#[autolua(...)]` accepts a combination of the following arguments:

* **`Into`**: Implements the `IntoLua` trait for `MyLuaData`. Allows moving ownership of the struct to Lua.
* **`RefInto`**: Implements the `IntoLua` trait for `&MyLuaData`. All fields will be `Clone`-ed during conversion.
* **`From`**: Implements the `FromLua` trait for `MyLuaData`. Allows automatic conversion from a Lua Table structure to a Rust struct.

#### Field Skipping & Special Handling

* **`mlua::Value` / `mlua::Table**`: If the field itself is an `mlua` type, the macro will keep it as-is without applying extra conversion logic.

> **⚠️ Important Note regarding `#[skip]**`
> Please **DO NOT use** the `#[skip]` attribute at this time.
> The `autolua` project was split from the `viator` project, and the `skip` feature relies on another sub-project, `viator-utils`, which has not yet been released independently. Therefore, using `#[skip]` now will cause the generated code to reference a non-existent struct (`viator_utils::MaybeValue`), resulting in a **compilation failure**. This feature will be fixed in the future.

---

### `bindlua!` Macro

`bindlua!` is a powerful macro used to define structs that possess both native Rust functionality and Lua `UserData` bindings. It allows you to specify which fields and methods should be exposed to Lua directly within the struct definition.

#### Example Usage

```rust
use autolua::bindlua;
use mlua::{Lua, Result};

// Assuming these types already exist
type AnotherIntoLua = String;
struct HasRefIntoLuaStruct; 
struct NoIntoLuaStruct;

bindlua! {
    #[derive(Debug)]
    lua pub MyStruct {
        // Fields exposed to Lua
        lua x: u32,
        lua y: String,
        
        // Both Rust pub and exposed to Lua
        lua pub z: Option<AnotherIntoLua>
        
        // Using the ref keyword indicates this field's type implements IntoLua for references
        lua pub ref refImplIntoLua: HasRefIntoLuaStruct
        
        // Normal Rust field, will not appear in Lua
        not_going_be_in_lua: NoIntoLuaStruct

        // Bind Lua method
        lua pub fn doSomething() {
            // Args and return values are handled automatically
            println!("Doing something in Lua!");
            
            // The macro automatically generates Ok(()) at the end unless you return manually
        }

        // Normal Rust method
        pub fn regular_rust_fn() {
            todo!()
        }
    }
}


```

#### Explanation

`bindlua!` generates two parts of code:

1. **Native Rust Struct Definition**: Contains all fields (regardless of whether they have the `lua` tag).
2. **`mlua::UserData` Implementation**: Contains only fields and methods marked with `lua`.

#### Keywords

Inside the `bindlua!` block, you can use the following modifiers:

* **`lua`**: The core tag. Only fields or functions with this tag will be added to `UserData` and accessible in Lua scripts.
* **`pub`**: Standard Rust visibility modifier.
* **`ref` (Fields only)**: Marks that the field's type implements `IntoLua` for `&Type`. When generating a getter, the macro uses reference conversion (`self.field.into_lua(lua)`) instead of cloning (`self.field.clone().into_lua(lua)`).
* **`mut`**: Used to mark mutable fields.
* **`get` / `set`**: Used to mark functions as getters or setters similar to Kotlin. Must be used with the `lua` modifier.

#### Method Autocompletion & Injection

For functions marked with `lua`, the macro provides smart argument injection:

* **Argument Autocompletion**: You don't need to manually declare all context arguments. The macro completes the argument list to `(lua: &Lua, this: &Self, args: mlua::MultiValue)`.
* If you only need the `lua` instance, just declare `lua: &Lua`, and the macro handles the rest.
* **Default Return Value**: Functions default to returning `mlua::Result<()>`.
* **Auto-Return Ok**: If using the default return value, the macro automatically appends `return Ok(());` to the end of the function.

## About

This project adopts the **Anti-996** License.

The purpose of choosing this license is to ensure the public **does not forget**: even today, hardworking programmers (and laborers in other industries) are still suffering from oppression by capitalists with nowhere to turn for justice.

You are **not required** to strictly follow all legal terms of the license, but you are **encouraged** to include the author's name (**lamadaemon**) and a link to [996.icu](https://996.icu) when referencing or distributing this project, to voice support for labor rights.

> This README document was generated by **Google Gemini 3 Pro** based on the `lib.rs` source code. The content has been reviewed by the author **lamadaemon**, who takes the full responsibility for the accuracy of the document. Furthermore, **all source code in this project, including inline documentation, contains NO AI-generated content**.