/*
*   main
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

use std::{ops::Deref, sync::LazyLock};

use axum::Router;
use sqlx::MySqlPool;

mod database;
use database::prelude::*;
mod model;
use model::user::user_router;

use crate::{server::auth::auth_router, util::keys};
mod util;
mod server;

const DATABASE_URL: DataBaseUrl<'_, mark::MariaDB> = DataBaseUrl::<'_, mark::MariaDB>::new(
    DataBaseConfig {
        user: "apb", 
        // todo: 不应把密码写在代码中
        password: "1145141919810", 
        host: "localhost",  
        port: 3306, 
        database: "apb_database"
    }
);

static KEYS: LazyLock<keys::Keys> = LazyLock::new(|| {
    // todo: 不应把密码写在代码中
    let secret = "Free as in Freedom";
    keys::Keys::new(secret.as_bytes())
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // initialize tracing
    tracing_subscriber::fmt::init();

    let pool = MySqlPool::connect(&DATABASE_URL.get_url()).await?;
    
    let app = Router::new()
        .nest("/users", user_router())
        .with_state(pool.clone())
        .nest("/auth", auth_router())
        .with_state((pool, KEYS.deref()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
