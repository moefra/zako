use std::sync::Arc;

use crate::{FastMap, FastSet, node::NodeKey};

#[derive(Debug, Clone)]
pub struct DependencyGraph<C, K: NodeKey<C>> {
    parents: FastMap<K, FastSet<K>>,
    children: FastMap<K, FastSet<K>>,
    _marker: std::marker::PhantomData<C>,
}

impl<C, K: NodeKey<C>> DependencyGraph<C, K> {
    pub fn new() -> Self {
        Self {
            parents: FastMap::default(),
            children: FastMap::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn add_parent(&self, this: K, parent: K) {
        let set = self.parents.entry(this.clone()).or_default();
        set.insert(parent.clone());
        self.children.entry(parent).or_default().insert(this);
    }

    pub fn add_child(&self, this: K, child: K) {
        let set = self.children.entry(this.clone()).or_default();
        set.insert(child.clone());
        self.parents.entry(child).or_default().insert(this);
    }

    pub fn add_parents(&self, this: K, parent: impl Iterator<Item = K>) {
        // TODO: optimize by locking once
        for p in parent {
            self.add_parent(this.clone(), p);
        }
    }

    pub fn add_children(&self, this: K, child: impl Iterator<Item = K>) {
        // TODO: optimize by locking once
        for c in child {
            self.add_child(this.clone(), c);
        }
    }

    pub fn clear_children_dependency_of(&self, this: K) {
        if let Some(children) = self.children.remove(&this) {
            for child in children.1.iter() {
                if let Some(parents) = self.parents.get_mut(&*child) {
                    parents.remove(&this);
                }
            }
        }
        self.children.get(&this).map(|children| children.clear());
    }

    pub fn get_parents(&self, key: K) -> dashmap::Entry<'_, K, FastSet<K>> {
        self.parents.entry(key)
    }

    pub fn get_children(&self, key: K) -> dashmap::Entry<'_, K, FastSet<K>> {
        self.children.entry(key)
    }
}
