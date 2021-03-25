use std::cmp::Ordering;

use jwt_simple::prelude::{Duration, JWTClaims, UnixTimeStamp};

use super::Jwt;


#[derive(Debug)]
pub struct JwtData {
    pub expires: UnixTimeStamp,
    pub token: String
}

impl JwtData {

    pub fn new (expires: UnixTimeStamp, token: String) -> JwtData {
        JwtData {
            expires,
            token
        }
    }

    pub fn new_from_claims<'a> (claims: JWTClaims<impl Jwt>, token: String) -> JwtData {
        Self::new (
            claims.expires_at.unwrap_or(Duration::from_hours(2)),
            token
        )
    }
    
}

impl Ord for JwtData {

    fn cmp(&self, other: &Self) -> Ordering {
        self.expires.cmp(&other.expires)
    }

}

impl Eq for JwtData {

}

impl PartialOrd for JwtData {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

}

impl PartialEq for JwtData {

    fn eq(&self, other: &Self) -> bool {
        self.expires == other.expires
    }
}
