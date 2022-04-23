use crate::{Fingerprint, Id};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserAtRest {
    pub fingerprint: Fingerprint,
    pub user: User,
}

/// A Student
///
/// - [`user_id`]: Can only be one of the preapproved students (who are enlisted in the course)
/// - [`public_key`]: A PEM format public key "---- BEGIN" and all
/// - [`balance`]: User's current Gradecoin amount
///
/// This should ideally include the fingerprint as well?
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub user_id: MetuId,
    pub public_key: String,
    pub balance: u16,
    #[serde(skip, default = "bool::default")]
    pub is_bot: bool,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_id.get_id())
    }
}

impl fmt::Display for UserAtRest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MetuId {
    id: Id,
    passwd: String,
}

impl fmt::Display for MetuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl MetuId {
    pub fn new(id: String, passwd: String) -> Self {
        MetuId { id, passwd }
    }

    pub fn get_id(&self) -> &Id {
        &self.id
    }

    pub fn get_passwd(&self) -> &String {
        &self.passwd
    }
}
