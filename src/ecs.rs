use std::collections::BTreeMap;

use crate::project::{Appearance, Project, SpriteConfig};
use crate::simulation::Vec2;

pub type EntityId = u64;

#[derive(Clone, Debug, PartialEq)]
pub struct EntityMetadata {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct EcsStorage {
    metadata: BTreeMap<EntityId, EntityMetadata>,
    positions: BTreeMap<EntityId, Vec2>,
    velocities: BTreeMap<EntityId, Vec2>,
    appearances: BTreeMap<EntityId, Appearance>,
    sprites: BTreeMap<EntityId, SpriteConfig>,
}

impl EcsStorage {
    pub fn from_project(project: &Project) -> Self {
        let mut storage = Self {
            metadata: BTreeMap::new(),
            positions: BTreeMap::new(),
            velocities: BTreeMap::new(),
            appearances: BTreeMap::new(),
            sprites: BTreeMap::new(),
        };
        for entity in &project.entities {
            storage.metadata.insert(
                entity.id,
                EntityMetadata {
                    name: entity.name.clone(),
                },
            );
            storage.positions.insert(
                entity.id,
                Vec2 {
                    x: entity.position.x,
                    y: entity.position.y,
                },
            );
            storage.velocities.insert(
                entity.id,
                Vec2 {
                    x: entity.velocity.x,
                    y: entity.velocity.y,
                },
            );
            storage
                .appearances
                .insert(entity.id, entity.appearance.clone());
            if let Some(sprite) = &entity.sprite {
                storage.sprites.insert(entity.id, sprite.clone());
            }
        }
        storage
    }

    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    pub fn ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.metadata.keys().copied()
    }

    pub fn metadata(&self, id: EntityId) -> Option<&EntityMetadata> {
        self.metadata.get(&id)
    }

    pub fn position(&self, id: EntityId) -> Option<&Vec2> {
        self.positions.get(&id)
    }

    pub fn position_mut(&mut self, id: EntityId) -> Option<&mut Vec2> {
        self.positions.get_mut(&id)
    }

    pub fn velocity(&self, id: EntityId) -> Option<&Vec2> {
        self.velocities.get(&id)
    }

    pub fn velocity_mut(&mut self, id: EntityId) -> Option<&mut Vec2> {
        self.velocities.get_mut(&id)
    }

    pub fn appearance(&self, id: EntityId) -> Option<&Appearance> {
        self.appearances.get(&id)
    }

    pub fn sprite(&self, id: EntityId) -> Option<&SpriteConfig> {
        self.sprites.get(&id)
    }

    pub fn move_all(&mut self) -> crate::Result<u64> {
        let velocities = &self.velocities;
        let mut visited = 0_u64;
        for (id, position) in &mut self.positions {
            let velocity = velocities
                .get(id)
                .ok_or_else(|| format!("vélocité absente pour l'entité {id}"))?;
            position.x = position
                .x
                .checked_add(velocity.x)
                .ok_or_else(|| format!("dépassement numérique sur l'entité {id}"))?;
            position.y = position
                .y
                .checked_add(velocity.y)
                .ok_or_else(|| format!("dépassement numérique sur l'entité {id}"))?;
            visited += 1;
        }
        Ok(visited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_order_does_not_change_canonical_storage() {
        let first: Project = toml::from_str(Project::example()).unwrap();
        let mut second = first.clone();
        second.entities.reverse();
        let first = EcsStorage::from_project(&first);
        let second = EcsStorage::from_project(&second);
        assert_eq!(first.ids().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(
            first.ids().collect::<Vec<_>>(),
            second.ids().collect::<Vec<_>>()
        );
        for id in first.ids() {
            assert_eq!(first.metadata(id), second.metadata(id));
            assert_eq!(first.position(id), second.position(id));
            assert_eq!(first.velocity(id), second.velocity(id));
            assert_eq!(first.appearance(id), second.appearance(id));
        }
    }
}
