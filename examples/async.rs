fn main() {
    #[cfg(feature = "test-async")]
    {
        use lua_bind::get_lua;
        use tokio::runtime::Runtime;
        use lua_bind::register_binding;

        #[derive(Default)]
        struct AsyncApi;
        
        impl mlua::UserData for AsyncApi {
            fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
                methods.add_function("new", |_, ()| Ok(AsyncApi) );

                #[cfg(feature = "async")]
                methods.add_async_method("fetch_data", |_, _, url: String| async move {
                    let result = reqwest::get(&url).await;
                    match result {
                        Ok(resp) => resp.text().await.map_err(|e| mlua::Error::external(e)),
                        Err(e) => Err(mlua::Error::external(e)),
                    }
                });
            }
        }
    
        register_binding!(AsyncApi);

        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let lua = get_lua().unwrap();
            lua.load(r#"
                local api = Rust.AsyncApi.new()
                local co1 = coroutine.create(function()
                    local data = api:fetch_data("https://httpbin.org/get")
                    print("Data:", data)
                end)
                local co2 = coroutine.create(function()
                    local data = api:fetch_data("https://httpbin.org/get")
                    print("Data:", data)
                end)
                coroutine.resume(co1)
                coroutine.resume(co2)
            "#).exec().unwrap();
        });
    }

    #[cfg(not(feature = "test-async"))]
    {
        println!("请使用 --features test-async 运行此示例");
    }
}
