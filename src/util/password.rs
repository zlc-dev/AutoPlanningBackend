/*
*   util::password
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

use serde::{de::Visitor, Deserialize, Serialize};

pub trait PasswordProperties {}

pub trait PasswordWithSalt: PasswordProperties {
    const COST: u32;
    const SALT: [u8; 16];
}

pub trait PasswordWithRandomSalt: PasswordProperties {
    const COST: u32;
}

#[derive(Debug, Clone)]
pub struct StringPassword<P: PasswordProperties> {
    pub value: String,
    _mark: PhantomData<P>,
}

impl<P: PasswordProperties> StringPassword<P> {
    pub fn new(value: String) -> Self {
        Self { value, _mark: PhantomData }
    }
}

impl<P: PasswordProperties> Serialize for StringPassword<P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(&self.value)
    }
}

impl<'de, P: PasswordProperties> Deserialize<'de> for StringPassword<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        struct UserPasswordVisitor<P: PasswordProperties>(PhantomData<P>);
        impl<'de, P: PasswordProperties> Visitor<'de> for UserPasswordVisitor<P> {
            type Value = StringPassword<P>;
        
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(StringPassword::new(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error, {
                Ok(StringPassword::new(v))
            }
        }
        deserializer.deserialize_string(UserPasswordVisitor::<P>(PhantomData))
    }
}

impl<P: PasswordWithSalt> StringPassword<P> {
    pub fn hash_with_salt(&self) -> Result<String, bcrypt::BcryptError> {
        bcrypt::hash_with_salt(&self.value, P::COST, P::SALT).map(|parts| {
            parts.to_string()
        })
    }
}

impl<P: PasswordWithRandomSalt> StringPassword<P> {
    pub fn hash_with_random_salt(&self) -> Result<String, bcrypt::BcryptError> {
        let mut salt = [0u8; 16];
        getrandom::fill(&mut salt).map_err(|err| bcrypt::BcryptError::Rand(err))?;
        bcrypt::hash_with_salt(&self.value, P::COST, salt).map(|parts| {
            parts.to_string()
        })
    }
}

