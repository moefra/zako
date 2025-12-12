use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::id::{InternedString, Interner};

#[derive(TS, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[ts(export, export_to = "author.d.ts", as = "AuthorTS")]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InternedAuthor(pub InternedString);

impl Author {
    pub fn intern(&self, interner: &mut Interner) -> InternedAuthor {
        InternedAuthor(interner.get_or_intern(format!("{} <{}>", self.name, self.email).as_str()))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthorParseError {
    #[error("Invalid author format")]
    InvalidFormat,
}

impl FromStr for Author {
    type Err = AuthorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('<').collect();
        if parts.len() != 2 {
            return Err(AuthorParseError::InvalidFormat);
        }
        let name = parts[0].trim().to_string();
        let email_part = parts[1].trim().trim_end_matches('>').trim();
        Ok(Author {
            name,
            email: email_part.to_string(),
        })
    }
}

struct AuthorTS;
impl TS for AuthorTS {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;

    fn decl() -> String {
        "type Author = `${string} <${string}@${string}>`".into()
    }
    fn decl_concrete() -> String {
        Self::decl()
    }
    fn name() -> String {
        "Author".into()
    }
    fn inline() -> String {
        "`${string} <${string}@${string}>`".into()
    }
    fn inline_flattened() -> String {
        "`${string} <${string}@${string}>`".into()
    }
}
