use std::{sync::{Arc, Mutex, MutexGuard}, thread::{self, sleep}, time::{SystemTime, UNIX_EPOCH}};
use jwt_simple::prelude::{Duration, UnixTimeStamp};
use sorted_vec::SortedVec;
use crate::verification::Blacklist;
use super::jwt_data::JwtData;

#[derive(Clone)]
pub struct ThreadBlacklist(Arc<Mutex<SortedVec<JwtData>>>);

impl ThreadBlacklist {

    pub fn new () -> Self {
        let mut bl = Self(Arc::new(Mutex::new(SortedVec::new())));
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
            loop {
                let duration = std::time::Duration::from_secs (5 * 60);
                sleep (duration);
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
            }
        });
    }

}

impl Blacklist for ThreadBlacklist {

    type Data = JwtData;

    fn blacklist (&self, jwt: JwtData) {
        self.lock ().insert (jwt);
    }

    fn is_blacklisted (&self, token: &String) -> bool {
        self.lock ().iter ()
            .filter (|data| &data.token == token)
            .take(1)
            .collect::<Vec<&JwtData>> ()
            .len () > 0
    }

}

impl std::ops::Deref for ThreadBlacklist {

    type Target = Arc<Mutex<SortedVec<JwtData>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ThreadBlacklist {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for ThreadBlacklist {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = (&**self.lock ())
            .into_iter ()
            .map (|data| format!("{:?}", data))
            .collect::<Vec<String>> ()
            .join (", ");
        write!(f, "Blacklist [ {} ]", string)
    }

}