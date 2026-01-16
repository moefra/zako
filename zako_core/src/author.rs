use std::{cmp::Ordering, str::FromStr};

use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use zako_digest::blake3::Blake3Hash;

use crate::{
    context::BuildContext,
    intern::{Internable, InternedString, Interner, Uninternable},
};

/// The `Author` should be a string with format `Author Name <emabil@example.com>`
///
/// The space between Author name and `<` must not be omitted.The author name can not contains `<` or `>`.
///
/// We use [::email_address::EmailAddress::from_str] to check input email address.
#[derive(
    TS,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
#[ts(export, export_to = "author.d.ts", as = "AuthorTS")]
pub struct Author {
    name: String,
    email: email_address::EmailAddress,
}

impl Internable for Author {
    type Interned = InternedAuthor;

    fn intern(self, interner: &Interner) -> eyre::Result<Self::Interned> {
        let interner = interner.as_ref();
        Ok(InternedAuthor {
            name: interner
                .get_or_intern(self.name.as_str())
                .map_err(|err| AuthorError::InternerError(err, self.name.clone()))?,
            email: interner
                .get_or_intern(self.email.as_str())
                .map_err(|err| AuthorError::InternerError(err, self.email.to_string()))?,
        })
    }
}

impl PartialOrd for Author {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let first_cmp = self.name.cmp(&other.name);

        Some(if first_cmp == Ordering::Equal {
            self.email.as_str().cmp(other.email.as_str())
        } else {
            first_cmp
        })
    }
}

impl Ord for Author {
    fn cmp(&self, other: &Self) -> Ordering {
        let first_cmp = self.name.cmp(&other.name);

        if first_cmp == Ordering::Equal {
            self.email.as_str().cmp(other.email.as_str())
        } else {
            first_cmp
        }
    }
}

impl Blake3Hash for Author {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.name.hash_into_blake3(hasher);
        self.email.as_str().hash_into_blake3(hasher);
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Copy, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
)]
pub struct InternedAuthor {
    pub name: InternedString,
    pub email: InternedString,
}

#[derive(Error, Debug)]
pub enum AuthorError {
    #[error("The author name can not contains `<` or `>`,or the `<` and `>` not found")]
    AuthorNameError,
    #[error("The email format is invalid")]
    EmailFormatError(#[from] email_address::Error),
    #[error("Interner error while processing author `{1}`: {0}")]
    InternerError(#[source] ::zako_interner::InternerError, String),
}

impl Author {
    pub fn new(name: &str, email: &str) -> Result<Author, AuthorError> {
        if name.contains("<") || name.contains(">") {
            return Err(AuthorError::AuthorNameError);
        }

        Ok(Author {
            name: name.to_string(),
            email: EmailAddress::from_str(email)?,
        })
    }

    pub fn author(&self) -> &str {
        &self.name
    }

    pub fn email<'a>(&'a self) -> &'a str {
        self.email.as_str()
    }

    pub fn get_output_format(&self) -> String {
        format!("{} <{}>", self.name, self.email.as_str())
    }
}

impl Uninternable for InternedAuthor {
    type Uninterned = Author;

    fn unintern(&self, interner: &Interner) -> eyre::Result<Author> {
        let interner = interner.as_ref();
        Ok(Author {
            name: interner
                .resolve(&self.name)
                .map_err(|err| AuthorError::InternerError(err, "resolving name".to_string()))?
                .to_string(),
            email: EmailAddress::new_unchecked(
                interner
                    .resolve(&self.email)
                    .map_err(|err| AuthorError::InternerError(err, "resolving email".to_string()))?
                    .to_string(),
            ),
        })
    }
}

impl FromStr for Author {
    type Err = AuthorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('<').collect();
        if parts.len() != 2 {
            return Err(Self::Err::AuthorNameError);
        }
        let name = parts[0].trim();
        let email_part = parts[1].trim().trim_end_matches('>').trim();
        return Self::new(name, email_part);
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
