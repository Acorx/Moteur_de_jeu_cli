use std::collections::{BTreeMap, BTreeSet};

use crate::project::{Appearance, Collider, Project, SpriteConfig};
use crate::simulation::Vec2;

pub type EntityId = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollisionStats {
    pub collisions_resolved: u64,
    pub entities_modified: u64,
}

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
    colliders: BTreeMap<EntityId, Collider>,
}

impl EcsStorage {
    pub fn from_project(project: &Project) -> Self {
        let mut storage = Self {
            metadata: BTreeMap::new(),
            positions: BTreeMap::new(),
            velocities: BTreeMap::new(),
            appearances: BTreeMap::new(),
            sprites: BTreeMap::new(),
            colliders: BTreeMap::new(),
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
            if let Some(collider) = &entity.collider {
                storage.colliders.insert(entity.id, collider.clone());
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

    pub fn collider(&self, id: EntityId) -> Option<&Collider> {
        self.colliders.get(&id)
    }

    pub fn collider_count(&self) -> usize {
        self.colliders.len()
    }

    pub fn move_all(&mut self) -> crate::Result<u64> {
        let colliders = &self.colliders;
        let velocities = &mut self.velocities;
        let mut visited = 0_u64;
        for (id, position) in &mut self.positions {
            if colliders.get(id).is_some_and(|collider| collider.is_static) {
                let velocity = velocities
                    .get_mut(id)
                    .ok_or_else(|| format!("vélocité absente pour l'entité {id}"))?;
                *velocity = Vec2 { x: 0, y: 0 };
                visited += 1;
                continue;
            }
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

    pub fn resolve_collisions(&mut self) -> crate::Result<CollisionStats> {
        let ids: Vec<_> = self.colliders.keys().copied().collect();
        let mut modified = BTreeSet::new();
        let mut collisions_resolved = 0_u64;

        for left_index in 0..ids.len() {
            for right_index in (left_index + 1)..ids.len() {
                let left_id = ids[left_index];
                let right_id = ids[right_index];
                let left_collider = self
                    .colliders
                    .get(&left_id)
                    .cloned()
                    .ok_or_else(|| format!("collider absent pour l'entité {left_id}"))?;
                let right_collider = self
                    .colliders
                    .get(&right_id)
                    .cloned()
                    .ok_or_else(|| format!("collider absent pour l'entité {right_id}"))?;
                let left_before = self
                    .position(left_id)
                    .copied()
                    .ok_or_else(|| format!("position absente pour l'entité {left_id}"))?;
                let right_before = self
                    .position(right_id)
                    .copied()
                    .ok_or_else(|| format!("position absente pour l'entité {right_id}"))?;
                let left_velocity_before = self
                    .velocity(left_id)
                    .copied()
                    .ok_or_else(|| format!("vélocité absente pour l'entité {left_id}"))?;
                let right_velocity_before = self
                    .velocity(right_id)
                    .copied()
                    .ok_or_else(|| format!("vélocité absente pour l'entité {right_id}"))?;

                let Some((axis, normal_sign, penetration)) =
                    collision_axis(left_before, &left_collider, right_before, &right_collider)
                else {
                    continue;
                };

                let contact = CollisionContact {
                    axis,
                    normal_sign,
                    penetration,
                    left_static: left_collider.is_static,
                    right_static: right_collider.is_static,
                };
                separate_pair(&mut self.positions, left_id, right_id, &contact)?;
                resolve_velocity_pair(
                    &mut self.velocities,
                    left_id,
                    right_id,
                    &contact,
                    &left_collider,
                    &right_collider,
                )?;
                collisions_resolved += 1;

                let left_after = self.position(left_id).copied().unwrap();
                let right_after = self.position(right_id).copied().unwrap();
                let left_velocity_after = self.velocity(left_id).copied().unwrap();
                let right_velocity_after = self.velocity(right_id).copied().unwrap();
                if left_before != left_after || left_velocity_before != left_velocity_after {
                    modified.insert(left_id);
                }
                if right_before != right_after || right_velocity_before != right_velocity_after {
                    modified.insert(right_id);
                }
            }
        }

        Ok(CollisionStats {
            collisions_resolved,
            entities_modified: u64::try_from(modified.len())
                .map_err(|_| "nombre d'entités modifiées hors limite")?,
        })
    }
}

#[derive(Clone, Copy)]
enum CollisionAxis {
    X,
    Y,
}

#[derive(Clone, Copy)]
struct CollisionContact {
    axis: CollisionAxis,
    normal_sign: i128,
    penetration: i128,
    left_static: bool,
    right_static: bool,
}

fn collision_axis(
    left: Vec2,
    left_collider: &Collider,
    right: Vec2,
    right_collider: &Collider,
) -> Option<(CollisionAxis, i128, i128)> {
    let delta_x = i128::from(right.x) - i128::from(left.x);
    let delta_y = i128::from(right.y) - i128::from(left.y);
    let overlap_x = i128::from(left_collider.half_width) + i128::from(right_collider.half_width)
        - delta_x.abs();
    let overlap_y = i128::from(left_collider.half_height) + i128::from(right_collider.half_height)
        - delta_y.abs();
    if overlap_x <= 0 || overlap_y <= 0 {
        return None;
    }
    if overlap_x <= overlap_y {
        Some((CollisionAxis::X, normal_sign(delta_x), overlap_x))
    } else {
        Some((CollisionAxis::Y, normal_sign(delta_y), overlap_y))
    }
}

fn normal_sign(delta: i128) -> i128 {
    if delta < 0 { -1 } else { 1 }
}

fn separate_pair(
    positions: &mut BTreeMap<EntityId, Vec2>,
    left_id: EntityId,
    right_id: EntityId,
    contact: &CollisionContact,
) -> crate::Result<()> {
    let left_shift = if contact.left_static {
        0
    } else if contact.right_static {
        contact.penetration
    } else {
        contact.penetration / 2
    };
    let right_shift = if contact.right_static {
        0
    } else if contact.left_static {
        contact.penetration
    } else {
        contact.penetration - left_shift
    };
    let left_delta = -contact.normal_sign * left_shift;
    let right_delta = contact.normal_sign * right_shift;
    let left = positions
        .get_mut(&left_id)
        .ok_or_else(|| format!("position absente pour l'entité {left_id}"))?;
    update_axis(left, contact.axis, left_delta)?;
    let right = positions
        .get_mut(&right_id)
        .ok_or_else(|| format!("position absente pour l'entité {right_id}"))?;
    update_axis(right, contact.axis, right_delta)
}

fn resolve_velocity_pair(
    velocities: &mut BTreeMap<EntityId, Vec2>,
    left_id: EntityId,
    right_id: EntityId,
    contact: &CollisionContact,
    left_collider: &Collider,
    right_collider: &Collider,
) -> crate::Result<()> {
    let left_before = *velocities
        .get(&left_id)
        .ok_or_else(|| format!("vélocité absente pour l'entité {left_id}"))?;
    let right_before = *velocities
        .get(&right_id)
        .ok_or_else(|| format!("vélocité absente pour l'entité {right_id}"))?;
    let mut left = left_before;
    let mut right = right_before;
    let left_normal = i128::from(axis_value(left, contact.axis)) * contact.normal_sign;
    let right_normal = i128::from(axis_value(right, contact.axis)) * contact.normal_sign;
    let restitution = left_collider
        .restitution_milli
        .min(right_collider.restitution_milli);

    if left_collider.is_static && right_collider.is_static {
        return Ok(());
    }
    if left_collider.is_static {
        if right_normal < 0 {
            set_axis_value(
                &mut right,
                contact.axis,
                round_mul(right_normal.abs(), restitution, 1000) * contact.normal_sign,
            )?;
        }
    } else if right_collider.is_static {
        if left_normal > 0 {
            set_axis_value(
                &mut left,
                contact.axis,
                -round_mul(left_normal, restitution, 1000) * contact.normal_sign,
            )?;
        }
    } else {
        let relative = right_normal - left_normal;
        if relative < 0 {
            let left_mass = i128::from(left_collider.mass_milli);
            let right_mass = i128::from(right_collider.mass_milli);
            let impulse_numerator = i128::from(1000 + restitution)
                .checked_mul(-relative)
                .and_then(|value| value.checked_mul(left_mass))
                .and_then(|value| value.checked_mul(right_mass))
                .ok_or("impulsion hors limite")?;
            let impulse_denominator = 1000_i128
                .checked_mul(left_mass + right_mass)
                .ok_or("impulsion hors limite")?;
            let impulse = round_div(impulse_numerator, impulse_denominator);
            let left_delta = round_div(impulse, left_mass);
            let right_delta = round_div(impulse, right_mass);
            let left_normal_after = left_normal - left_delta;
            let right_normal_after = right_normal + right_delta;
            set_axis_value(
                &mut left,
                contact.axis,
                left_normal_after * contact.normal_sign,
            )?;
            set_axis_value(
                &mut right,
                contact.axis,
                right_normal_after * contact.normal_sign,
            )?;
        }
    }
    velocities.insert(left_id, left);
    velocities.insert(right_id, right);
    Ok(())
}

fn axis_value(value: Vec2, axis: CollisionAxis) -> i64 {
    match axis {
        CollisionAxis::X => value.x,
        CollisionAxis::Y => value.y,
    }
}

fn set_axis_value(value: &mut Vec2, axis: CollisionAxis, next: i128) -> crate::Result<()> {
    let next = i64::try_from(next).map_err(|_| "vélocité de collision hors limite")?;
    match axis {
        CollisionAxis::X => value.x = next,
        CollisionAxis::Y => value.y = next,
    }
    Ok(())
}

fn update_axis(value: &mut Vec2, axis: CollisionAxis, delta: i128) -> crate::Result<()> {
    let current = i128::from(axis_value(*value, axis));
    set_axis_value(value, axis, current + delta)
}

fn round_div(numerator: i128, denominator: i128) -> i128 {
    (numerator + denominator / 2) / denominator
}

fn round_mul(value: i128, factor: u32, denominator: u32) -> i128 {
    (value * i128::from(factor) + i128::from(denominator / 2)) / i128::from(denominator)
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
            assert_eq!(first.collider(id), second.collider(id));
        }
    }

    fn collider_project(static_left: bool) -> Project {
        let left_velocity = if static_left { 99 } else { 1 };
        let right_velocity = if static_left { -1 } else { 0 };
        let left_static = if static_left { "true" } else { "false" };
        toml::from_str(&format!(
            "[project]\nname = \"collision\"\nformat_version = 1\n\n[simulation]\ntick_rate = 60\nseed = 1\n\n[[entities]]\nid = 1\nname = \"left\"\nposition = {{ x = 0, y = 0 }}\nvelocity = {{ x = {left_velocity}, y = 0 }}\ncollider = {{ half_width = 1, half_height = 1, mass_milli = 1000, restitution_milli = 1000, is_static = {left_static} }}\n\n[[entities]]\nid = 2\nname = \"right\"\nposition = {{ x = 1, y = 0 }}\nvelocity = {{ x = {right_velocity}, y = 0 }}\ncollider = {{ half_width = 1, half_height = 1, mass_milli = 1000, restitution_milli = 1000 }}\n"
        ))
        .unwrap()
    }

    #[test]
    fn dynamic_collision_is_canonical_and_restitutes() {
        let mut storage = EcsStorage::from_project(&collider_project(false));
        storage.move_all().unwrap();
        let stats = storage.resolve_collisions().unwrap();
        assert_eq!(stats.collisions_resolved, 1);
        assert_eq!(stats.entities_modified, 2);
        assert_eq!(storage.position(1), Some(&Vec2 { x: 0, y: 0 }));
        assert_eq!(storage.position(2), Some(&Vec2 { x: 2, y: 0 }));
        assert_eq!(storage.velocity(1), Some(&Vec2 { x: 0, y: 0 }));
        assert_eq!(storage.velocity(2), Some(&Vec2 { x: 1, y: 0 }));
    }

    #[test]
    fn static_collision_does_not_move_static_body() {
        let mut storage = EcsStorage::from_project(&collider_project(true));
        storage.move_all().unwrap();
        assert_eq!(storage.position(1), Some(&Vec2 { x: 0, y: 0 }));
        assert_eq!(storage.velocity(1), Some(&Vec2 { x: 0, y: 0 }));
        let stats = storage.resolve_collisions().unwrap();
        assert_eq!(stats.collisions_resolved, 1);
        assert_eq!(storage.position(1), Some(&Vec2 { x: 0, y: 0 }));
        assert_eq!(storage.position(2), Some(&Vec2 { x: 2, y: 0 }));
        assert_eq!(storage.velocity(2), Some(&Vec2 { x: 1, y: 0 }));
    }
}
