//! Toki 超主权数字货币平台 - API 模块

pub mod rest;
pub mod websocket;
pub mod handlers;
pub mod routes;

pub use rest::*;
pub use websocket::*;
pub use handlers::ApiState;

#[cfg(test)]
mod tests {
    #[test]
    fn test_api_module() {
        assert!(true);
    }
}
