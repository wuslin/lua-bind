//! # Lua绑定库
//!
//! 提供Rust与Lua之间的安全绑定功能
//!
//! ## 基本用法示例
//!
//! ```rust,no_run
//! use lua_bind::{get_lua, register_binding};
//!
//! #[derive(Default)]
//! struct MathUtils;
//!
//! impl mlua::UserData for MathUtils {
//!     fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
//!         methods.add_method("add", |_, _, (a, b): (i32, i32)| Ok(a + b));
//!         methods.add_method("sub", |_, _, (a, b): (i32, i32)| Ok(a - b));
//!     }
//! }
//!
//! register_binding!(MathUtils);
//!
//! fn main() {
//!     let lua = get_lua().unwrap();
//!     lua.load(r#"
//!         local math = Rust.MathUtils
//!         print("5 + 3 =", math:add(5, 3))
//!         print("5 - 3 =", math:sub(5, 3))
//!     "#).exec().unwrap();
//! }
//! ```

mod error;
mod state;
mod bind;

pub use error::{Result, LuaBindError};
pub use bind::{BINDINGS, LuaBindings};
pub use state::{get_lua, call_with};

#[cfg(feature = "async")]
pub use state::call_async_with;
