#[derive(Debug, thiserror::Error)]
pub enum LuaBindError {
    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),
    
    #[error("Initialization error: {0}")]
    Init(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Type conversion error: {0}")]
    TypeError(String),
}

pub type Result<T> = std::result::Result<T, LuaBindError>;
