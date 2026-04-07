//! Toki 超主权数字货币平台 - API 模块

pub mod handlers;
pub mod rest;
pub mod routes;
pub mod websocket;

pub use handlers::ApiState;
pub use rest::*;
pub use websocket::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_api_module() {
        assert!(true);
    }
}
