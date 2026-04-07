//! API 认证中间件
//!
//! 提供 JWT 认证和 API Key 认证

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// JWT 配置
#[derive(Clone)]
pub struct JwtConfig {
    /// 密钥
    pub secret: String,
    /// 过期时间（秒）
    pub expiration: u64,
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: String,
    /// 过期时间
    pub exp: usize,
    /// 签发时间
    pub iat: usize,
}

/// API Key 存储
pub struct ApiKeyStore {
    /// 有效的 API Keys
    keys: Vec<String>,
}

impl ApiKeyStore {
    /// 创建新的 API Key 存储
    pub fn new(keys: Vec<String>) -> Self {
        ApiKeyStore { keys }
    }

    /// 验证 API Key
    pub fn verify(&self, key: &str) -> bool {
        self.keys.iter().any(|k| k == key)
    }
}

/// 认证状态
#[derive(Clone)]
pub struct AuthState {
    /// JWT 配置
    pub jwt_config: JwtConfig,
    /// API Key 存储
    pub api_keys: Arc<ApiKeyStore>,
}

/// 生成 JWT Token
pub fn generate_token(
    user_id: &str,
    config: &JwtConfig,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let exp = now + chrono::Duration::seconds(config.expiration as i64);

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_ref()),
    )
}

/// 验证 JWT Token
pub fn verify_token(
    token: &str,
    config: &JwtConfig,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

/// JWT 认证中间件
pub async fn jwt_auth(
    State(state): State<AuthState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从 Header 中获取 Token
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    // 验证 Token
    if let Some(token) = token {
        if verify_token(token, &state.jwt_config).is_ok() {
            return Ok(next.run(request).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// API Key 认证中间件
pub async fn api_key_auth(
    State(state): State<AuthState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从 Header 中获取 API Key
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|header| header.to_str().ok());

    // 验证 API Key
    if let Some(key) = api_key {
        if state.api_keys.verify(key) {
            return Ok(next.run(request).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// 可选认证中间件（允许未认证访问）
pub async fn optional_auth(
    State(state): State<AuthState>,
    request: Request,
    next: Next,
) -> Response {
    // 检查 JWT Token
    let jwt_valid = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        })
        .map(|token| verify_token(token, &state.jwt_config).is_ok())
        .unwrap_or(false);

    // 检查 API Key
    let api_key_valid = request
        .headers()
        .get("X-API-Key")
        .and_then(|header| header.to_str().ok())
        .map(|key| state.api_keys.verify(key))
        .unwrap_or(false);

    // 如果任一认证方式有效，添加认证标记
    let mut request = request;
    if jwt_valid || api_key_valid {
        request.extensions_mut().insert(Authenticated(true));
    }

    next.run(request).await
}

/// 认证标记
#[derive(Clone, Copy)]
pub struct Authenticated(pub bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let config = JwtConfig {
            secret: "test_secret".to_string(),
            expiration: 3600,
        };

        let token = generate_token("user1", &config).unwrap();
        let claims = verify_token(&token, &config).unwrap();

        assert_eq!(claims.sub, "user1");
    }

    #[test]
    fn test_api_key_verification() {
        let store = ApiKeyStore::new(vec![
            "key1".to_string(),
            "key2".to_string(),
        ]);

        assert!(store.verify("key1"));
        assert!(store.verify("key2"));
        assert!(!store.verify("key3"));
    }
}
