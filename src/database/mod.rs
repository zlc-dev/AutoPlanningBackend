/*
*   database
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

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

pub trait DataBaseType {}

pub mod prelude {
    pub use super::{ 
        DataBaseConfig, 
        DataBaseUrl, 
        mark
    };
}

pub mod mark {
    #[derive(Debug, Default)]
    pub struct MariaDB;

    #[derive(Debug, Default)]
    pub struct MySql;

    impl super::DataBaseType for MariaDB {}
    impl super::DataBaseType for MySql {}
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DataBaseConfig<'a> {
    pub user: &'a str,
    pub password: &'a str,
    pub host: &'a str,
    pub port: usize,
    pub database: &'a str,
}

pub struct DataBaseUrl<'a, T: DataBaseType> {
    pub config: DataBaseConfig<'a>,
    _mark: PhantomData<T>,
}

impl<'a, T: DataBaseType> DataBaseUrl<'a, T> {
    pub const fn new(config: DataBaseConfig<'a>) -> Self {
        Self { config, _mark: PhantomData }
    }
}

impl<'a> DataBaseUrl<'a, mark::MariaDB> {
    pub fn get_url(&self) -> String {
        format!(
            "mariadb://{}:{}@{}:{}/{}", 
            self.config.user, 
            self.config.password, 
            self.config.host, 
            self.config.port, 
            self.config.database
        )
    }
}

impl<'a> DataBaseUrl<'a, mark::MySql> {
    pub fn get_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}", 
            self.config.user, 
            self.config.password, 
            self.config.host, 
            self.config.port, 
            self.config.database
        )
    }
}

