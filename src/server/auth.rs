/*
*   server::auth
*   Copyright (C) 2025 zlc
*
*   This program is free software: you can redistribute it and/or modify
*   it under the terms of the GNU General Public License as published by
*   the Free Software Foundation, either version 3 of the License, or
*   (at your option) any later version.
*
*   This program is distributed in the hope that it will be useful,
*   but WITHOUT ANY WARRANTY; without even the implied warranty of
*   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
*   GNU General Public License for more details.
*
*   You should have received a copy of the GNU General Public License
*   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::{fmt::Display, sync::LazyLock};

use axum::{extract::{FromRequestParts, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, RequestPartsExt, Router};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;

pub fn auth_router(pool: sqlx::MySqlPool) -> Router{
    Router::new()
        .route("/authorize", post(authorize))
        .route("/protected", get(protected))
        .with_state(pool)
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    id: Option<i32>,
    name: Option<String>,
    password: String,
}

#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    MissingToken
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token")
        };
        let body = Json(json!({
            "error": error_message
        }));
        (status, body).into_response()
    }
}

struct Keys {
    encoding: jsonwebtoken::EncodingKey,
    decoding: jsonwebtoken::DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: jsonwebtoken::EncodingKey::from_secret(secret),
            decoding: jsonwebtoken::DecodingKey::from_secret(secret),
        }
    }
}

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = "Free as in Freedom";
    Keys::new(secret.as_bytes())
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: i32,
    pub name: String,
    pub exp: i64,
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let offset = chrono::FixedOffset::east_opt(8 * 3600).ok_or(std::fmt::Error)?;
        let naive_datetime = chrono::DateTime::from_timestamp(self.exp, 0)
            .ok_or(std::fmt::Error)?.naive_utc();

        let expire = chrono::DateTime::<chrono::offset::FixedOffset>::from_naive_utc_and_offset(
                naive_datetime, offset
            )
            .to_string();

        write!(f, "ID: {}\nName: {}\nExpire: {}", self.id, self.name, expire)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection>  {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let validation = &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        
        let token_date = jsonwebtoken::decode::<Claims>(
            bearer.token(), &KEYS.decoding, &validation 
        ).map_err(|_| AuthError::InvalidToken)?;
        if token_date.claims.exp <= chrono::Utc::now().timestamp() {
            return Err(AuthError::InvalidToken);
        }
        Ok(token_date.claims)
    }
}

async fn authorize(State(pool): State<sqlx::MySqlPool>, Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>, AuthError> {

    let query = match payload {
        AuthPayload { password: password_hash, .. } if password_hash.is_empty() => {
            return Err(AuthError::MissingCredentials);
        }
        AuthPayload { id: Some(id), .. } => {
            sqlx::query("SELECT id, name, password_hash FROM user WHERE id=?")
                .bind(id)
        }
        AuthPayload { name: Some(name), .. } => {
            sqlx::query("SELECT id, name, password_hash FROM user WHERE name=?")
                .bind(name)
        }
        _ => { 
            return Err(AuthError::MissingToken);
        }
    };

    let row = query.fetch_one(&pool)
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    let id: i32 = row.get(0);
    let name: String = row.get(1);
    let password_hash: String = row.get(2);
    if !bcrypt::verify(payload.password, &password_hash).map_err(|_| AuthError::WrongCredentials)? {
        return Err(AuthError::WrongCredentials);
    }
    let exp = chrono::Utc::now().timestamp() + 3600;
    let claims = Claims {
        id,
        name,
        exp,
    };

    // Create the authorization token
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    return Ok(Json(AuthBody::new(token)));
}

async fn protected(claims: Claims) -> Result<String, AuthError> {
    // Send the protected data to the user
    Ok(format!(
        "Welcome to the protected area :)\nYour data:\n{claims}",
    ))
}