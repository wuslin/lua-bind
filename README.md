# Lua-Bind - Rust与Lua的无缝绑定库

[![Crates.io](https://img.shields.io/crates/v/lua-bind)](https://crates.io/crates/lua-bind)
[![Docs.rs](https://docs.rs/lua-bind/badge.svg)](https://docs.rs/lua-bind)
[![License](https://img.shields.io/crates/l/lua-bind)](LICENSE)

一个高性能、线程安全的Rust到Lua的绑定系统，提供自动类型注册和依赖注入功能。

## 功能特性

- 🚀 **自动类型注册** - 通过宏自动将Rust类型暴露给Lua
- 🔒 **线程安全** - 内置线程安全的Lua状态管理
- ⚡ **高性能** - 零成本抽象，最小化运行时开销
- 🔄 **双向交互** - 支持双向调用(Rust→Lua和Lua→Rust)
- 📦 **模块化设计** - 可扩展的架构设计

## 安装

在`Cargo.toml`中添加：

```toml
[dependencies]
lua-bind = { version = "0.1", features = ["async"] }  # 按需启用特性
```

可选特性：
- `async` - 启用异步支持
- `vendored` - 静态链接Lua库

## 快速开始

### 基本类型绑定

```rust
use lua_bind::{get_lua, register_binding};

#[derive(Default)]
struct MathUtils;

impl mlua::UserData for MathUtils {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("add", |_, _, (a, b): (i32, i32)| Ok(a + b));
    }
}

register_binding!(MathUtils);

fn main() {
    let lua = get_lua().unwrap();
    lua.load(r#"
        print(Rust.MathUtils.add(1, 2))  -- 输出3
    "#).exec().unwrap();
}
```

### 异步支持

```rust
#[derive(Default)]
struct AsyncApi;

impl mlua::UserData for AsyncApi {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("fetch", |_, _, url: String| async {
            Ok(reqwest::get(&url).await?.text().await?)
        });
    }
}
```

## 进阶用法

### 序列化支持

```rust
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: u32
}

impl mlua::UserData for User {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("to_json", |_, this, ()| {
            Ok(serde_json::to_string(this)?)
        });
    }
}
```

### 多线程安全

```rust
let handles: Vec<_> = (0..4).map(|_| {
    thread::spawn(|| {
        let lua = get_lua().unwrap();
        // 安全并发访问
    })
}).collect();
```

## API参考

完整API文档见: [docs.rs/lua-bind](https://docs.rs/lua-bind)

## 示例代码

查看`examples/`目录获取更多示例：
- `basic.rs` - 基础用法
- `async.rs` - 异步交互
- `serialization.rs` - 序列化示例

### 运行示例

```bash
cargo run --example basic
cargo run --example async --features test-async
cargo run --example serialization
```

- `test-async` 只为了适配`async`案例设计的特性，不建议直接使用

## 贡献指南

欢迎提交Issue和PR！开发前请阅读：
1. 运行测试：`cargo test --all-features`
2. 检查格式：`cargo fmt --all`
3. 更新文档和CHANGELOG

## 许可证

本项目采用双重许可：
- MIT许可证（见[LICENSE](LICENSE)）
- Apache-2.0许可证（见[LICENSE](LICENSE)）
