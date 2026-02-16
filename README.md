# autolua

`autolua` 是一个为 [mlua](https://github.com/mlua-rs/mlua) 设计的 Rust 宏集合，旨在简化 Rust 与 Lua 之间的绑定过程。它提供了自动生成 `IntoLua`/`FromLua` 转换特质（Trait）以及快速定义 `UserData` 的功能，极大地减少了手动编写样板代码的工作量。

[English](README.en.md) | 简体中文

[![996.icu](https://img.shields.io/badge/link-996.icu-red.svg)](https://996.icu)
[![LICENSE](https://img.shields.io/badge/license-Anti%20996-blue.svg)](https://github.com/996icu/996.ICU/blob/master/LICENSE)

---
## 目录

- [安装](#安装)
- [核心功能](#核心功能)
    - [#[autolua] 属性宏](#autolua-属性宏)
        - [参数说明](#参数说明)
        - [字段跳过与特殊处理](#字段跳过与特殊处理)
    - [bindlua! 宏](#bindlua-宏)
        - [关键字说明](#关键字说明)
        - [方法自动补全](#方法自动补全与注入)
- [关于](#关于)

## 安装

在你的 `Cargo.toml` 中添加 `autolua` ：

```toml
[dependencies]
autolua = "0.1.2"
```

## 核心功能

### `#[autolua]` 属性宏

该宏用于为 Rust 结构体自动生成 `mlua::IntoLua` 和 `mlua::FromLua` 的实现，使得结构体可以轻松地在 Rust 和 Lua 之间进行值传递。

#### 使用案例

```rust
use mlua::Lua;
use autolua::autolua;

#[autolua(Into, RefInto, From)]
struct MyLuaData {
    number: u32,
    str: String,

    // 使用 #[skip] 标记的字段将不会被转换到 Lua 中，
    // 在从 Lua 转换回 Rust 时会尝试使用默认值或特殊处理
    #[skip]
    skip_me: SomeInternalType,

    // 直接保留 mlua 的值类型
    dont_deluaify: mlua::Value,
    keep_the_original_taste: mlua::Table
}

```

#### 参数说明

`#[autolua(...)]` 接受以下参数的组合：

* **`Into`**: 为 `MyLuaData` 实现 `IntoLua` 特质。允许将结构体所有权移交给 Lua。
* **`RefInto`**: 为 `&MyLuaData` 实现 `IntoLua` 特质。在转换过程中，所有字段会被 `Clone`。
* **`From`**: 为 `MyLuaData` 实现 `FromLua` 特质。允许从 Lua 的 Table 结构自动转换为 Rust 结构体。

#### 字段跳过与特殊处理

* **`mlua::Value` / `mlua::Table`**: 如果字段本身就是 `mlua` 的类型，宏会按原样保留，不会进行额外的转换逻辑。

> **⚠️ 关于 `#[skip]` 的重要说明**
> 目前 **请勿使用** `#[skip]` 属性。
> `autolua` 项目拆分自 `viator` 项目，而 `skip` 功能的实现依赖于另一个尚未独立发布的子项目 `viator-utils`。因此，如果现在使用 `#[skip]`，宏生成的代码将会引用一个不存在的结构体（`viator_utils::MaybeValue`），从而导致**编译失败**。该功能将在未来修复。

---

### `bindlua!` 宏

`bindlua!` 是一个功能强大的宏，用于定义同时具备 Rust 原生功能和 Lua `UserData` 绑定的结构体。它允许你在定义结构体的同时指定哪些字段和方法应该暴露给 Lua。

#### 使用案例

```rust
use autolua::bindlua;
use mlua::{Lua, Result};

// 假设这些类型已经存在
type AnotherIntoLua = String;
struct HasRefIntoLuaStruct; 
struct NoIntoLuaStruct;

bindlua! {
    #[derive(Debug)]
    lua pub MyStruct {
        // 暴露给 Lua 的字段
        lua x: u32,
        lua y: String,
        
        // 既是 Rust pub，也暴露给 Lua
        lua pub z: Option<AnotherIntoLua>
        
        // 使用 ref 关键字，表示该字段类型实现了针对引用的 IntoLua
        lua pub ref refImplIntoLua: HasRefIntoLuaStruct
        
        // 普通的 Rust 字段，不会出现在 Lua 中
        not_going_be_in_lua: NoIntoLuaStruct

        // 绑定 Lua 方法
        lua pub fn doSomething() {
            // 参数和返回值会被自动处理
            println!("Doing something in Lua!");
            
            // 宏会自动在末尾生成 Ok(())，除非你手动返回
        }

        // 普通的 Rust 方法
        pub fn regular_rust_fn() {
            todo!()
        }
    }
}

```

#### 解释

`bindlua!` 会生成两部分代码：

1. **原生 Rust 结构体定义**：包含所有字段（无论是否有 `lua` 标记）。
2. **`mlua::UserData` 实现**：仅包含标记为 `lua` 的字段和方法。

#### 关键字说明

在 `bindlua!` 块内部，你可以使用以下修饰符：

* **`lua`**: 核心标记。只有带有此标记的字段或函数才会被添加到 `UserData` 中，从而可以在 Lua 脚本中访问。
* **`pub`**: 标准 Rust 可见性修饰符。
* **`ref` (仅字段)**: 标记该字段的类型实现了 `IntoLua` for `&Type`。生成 getter 时，宏会直接使用引用转换 (`self.field.into_lua(lua)`) 而不是克隆 (`self.field.clone().into_lua(lua)`)。
* **`mut`**: 用于标记可变字段。
* **`get` / `set`**: 用于将函数标记为类似于 Kotlin 的 getter 或 setter。需要搭配 `lua` 修饰符使用。

#### 方法自动补全与注入

在 `lua` 标记的函数中，宏提供了智能的参数注入：

* **参数自动补全**: 你不需要手动声明所有上下文参数。宏会将参数列表补全为 `(lua: &Lua, this: &Self, args: mlua::MultiValue)`。
* 如果你只需要 `lua` 实例，只需声明 `lua: &Lua`，宏会处理其余部分。


* **返回值默认值**: 函数默认返回 `mlua::Result<()>`。
* **自动返回 Ok**: 如果是默认返回值，宏会自动在函数末尾追加 `return Ok(());`。

## 关于

本项目采用 **Anti-996** 协议。

选择此协议的初衷，是为了使大众不要忘记：时至今日，辛苦工作的程序员（以及其他行业的劳动者）们仍在遭受资本家的压迫却无处伸冤。

您**无需**严格遵循协议的全部法律条款，但**鼓励**您在引用或分发本项目时，附上作者名称 (**lamadaemon**) 以及 [996.icu](https://996.icu) 的链接，以声援劳动者权益。

> 本 README 文档基于 `lib.rs` 源码由 **Google Gemini 3 Pro** 生成。内容已由作者 **lamadaemon** 人工审查并确认，作者对文档内容的准确性负责。此外，**本项目的所有源代码包括行内文档均不包含任何 AI 生成的内容**。
