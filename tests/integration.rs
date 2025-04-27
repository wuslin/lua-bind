use lua_bind::{get_lua, register_binding};
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
}

impl mlua::UserData for User {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("new", |_, (name, age) : (String, u32)| Ok(User {name : name, age : age }));
        methods.add_method("get_name", |_, this, ()| Ok(this.name.clone()));
        methods.add_method("to_json", |_, this, ()| {
            Ok(serde_json::to_string(this).unwrap())
        });
    }
}

register_binding!(User);

#[test]
fn test_complex_types() {
    let lua = get_lua().unwrap();
    lua.load(r#"
        local user = Rust.User.new("Alice", 21)
        print(user:get_name())  -- 输出 "Alice"
        print(user:to_json())   -- 输出 JSON
    "#).exec().unwrap();
}
