use crate::Fingerprint;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};

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

/// The values are hard coded in [`static@OUR_STUDENTS`] so `MetuId::new`() can accept/reject values based on that
/// TODO update the statement above
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MetuId {
    id: String,
    passwd: String,
}

impl fmt::Display for MetuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl MetuId {
    pub fn new(id: String, pwd: String) -> Option<Self> {
        if OUR_STUDENTS.contains(&(&*id, &*pwd)) {
            Some(MetuId { id, passwd: pwd })
        } else {
            None
        }
    }

    // TODO: replace the function above with this <15-04-22, yigit> //
    pub fn _new(id: String, passwd: String) -> Self {
        MetuId  { id, passwd }
    }
}

// TODO: remove this, read from a yaml or something, then MetuId::new gets a self <11-04-22, yigit> //
// Students who are authorized to have Gradecoin accounts
lazy_static! {
    static ref OUR_STUDENTS: HashSet<(&'static str, &'static str)> = {
        [
            ("e254275", "DtNX1qk4YF4saRH"),
            ("e223687", "cvFEs4XLjuGBD1v"),
            ("e211024", "voQAcxiKJmEXYRT"),
            ("e209888", "O75dli6AQtz2tUi"),
            ("e223725", "xXuTD3Y4tyrv2Jz"),
            ("e209362", "N7wGm5XU5zVWOWu"),
            ("e209898", "aKBFfB8fZMq8pVn"),
            ("e230995", "TgcHGlqeFhQGx42"),
            ("e223743", "YVWVSWuIHplJk9C"),
            ("e223747", "8LAeHrsjnwXh59Q"),
            ("e223749", "HMFeJqVOzwCPHbc"),
            ("e223751", "NjMsxmtmy2VOwMW"),
            ("e188126", "QibuPdV2gXfsVJW"),
            ("e209913", "kMxJvl2vHSWCy4A"),
            ("e203608", "mfkkR0MWurk6Rp1"),
            ("e233013", "GCqHxdOaDj2pWXx"),
            ("e216982", "2Z0xmgCStnj5qg5"),
            ("e217185", "BcaZNlzlhPph7A3"),
            ("e223780", "2KvVxKUQaA9H4sn"),
            ("e194931", "hsC0Wb8PQ5vzwdQ"),
            ("e223783", "ETUJA3kt1QYvJai"),
            ("e254550", "rPRjX0A4NefvKWi"),
            ("e217203", "lN3IWhGyCrGfkk5"),
            ("e217477", "O9xlMaa7LanC82w"),
            ("e223786", "UxI6czykJfp9T9N"),
            ("e231060", "VJgziofQQPCoisH"),
            ("e223795", "pmcTCKox99NFsqp"),
            ("e223715", "1H5QuOYI1b2r9ET"),
            ("e181932", "THANKYOUHAVEFUN"),
            ("bank", "P7oxDm30g1jeIId"),
            ("friend_1", "not_used"),
            ("friend_2", "not_used"),
            ("friend_3", "not_used"),
            ("friend_4", "not_used"),
        ]
        .iter()
        .copied()
        .collect()
    };
}
