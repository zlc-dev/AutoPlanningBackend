/*
*   util::key
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

pub trait AuthKeys {
    fn get_encoding(&self) -> &jsonwebtoken::EncodingKey;
    fn get_decoding(&self) -> &jsonwebtoken::DecodingKey;
}


pub struct Keys {
    encoding: jsonwebtoken::EncodingKey,
    decoding: jsonwebtoken::DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: jsonwebtoken::EncodingKey::from_secret(secret),
            decoding: jsonwebtoken::DecodingKey::from_secret(secret),
        }
    }
}

impl AuthKeys for Keys {
    fn get_encoding(&self) -> &jsonwebtoken::EncodingKey {
        &self.encoding
    }

    fn get_decoding(&self) -> &jsonwebtoken::DecodingKey {
        &self.decoding
    }
}
