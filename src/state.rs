//! Lua状态管理模块，提供线程安全的Lua环境管理

use std::sync::Mutex;
use std::cell::RefCell;
use lazy_static::lazy_static;


use super::error::Result;
use super::bind::iterate_bindings;

struct LuaPtr(*mut mlua::Lua);

unsafe impl Send for LuaPtr {}

lazy_static! {
    static ref GLOBAL_LUA_INSTANCES: Mutex<Vec<LuaPtr>> = Mutex::new(Vec::new());
}

thread_local! {
    static LUA_GUARD: RefCell<Option<LuaGuard>> = RefCell::new(None);
}

struct LuaGuard {
    lua: &'static mlua::Lua,
}

impl LuaGuard {
    fn new() -> Result<Self> {
        let lua = Box::leak(Box::new(mlua::Lua::new()));
        iterate_bindings(lua)?;
        GLOBAL_LUA_INSTANCES.lock().unwrap().push(LuaPtr(lua as *mut _));
        Ok(Self { lua })
    }
}

impl Drop for LuaGuard {
    fn drop(&mut self) {
        let mut instances = GLOBAL_LUA_INSTANCES.lock().unwrap();
        let ptr = self.lua as *const _ as *mut _;
        instances.retain(|p| p.0 != ptr);
        unsafe { let _ = Box::from_raw(ptr); };
    }
}

pub fn get_lua() -> Result<&'static mlua::Lua> {
    LUA_GUARD.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = Some(LuaGuard::new()?);
        }
        Ok(guard.as_ref().unwrap().lua)
    })
}

/// 获取Lua函数引用
pub fn get_func(module: &str, func_name: &str) -> Result<mlua::Function<'static>> {
    let lua = get_lua()?;
    let module_loader = lua.load(format!("return require('{}')", module));
    let table : mlua::Table = module_loader.eval()?;
    Ok(table.get::<_, mlua::Function<'static>>(func_name)?)
}

/// 调用Lua模块函数
pub fn call_with<F, R>(module: &str, func_name: &str, callback: F) -> Result<R> 
where
    F: FnOnce(&mlua::Function) -> mlua::Result<R>,
{
    let func = get_func(module, func_name)?;
    Ok(callback(&func)?)
}

/// 异步调用Lua模块函数
#[cfg(feature = "async")]
pub async fn call_async_with<F, R, Fut>(module: &str, func_name: &str, callback: F) -> Result<R>
where
    F: FnOnce(String, String) -> Fut,
    Fut: std::future::Future<Output = mlua::Result<R>>,
{
    Ok(callback(module.to_string(), func_name.to_string()).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use rstest::rstest;

    #[rstest]
    fn test_get_lua() {
        let lua = get_lua().unwrap();
        assert!(!lua.load("print('test')").exec().is_err());
    }

    #[rstest]
    fn test_call_with() {
        let result = call_with("math", "abs", |func| func.call::<_, i32>(-42)).unwrap();
        assert_eq!(result, 42);
    }

    #[rstest]
    fn test_thread_safety() {
        let handles: Vec<_> = (0..4).map(|_| {
            thread::spawn(|| {
                let lua = get_lua().unwrap();
                lua.load("print('thread test')").exec().unwrap();
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_call_async_with() {
        let lua = get_lua().unwrap();
        lua.load(r#"
            function async_add(a, b)
                return a + b
            end
        "#).exec().unwrap();

        let result = call_async_with("_G", "async_add", |module, func_name| async move {
            let func = get_func(&module, &func_name).map_err(|e| mlua::Error::external(e))?;
            func.call_async::<_, i32>((5, 7)).await
        }).await.unwrap();
        assert_eq!(result, 12);
    }
}
