pub use derive_entity::Entity;

#[derive(Debug)]
pub struct EntityModel {
    pub entity: &'static str,
    pub columns: Vec<&'static str>,
    pub tys: Vec<&'static str>,
}

impl Default for EntityModel {
    fn default() -> Self {
        EntityModel {
            entity: "",
            columns: vec![],
            tys: vec![],
        }
    }
}
