//! Lua绑定系统模块，提供自动注册和类型绑定功能

use std::any::TypeId;
use linkme::distributed_slice;

use super::error::Result;

/// Lua绑定trait，所有需要注册到Lua的类型必须实现此trait
pub trait LuaBindings: Send + Sync + 'static {
    /// 获取类型的完整名称
    fn name(&self) -> &'static str;

    /// 获取类型的简短名称(去掉命名空间)
    fn short(&self) -> &'static str;

    /// 获取类型的TypeId
    fn type_id(&self) -> TypeId;

    /// 将类型注册到Lua环境中
    fn register(&self, lua: &mlua::Lua, table: &mlua::Table) -> Result<()>;
}

impl<T> LuaBindings for T
where
    T: mlua::UserData + Default + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn short(&self) -> &'static str {
        let full = self.name();
        full.rsplit("::").next().unwrap_or(full)
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
    
    fn register(&self, lua: &mlua::Lua, table: &mlua::Table) -> Result<()> {
        let proxy = lua.create_proxy::<Self>()?;
        table.set(self.short(), proxy)?;
        Ok(())
    }
}

#[distributed_slice]
pub static BINDINGS: [fn() -> Box<dyn LuaBindings>] = [..];

#[macro_export]
macro_rules! register_binding {
    ($t:ty) => {
        paste::paste! {
            #[linkme::distributed_slice($crate::BINDINGS)]
            static [<__REGISTER_ $t:upper>]: fn() -> Box<dyn $crate::LuaBindings> = || {
                Box::new(<$t>::default()) as Box<dyn $crate::LuaBindings>
            };
        }
    };
}

/// 遍历所有注册的绑定并初始化Lua环境
pub(crate) fn iterate_bindings(lua: &mlua::Lua) -> Result<()> {
    let registry = lua.create_table()?;
    for constructor in BINDINGS {
        let binding = constructor();
        binding.register(lua, &registry)?;
    }
    lua.globals().set("Rust", registry).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::get_lua;
    use rstest::rstest;
    use mlua::UserDataMethods;

    #[derive(Default, Clone)]
    struct TestBinding;

    impl mlua::UserData for TestBinding {
        fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
            methods.add_function("new", |_, ()| Ok(TestBinding) );
            methods.add_method("test", |_, _, ()| Ok("test passed") );
        }
    }

    register_binding!(TestBinding);

    #[rstest]
    fn test_binding_registration() {
        let lua = get_lua().unwrap();
        
        let result: String = lua.load(r#"Rust.TestBinding.new():test()"#).eval().unwrap();
        assert_eq!(result, "test passed");
    }

    #[test]
    fn test_multiple_bindings() {
        #[derive(Default)]
        struct Api1;
        #[derive(Default)]
        struct Api2;

        impl mlua::UserData for Api1 {
            fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
                methods.add_function("hello", |_, ()| Ok("hello"));
            }
        }

        impl mlua::UserData for Api2 {
            fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
                methods.add_function("world", |_, ()| Ok("world"));
            }
        }

        register_binding!(Api1);
        register_binding!(Api2);

        let lua = get_lua().unwrap();

        let s: String = lua.load(r#"
            return Rust.Api1.hello() .. " " .. Rust.Api2.world()
        "#).eval().unwrap();
        assert_eq!(s, "hello world");
    }
}
