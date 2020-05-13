use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use arc_swap::ArcSwap;
use crossbeam_utils::sync::ShardedLock;
use im::HashMap as ImmutableHashMap;
use parking_lot::Mutex;

/// A high-performance metric registry.
///
/// `Registry` provides the ability to maintain a central listing of metrics mapped by a given key.
///
/// In many cases, `K` will be a composite key, where the fundamental `Key` type from `metrics` is
/// present, and differentiation is provided by storing the metric type alongside.
///
/// Metrics themselves are represented opaquely behind `H`.  In most cases, this would be a
/// thread-safe handle to the underlying metrics storage that the owner of the registry can use to
/// update the actual metric value(s) as needed.  `Handle`, from this crate, is a solid default
/// choice.
///
/// `Registry` handles deduplicating metrics, and will return the identifier for an existing
/// metric if a caller attempts to reregister it.
///
/// `Registry` is optimized for reads.
pub struct Registry<K, H: 'static> {
    mappings: ArcSwap<ImmutableHashMap<K, (usize, &'static H)>>,
    handles: ShardedLock<Vec<(K, &'static H)>>,
    lock: Mutex<()>,
}

impl<K, H> Registry<K, H>
where
    K: Eq + Hash + Clone,
{
    /// Creates a new `Registry`.
    pub fn new() -> Self {
        Registry {
            mappings: ArcSwap::from(Arc::new(ImmutableHashMap::new())),
            handles: ShardedLock::new(Vec::new()),
            lock: Mutex::new(()),
        }
    }

    /// Get or create a handle for a given key.
    ///
    /// If the handle referenced by the given does not already exist, it will be created by calling
    /// `f` and using the result.  The handle will be stored such that it exists for the lifetime
    /// of the process.
    ///
    /// An identifier is given back which can be used later to reference the handle, as well as an
    /// immediate handle to the reference.
    pub fn get_or_create_handle<F>(&self, key: K, f: F) -> (usize, &'static H)
    where
        F: FnOnce(usize, &K) -> H,
    {
        // Check our mapping table first.
        if let Some(entry) = self.mappings.load().get(&key) {
            return *entry;
        }

        // Take control of the registry.
        let guard = self.lock.lock();

        // Check our mapping table again, in case someone just inserted what we need.
        let mappings = self.mappings.load();
        if let Some(entry) = mappings.get(&key) {
            return *entry;
        }

        // Our identifier will be the index we insert the handle into.
        let mut wg = self
            .handles
            .write()
            .expect("handles write lock was poisoned!");
        let id = wg.len();
        let handle = f(id, &key);
        let sh = &*Box::leak(Box::new(handle));

        wg.push((key.clone(), sh));
        drop(wg);

        // Update our mapping table and drop the lock.
        let new_mappings = mappings.update(key, (id, sh));
        drop(mappings);
        self.mappings.store(Arc::new(new_mappings));
        drop(guard);

        (id, sh)
    }

    /// Gets the handle for a given identifier.
    pub fn with_handle<F, V>(&self, id: usize, f: F) -> Option<V>
    where
        F: FnOnce(&K, &'static H) -> V,
    {
        let rg = self
            .handles
            .read()
            .expect("handles read lock was poisoned!");
        rg.get(id).map(|(k, h)| f(k, *h))
    }

    /// Creates a snapshot of all handles currently registered.
    pub fn get_handles(&self) -> HashMap<K, &'static H> {
        let guard = self.mappings.load();
        let mappings = ImmutableHashMap::clone(&guard);
        mappings.into_iter()
            .map(|(k, (_, h))| (k, h))
            .collect::<HashMap<_, _>>()
    }
}
