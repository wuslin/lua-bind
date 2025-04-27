use lua_bind::{LuaBindError, get_lua, register_binding};
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
}

impl mlua::UserData for User {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("from_json", |_, json: String| {
            serde_json::from_str::<User>(&json)
                .map_err(|e| mlua::Error::external(LuaBindError::Serialization(e.to_string())))
        });
        methods.add_method("to_json", |_, this, ()| {
            serde_json::to_string(this)
                .map_err(|e| mlua::Error::external(LuaBindError::Serialization(e.to_string())))
        });
    }
}

register_binding!(User);

fn main() {
    let lua = get_lua().unwrap();
    lua.load(r#"
        local user = Rust.User.from_json('{"name":"Alice","age":25}')
        local json = user:to_json()
        print("JSON:", json)
    "#).exec().unwrap();
}