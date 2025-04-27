use lua_bind::{get_lua, register_binding};

#[derive(Default)]
struct MathUtils;

impl mlua::UserData for MathUtils {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("new", |_, (): ()| Ok(MathUtils));
        methods.add_method("add", |_, _, (a, b): (i32, i32)| Ok(a + b));
        methods.add_method("sub", |_, _, (a, b): (i32, i32)| Ok(a - b));
    }
}

register_binding!(MathUtils);

fn main() {
    let lua = get_lua().unwrap();
    lua.load(r#"
        local math = Rust.MathUtils.new()
        print("5 + 3 =", math:add(5, 3))
        print("5 - 3 =", math:sub(5, 3))
    "#).exec().unwrap();
}