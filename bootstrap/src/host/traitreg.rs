use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub enum TraitRegError {
    TypeExists { type_id: TypeId },
    TypeNotFound { type_id: TypeId },
    TraitNotFound { trait_id: TraitId },
    AlreadyImplements { type_id: TypeId, trait_id: TraitId },
    NotImplemented { type_id: TypeId, trait_id: TraitId },
    NameConflict { name: String },
}

impl std::fmt::Display for TraitRegError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraitRegError::TypeExists { type_id } => write!(f, "type {:?} exists", type_id),
            TraitRegError::TypeNotFound { type_id } => write!(f, "type {:?} not found", type_id),
            TraitRegError::TraitNotFound { trait_id } => write!(f, "trait {:?} not found", trait_id),
            TraitRegError::AlreadyImplements { type_id, trait_id } =>
                write!(f, "type {:?} already impls {:?}", type_id, trait_id),
            TraitRegError::NotImplemented { type_id, trait_id } =>
                write!(f, "type {:?} doesn't impl {:?}", type_id, trait_id),
            TraitRegError::NameConflict { name } => write!(f, "name {name} conflicts"),
        }
    }
}

impl std::error::Error for TraitRegError {}

#[derive(Debug, Clone)]
struct TypeEntry {
    id: TypeId,
    name: String,
    traits: BTreeSet<TraitId>,
}

#[derive(Debug, Clone)]
struct TraitEntry {
    id: TraitId,
    name: String,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub id: TypeId,
    pub name: String,
    pub traits: Vec<TraitId>,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub id: TraitId,
    pub name: String,
    pub implementors: Vec<TypeId>,
}

#[derive(Debug, Clone)]
pub struct TraitRegistry {
    types: BTreeMap<u64, TypeEntry>,
    traits: BTreeMap<u64, TraitEntry>,
    name_to_type: BTreeMap<String, u64>,
    name_to_trait: BTreeMap<String, u64>,
}

impl TraitRegistry {
    pub fn new() -> Self {
        Self { types: BTreeMap::new(), traits: BTreeMap::new(), name_to_type: BTreeMap::new(), name_to_trait: BTreeMap::new() }
    }

    pub fn register_type(&mut self, id: TypeId, name: &str) -> Result<(), TraitRegError> {
        if self.types.contains_key(&id.0) { return Err(TraitRegError::TypeExists { type_id: id }); }
        if self.name_to_type.contains_key(name) { return Err(TraitRegError::NameConflict { name: name.to_string() }); }
        self.types.insert(id.0, TypeEntry { id, name: name.to_string(), traits: BTreeSet::new() });
        self.name_to_type.insert(name.to_string(), id.0);
        Ok(())
    }

    pub fn register_trait(&mut self, id: TraitId, name: &str) -> Result<(), TraitRegError> {
        if self.name_to_trait.contains_key(name) { return Err(TraitRegError::NameConflict { name: name.to_string() }); }
        self.traits.insert(id.0, TraitEntry { id, name: name.to_string() });
        self.name_to_trait.insert(name.to_string(), id.0);
        Ok(())
    }

    pub fn implement(&mut self, type_id: TypeId, trait_id: TraitId) -> Result<(), TraitRegError> {
        if !self.types.contains_key(&type_id.0) { return Err(TraitRegError::TypeNotFound { type_id }); }
        if !self.traits.contains_key(&trait_id.0) { return Err(TraitRegError::TraitNotFound { trait_id }); }
        let te = self.types.get_mut(&type_id.0).unwrap();
        if te.traits.contains(&trait_id) { return Err(TraitRegError::AlreadyImplements { type_id, trait_id }); }
        te.traits.insert(trait_id);
        Ok(())
    }

    pub fn query_by_trait(&self, trait_id: TraitId) -> Vec<TypeId> {
        self.types.values()
            .filter(|t| t.traits.contains(&trait_id))
            .map(|t| t.id)
            .collect()
    }

    pub fn query_types_impl_all(&self, trait_ids: &[TraitId]) -> Vec<TypeId> {
        self.types.values()
            .filter(|t| trait_ids.iter().all(|tid| t.traits.contains(tid)))
            .map(|t| t.id)
            .collect()
    }

    pub fn implements(&self, type_id: TypeId, trait_id: TraitId) -> bool {
        self.types.get(&type_id.0).map(|t| t.traits.contains(&trait_id)).unwrap_or(false)
    }

    pub fn type_info(&self, type_id: TypeId) -> Option<TypeInfo> {
        self.types.get(&type_id.0).map(|t| TypeInfo {
            id: t.id, name: t.name.clone(), traits: t.traits.iter().copied().collect(),
        })
    }

    pub fn trait_info(&self, trait_id: TraitId) -> Option<TraitInfo> {
        self.traits.get(&trait_id.0).map(|te| TraitInfo {
            id: te.id, name: te.name.clone(),
            implementors: self.types.values().filter(|t| t.traits.contains(&trait_id)).map(|t| t.id).collect(),
        })
    }

    pub fn type_count(&self) -> usize { self.types.len() }
    pub fn trait_count(&self) -> usize { self.traits.len() }

    pub fn lookup_type(&self, name: &str) -> Option<TypeId> {
        self.name_to_type.get(name).map(|&id| TypeId(id))
    }

    pub fn lookup_trait(&self, name: &str) -> Option<TraitId> {
        self.name_to_trait.get(name).map(|&id| TraitId(id))
    }
}

impl Default for TraitRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry() {
        let tr = TraitRegistry::new();
        assert_eq!(tr.type_count(), 0);
        assert_eq!(tr.trait_count(), 0);
    }

    #[test]
    fn register_type_and_trait() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "Foo").unwrap();
        tr.register_trait(TraitId(10), "Bar").unwrap();
        assert_eq!(tr.type_count(), 1);
        assert_eq!(tr.trait_count(), 1);
    }

    #[test]
    fn duplicate_type() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "Foo").unwrap();
        let err = tr.register_type(TypeId(1), "Bar").unwrap_err();
        assert!(matches!(err, TraitRegError::TypeExists { .. }));
    }

    #[test]
    fn name_conflict() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "Foo").unwrap();
        let err = tr.register_type(TypeId(2), "Foo").unwrap_err();
        assert!(matches!(err, TraitRegError::NameConflict { .. }));
    }

    #[test]
    fn implement_trait() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "Foo").unwrap();
        tr.register_trait(TraitId(10), "Bar").unwrap();
        tr.implement(TypeId(1), TraitId(10)).unwrap();
        assert!(tr.implements(TypeId(1), TraitId(10)));
    }

    #[test]
    fn double_impl() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "Foo").unwrap();
        tr.register_trait(TraitId(10), "Bar").unwrap();
        tr.implement(TypeId(1), TraitId(10)).unwrap();
        let err = tr.implement(TypeId(1), TraitId(10)).unwrap_err();
        assert!(matches!(err, TraitRegError::AlreadyImplements { .. }));
    }

    #[test]
    fn query_by_trait() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "A").unwrap();
        tr.register_type(TypeId(2), "B").unwrap();
        tr.register_type(TypeId(3), "C").unwrap();
        tr.register_trait(TraitId(10), "Print").unwrap();
        tr.implement(TypeId(1), TraitId(10)).unwrap();
        tr.implement(TypeId(3), TraitId(10)).unwrap();
        let result = tr.query_by_trait(TraitId(10));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn query_impl_all() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "A").unwrap();
        tr.register_trait(TraitId(10), "X").unwrap();
        tr.register_trait(TraitId(11), "Y").unwrap();
        tr.implement(TypeId(1), TraitId(10)).unwrap();
        tr.implement(TypeId(1), TraitId(11)).unwrap();
        let result = tr.query_types_impl_all(&[TraitId(10), TraitId(11)]);
        assert_eq!(result, vec![TypeId(1)]);
    }

    #[test]
    fn lookup() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "Foo").unwrap();
        tr.register_trait(TraitId(10), "Bar").unwrap();
        assert_eq!(tr.lookup_type("Foo"), Some(TypeId(1)));
        assert_eq!(tr.lookup_trait("Bar"), Some(TraitId(10)));
    }

    #[test]
    fn info() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "A").unwrap();
        tr.register_trait(TraitId(10), "X").unwrap();
        tr.implement(TypeId(1), TraitId(10)).unwrap();
        let ti = tr.type_info(TypeId(1)).unwrap();
        assert_eq!(ti.name, "A");
        assert_eq!(ti.traits, vec![TraitId(10)]);
    }

    #[test]
    fn trait_info() {
        let mut tr = TraitRegistry::new();
        tr.register_type(TypeId(1), "A").unwrap();
        tr.register_trait(TraitId(10), "X").unwrap();
        tr.implement(TypeId(1), TraitId(10)).unwrap();
        let ti = tr.trait_info(TraitId(10)).unwrap();
        assert_eq!(ti.implementors, vec![TypeId(1)]);
    }

    #[test]
    fn error_display() {
        assert!(TraitRegError::TypeNotFound { type_id: TypeId(3) }.to_string().contains("3"));
    }
}
