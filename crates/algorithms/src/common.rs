use fxhash::FxBuildHasher;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, FxBuildHasher>;
pub(crate) type IndexSet<K> = indexmap::IndexSet<K, FxBuildHasher>;
