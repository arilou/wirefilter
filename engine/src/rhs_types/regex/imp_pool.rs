use hashbrown::HashSet;
use lazy_static::lazy_static;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub use regex::Error;

/// Wrapper around [`regex::bytes::Regex`]
#[derive(Clone)]
pub struct Regex(Arc<regex::bytes::Regex>);

lazy_static! {
    static ref REGEX_POOL: Mutex<HashSet<Regex>> = Mutex::new(HashSet::new());
}

impl Drop for Regex {
    fn drop(&mut self) {
        // The logic here is a bit hacky, we check the strong_count for 2,
        // because we have this reference and the one that lives in the HashSet.
        // Once we will call remove on the pool, we will enter this function again
        // this time with the entry that came from the HashSet, it too has a strong_count
        // of 2, because we have not finished dropping the previous Arc.
        // In order to distinguish between the two, we are doing a small hack here and
        // take a weak reference prior to calling remove, this way the 2nd drop can know
        // it's the one that came from the pool and does not need to do any additional
        // work.
        // This is how we solve the deadlock of the mutex being acquired more than once.
        if Arc::strong_count(&self.0) == 2 && Arc::weak_count(&self.0) == 0 {
            let _dummy_weak = Arc::downgrade(&self.0);
            let mut pool = REGEX_POOL.lock().unwrap();
            pool.remove(&self);
            return;
        }
    }
}

impl FromStr for Regex {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let regex = Self(Arc::new(
            ::regex::bytes::RegexBuilder::new(s)
                .unicode(false)
                .build()?,
        ));

        let mut pool = REGEX_POOL.lock().unwrap();
        Ok(pool.get_or_insert(regex).clone())
    }
}

impl Regex {
    /// Returns true if and only if the regex matches the string given.
    pub fn is_match(&self, text: &[u8]) -> bool {
        self.0.is_match(text)
    }

    /// Returns the original string of this regex.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
