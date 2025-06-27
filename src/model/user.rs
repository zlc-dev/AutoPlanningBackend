/*
*   model::error
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

use axum::{extract::{Query, State}, http::StatusCode, routing::post, Json, Router};
use sqlx::{prelude::*, MySqlPool};
use serde::{Serialize, Deserialize};
use crate::util::error::internal_error;

// 用户数据库模型
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password_hash: String,
    pub created_at: sqlx::types::chrono::NaiveDateTime
}

pub fn user_router(pool: MySqlPool) -> Router {
    Router::new()
        .route("/", post(create_user).get(query_user))
        .with_state(pool)
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    password: String,
}

async fn create_user(
    State(pool): State<MySqlPool>, Json(payload): Json<CreateUserRequest>
) -> Result<String, (StatusCode, String)> {
    let password_hash = bcrypt::hash_with_salt(
            payload.password, 
            5, 
            [0; 16]
        )
        .map_err(internal_error)?
        .to_string();

    sqlx::query("INSERT INTO user (name, password_hash) VALUES (?,?)")
        .bind(payload.name)
        .bind(password_hash)
        .execute(&pool)
        .await
        .map(|_| "ok".to_string())
        .map_err(internal_error)
}

#[derive(Debug, Deserialize)]
struct QueryUserParams {
    id: Option<String>,
    name: Option<String>,
}

async fn query_user(
    State(pool): State<MySqlPool>, Query(params): Query<QueryUserParams>
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let query;
    match params {
        QueryUserParams { id: Some(id), name: Some(name) } => {
            query = sqlx::query("SELECT id, name, password_hash, created_at FROM user WHERE id=? AND name=?")
                .bind(id)
                .bind(name)
        }
        QueryUserParams { id: Some(id), name: None } => {
            query = sqlx::query("SELECT id, name, password_hash, created_at FROM user WHERE id=?")
                .bind(id)
        }
        QueryUserParams { id: None, name: Some(name) } => {
            query = sqlx::query("SELECT id, name, password_hash, created_at FROM user WHERE name=?")
                .bind(name)
        }
        _ => return Ok(Json(vec![]))
    }
    let users = query.fetch_all(&pool).await.map_err(internal_error)?;
    Ok(
        Json(users.iter().map(|row| {
            User {
                id: row.get("id"),
                name: row.get("name"),
                password_hash: row.get("password_hash"),
                created_at: row.get("created_at"),
            }
        }).collect())
    )
}
