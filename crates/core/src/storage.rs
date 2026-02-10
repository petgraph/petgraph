use alloc::vec::Vec;
use core::hash::Hash;
use std::collections::HashMap;

use crate::{graph::Graph, id::IndexId};

// pub fn get_node_storage<G, Value>(mut graph: &G) -> impl Storage<G::NodeId, Value>
// where
//     G: Graph,
//     for<'a, 'b, 'c> &'a mut &'b mut &'c G: Storable + Graph<NodeId = G::NodeId>,
//     G::NodeId: Hash + Eq,
//     Value: Default,
// {
//     let binding = &mut &mut graph;
//     binding.get_node_storage::<Value>()
// }

pub trait Storable: Graph {
    type NodeStorage<Value>: Storage<<Self as Graph>::NodeId, Value>;
    type EdgeStorage<Value>: Storage<<Self as Graph>::EdgeId, Value>;

    fn get_node_storage<Value: Default>(&self) -> Self::NodeStorage<Value>;
    fn get_edge_storage<Value: Default>(&self) -> Self::EdgeStorage<Value>;
}

impl<G> Storable for &mut &G
where
    G: Graph,
    G::NodeId: Hash + Eq + Default,
    G::EdgeId: Hash + Eq + Default,
{
    type EdgeStorage<Value> = HashMap<<Self as Graph>::EdgeId, Value>;
    type NodeStorage<Value> = HashMap<<Self as Graph>::NodeId, Value>;

    fn get_node_storage<Value: Default>(&self) -> Self::NodeStorage<Value> {
        HashMap::new()
    }

    fn get_edge_storage<Value: Default>(&self) -> Self::EdgeStorage<Value> {
        HashMap::new()
    }
}

impl<G> Storable for &mut &mut &G
where
    G: Graph,
    G::NodeId: Hash + Eq,
    G::EdgeId: Hash + Eq,
    G::NodeId: IndexId,
    G::EdgeId: IndexId,
{
    type EdgeStorage<Value> = Vec<Value>;
    type NodeStorage<Value> = Vec<Value>;

    fn get_node_storage<Value>(&self) -> Self::NodeStorage<Value> {
        Vec::new()
    }

    fn get_edge_storage<Value>(&self) -> Self::EdgeStorage<Value> {
        Vec::new()
    }
}

// impl<G> Storable for &mut &mut &mut &G
// where
//     G: Graph,
//     G::NodeId: Hash + Eq,
//     G::EdgeId: Hash + Eq,
//     G::NodeId: IndexId,
//     G::EdgeId: IndexId,
//     G::NodeId: GaplessIndexId,
//     G::EdgeId: GaplessIndexId,
// {
//     fn get_node_storage<Value>(&self) -> impl Storage<<Self as Graph>::NodeId, Value> {
//         println!("Using Vec Indexable for node storage");
//         Vec::new()
//     }

//     fn get_edge_storage<Value>(&self) -> impl Storage<<Self as Graph>::EdgeId, Value> {
//         println!("Using Vec Indexable for edge storage");
//         Vec::new()
//     }
// }

pub trait Storage<Key, Value> {
    fn get(&self, key: Key) -> Option<&Value>;
    fn set(&mut self, key: Key, value: Value);
}

impl<Key, Value> Storage<Key, Value> for HashMap<Key, Value>
where
    Key: Hash + Eq,
{
    fn get(&self, key: Key) -> Option<&Value> {
        self.get(&key)
    }

    fn set(&mut self, key: Key, value: Value) {
        self.insert(key, value);
    }
}

impl<Key, Value> Storage<Key, Value> for Vec<Value>
where
    Key: IndexId,
{
    fn get(&self, key: Key) -> Option<&Value> {
        self.as_slice().get::<usize>(key.as_usize())
    }

    fn set(&mut self, key: Key, value: Value) {
        // We guarantee that the key is always within the valid range of the graph, so we can safely
        // unwrap here.
        let index = key.as_usize();
        *self.get_mut(index).unwrap() = value;
    }
}

#[cfg(test)]
#[cfg(feature = "alloc")]
#[cfg(feature = "utils")]
mod tests {
    use core::{fmt, fmt::Display};

    use super::*;
    use crate::id::{Id, IndexIdTryFromIntError};

    /// Helper function to get the type name of a value, used for testing which storage
    /// implementation is being used.
    ///
    /// This is a bit of a hack, since the docs of `std::any::type_name` explicitly state that the
    /// output is not guaranteed to be stable: "This is intended for diagnostic use. The exact
    /// contents and format of the string returned are not specified, other than being a
    /// best-effort description of the type."
    fn get_type_name<T>(_val: &T) -> &'static str {
        std::any::type_name::<T>()
    }

    /// Struct to test the base case of the specialization, where the graph only allows for storing
    /// data in a HashMap.
    struct TestGraphBaseCase;
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Ord, PartialOrd)]
    struct NodeIdBaseCase(usize);
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Ord, PartialOrd)]
    struct EdgeIdBaseCase(usize);

    impl Display for NodeIdBaseCase {
        fn fmt(&self, _fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            todo!()
        }
    }

    impl Display for EdgeIdBaseCase {
        fn fmt(&self, _fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            todo!()
        }
    }

    impl Id for NodeIdBaseCase {}
    impl Id for EdgeIdBaseCase {}

    impl Graph for TestGraphBaseCase {
        type EdgeData<'graph>
            = usize
        where
            Self: 'graph;
        type EdgeDataMut<'graph>
            = usize
        where
            Self: 'graph;
        type EdgeDataRef<'graph>
            = usize
        where
            Self: 'graph;
        type EdgeId = EdgeIdBaseCase;
        type NodeData<'graph>
            = usize
        where
            Self: 'graph;
        type NodeDataMut<'graph>
            = usize
        where
            Self: 'graph;
        type NodeDataRef<'graph>
            = usize
        where
            Self: 'graph;
        type NodeId = NodeIdBaseCase;
    }

    #[test]
    fn test_storable_specialization_base_case() {
        let graph = TestGraphBaseCase {};
        let binding = &mut &mut &graph;
        let node_storage = binding.get_node_storage::<usize>();
        let type_name = get_type_name(&node_storage);
        assert!(type_name.contains("HashMap"));
    }

    /// Struct to test the case where the graph allows for storing data in a Vec.
    struct TestGraphIndexId;
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Ord, PartialOrd)]
    struct NodeIdIndexId(usize);
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Ord, PartialOrd)]
    struct EdgeIdIndexId(usize);

    impl Display for NodeIdIndexId {
        fn fmt(&self, _fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            todo!()
        }
    }

    impl Id for NodeIdIndexId {}

    impl TryFrom<u16> for NodeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: u16) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl TryFrom<u32> for NodeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: u32) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl TryFrom<u64> for NodeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: u64) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl TryFrom<usize> for NodeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: usize) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl IndexId for NodeIdIndexId {
        const MAX: Self = Self(usize::MAX);
        const MIN: Self = Self(0);

        fn as_u16(self) -> u16 {
            todo!()
        }

        fn as_u32(self) -> u32 {
            todo!()
        }

        fn as_u64(self) -> u64 {
            todo!()
        }

        fn as_usize(self) -> usize {
            todo!()
        }
    }

    impl Display for EdgeIdIndexId {
        fn fmt(&self, _fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            todo!()
        }
    }

    impl Id for EdgeIdIndexId {}

    impl TryFrom<u16> for EdgeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: u16) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl TryFrom<u32> for EdgeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: u32) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl TryFrom<u64> for EdgeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: u64) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl TryFrom<usize> for EdgeIdIndexId {
        type Error = IndexIdTryFromIntError;

        fn try_from(_value: usize) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl IndexId for EdgeIdIndexId {
        const MAX: Self = Self(usize::MAX);
        const MIN: Self = Self(0);

        fn as_u16(self) -> u16 {
            todo!()
        }

        fn as_u32(self) -> u32 {
            todo!()
        }

        fn as_u64(self) -> u64 {
            todo!()
        }

        fn as_usize(self) -> usize {
            todo!()
        }
    }

    impl Graph for TestGraphIndexId {
        type EdgeData<'graph>
            = usize
        where
            Self: 'graph;
        type EdgeDataMut<'graph>
            = usize
        where
            Self: 'graph;
        type EdgeDataRef<'graph>
            = usize
        where
            Self: 'graph;
        type EdgeId = EdgeIdIndexId;
        type NodeData<'graph>
            = usize
        where
            Self: 'graph;
        type NodeDataMut<'graph>
            = usize
        where
            Self: 'graph;
        type NodeDataRef<'graph>
            = usize
        where
            Self: 'graph;
        type NodeId = NodeIdIndexId;
    }

    #[test]
    fn test_storable_specialization_index_id() {
        let graph = TestGraphIndexId {};
        let binding = &mut &mut &mut &graph;
        let node_storage = binding.get_node_storage::<usize>();
        let type_name = get_type_name(&node_storage);
        assert!(type_name.contains("Vec"));
    }
}
