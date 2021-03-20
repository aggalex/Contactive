use std::{cmp::Ordering, sync::{Arc, Mutex, MutexGuard}, thread::{self, sleep}, time::{SystemTime, UNIX_EPOCH}};
use jwt_simple::prelude::{Duration, JWTClaims, UnixTimeStamp};
use sorted_vec::SortedVec;
use std::cmp::Ord;
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

    pub fn new_from_claims (claims: JWTClaims<Jwt>, token: String) -> JwtData {
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

#[derive(Clone)]
pub struct Blacklist(Arc<Mutex<SortedVec<JwtData>>>);

impl Blacklist {

    pub fn new () -> Blacklist {
        let mut bl = Blacklist(Arc::new(Mutex::new(SortedVec::new())));
        bl.start_gc ();
        bl
    }

    fn lock (&self) -> MutexGuard<SortedVec<JwtData>> {
        match (*self.0).lock () {
            Ok(guard) => guard,
            Err(poisoned) => {
                let mut guard = poisoned.into_inner ();
                let _ = std::mem::replace (&mut *guard, SortedVec::new ());
                guard
            }
        }
    }

    fn start_gc (&mut self) {
        let this = self.clone ();
        thread::spawn (move || {
            println!("Hello from garbage collector thread");

            loop {
                let duration = std::time::Duration::from_secs (5 * 60);
                sleep (duration);

                println!("GC CYCLE");

                {
                    let mut queue = this.lock ();

                    let mut to_delete: i128 = 0;
                    let now: UnixTimeStamp = Duration::from_secs(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect ("Time went backwards")
                            .as_secs()
                    );
                    
                    for auth in (&**queue).into_iter() {
                        if auth.expires < now {
                            to_delete += 1;
                        } else {
                            break
                        }
                    }

                    for _ in 0..to_delete {
                        queue.remove_index (0);
                    }
                }

                println!("{}", this);
            }
        });
    }

    pub fn blacklist (&self, jwt: JwtData) {
        self.lock ().insert (jwt);
    }

    pub fn contains (&self, token: &String) -> bool {
        self.lock ().iter ()
            .filter (|data| &data.token == token)
            .take(1)
            .collect::<Vec<&JwtData>> ()
            .len () > 0
    }

}

impl std::ops::Deref for Blacklist {

    type Target = Arc<Mutex<SortedVec<JwtData>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Blacklist {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Blacklist {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = (&**self.lock ())
            .into_iter ()
            .map (|data| format!("{:?}", data))
            .collect::<Vec<String>> ()
            .join (", ");
        write!(f, "Blacklist [ {} ]", string)
    }
}