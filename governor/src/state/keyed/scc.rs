#![cfg(all(feature = "std", feature = "dashmap"))]

use std::prelude::v1::*;

use crate::nanos::Nanos;
use crate::state::{InMemoryState, StateStore};
use crate::{clock, Quota, RateLimiter};
use crate::{middleware::NoOpMiddleware, state::keyed::ShrinkableKeyedStateStore};
use std::hash::Hash;

use scc::HashMap as SccHashMap;

/// A concurrent, thread-safe and performant hashmap based on [`scc::HashMap`]
pub type SccHashMapStateStore<K> = SccHashMap<K, InMemoryState, super::HashBuilder>;

impl<K: Hash + Eq + Clone> StateStore for SccHashMapStateStore<K> {
    type Key = K;

    fn measure_and_replace<T, F, E>(&self, key: &Self::Key, mut f: F) -> Result<T, E>
    where
        F: Fn(Option<Nanos>) -> Result<(T, Nanos), E>,
    {
        if let Some(r) = self.read(key, |_, v| v.measure_and_replace_one(&mut f)) {
            return r;
        }

        self.entry(key.clone())
            .or_default()
            .get()
            .measure_and_replace_one(f)
    }
}

impl<K, C> RateLimiter<K, SccHashMapStateStore<K>, C, NoOpMiddleware<C::Instant>>
where
    K: Hash + Eq + Clone,
    C: clock::Clock,
{
    pub fn scchashmap_with_clock(quota: Quota, clock: &C) -> Self {
        let state: SccHashMapStateStore<K> = SccHashMap::default();
        RateLimiter::new(quota, state, clock)
    }
}

impl<K> ShrinkableKeyedStateStore<K> for SccHashMapStateStore<K>
where
    K: Hash + Eq + Clone,
{
    fn retain_recent(&self, drop_below: Nanos) {
        self.retain(|_, v| !v.is_older_than(drop_below));
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
