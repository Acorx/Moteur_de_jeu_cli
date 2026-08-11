use serde::Serialize;

use crate::ecs::{EcsStorage, EntityId};
use crate::replay::{InputCommand, InputEvent};
use crate::rng::SplitMix64;
use crate::scheduler::{INPUT_SYSTEM, MOVEMENT_SYSTEM, Scheduler};
use crate::telemetry::{SystemCounters, TELEMETRY_SCHEMA, Telemetry};

use crate::Result;
use crate::project::{Appearance, Project};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EntityState {
    pub id: u64,
    pub name: String,
    pub position: Vec2,
    pub velocity: Vec2,
    pub appearance: Appearance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Vec2 {
    pub x: i64,
    pub y: i64,
}

#[derive(Serialize)]
struct Snapshot<'a> {
    schema: &'static str,
    project: &'a str,
    seed: u64,
    tick_rate: u32,
    tick: u64,
    entities: Vec<EntityState>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub schema: &'static str,
    pub project: String,
    pub seed: u64,
    pub tick_rate: u32,
    pub tick: u64,
    storage: EcsStorage,
    scheduler: Scheduler,
    rng: SplitMix64,
    counters: Vec<SystemCounters>,
}

impl World {
    pub fn from_project(project: Project) -> Self {
        let storage = EcsStorage::from_project(&project);
        Self {
            schema: "aetherion.snapshot/v1",
            project: project.project.name,
            seed: project.simulation.seed,
            tick_rate: project.simulation.tick_rate,
            tick: 0,
            storage,
            scheduler: Scheduler::standard(),
            rng: SplitMix64::new(project.simulation.seed),
            counters: vec![
                SystemCounters {
                    name: INPUT_SYSTEM,
                    ..SystemCounters::default()
                },
                SystemCounters {
                    name: MOVEMENT_SYSTEM,
                    ..SystemCounters::default()
                },
            ],
        }
    }

    pub fn entity_count(&self) -> usize {
        self.storage.len()
    }

    pub fn entities(&self) -> impl Iterator<Item = EntityState> + '_ {
        self.storage
            .ids()
            .map(|id| self.entity(id).expect("stockages ECS cohérents"))
    }

    pub fn entity(&self, id: EntityId) -> Option<EntityState> {
        Some(EntityState {
            id,
            name: self.storage.metadata(id)?.name.clone(),
            position: *self.storage.position(id)?,
            velocity: *self.storage.velocity(id)?,
            appearance: self.storage.appearance(id)?.clone(),
        })
    }

    pub fn position(&self, id: EntityId) -> Option<Vec2> {
        self.storage.position(id).copied()
    }
    pub fn velocity(&self, id: EntityId) -> Option<Vec2> {
        self.storage.velocity(id).copied()
    }
    pub fn sprite(&self, id: EntityId) -> Option<&crate::project::SpriteConfig> {
        self.storage.sprite(id)
    }
    pub fn scheduler_order(&self) -> &[&'static str] {
        self.scheduler.order()
    }
    pub fn rng_state(&self) -> u64 {
        self.rng.state()
    }

    pub fn next_random_u64(&mut self) -> u64 {
        self.counters[0].prng_calls += 1;
        self.rng.next_u64()
    }

    pub fn step(&mut self) -> Result<()> {
        self.step_with_events(&[])
    }

    pub fn step_with_events(&mut self, events: &[InputEvent]) -> Result<()> {
        for index in 0..self.scheduler.order().len() {
            match self.scheduler.order()[index] {
                INPUT_SYSTEM => self.run_input(events)?,
                MOVEMENT_SYSTEM => self.run_movement()?,
                name => return Err(format!("système non exécutable: {name}").into()),
            }
        }
        self.tick = self
            .tick
            .checked_add(1)
            .ok_or("nombre de ticks hors limite")?;
        Ok(())
    }

    fn run_input(&mut self, events: &[InputEvent]) -> Result<()> {
        self.counters[0].ticks += 1;
        for event in events {
            self.apply_event(event)?;
            self.counters[0].events_applied += 1;
            self.counters[0].entities_visited += 1;
            self.counters[0].entities_modified += 1;
        }
        Ok(())
    }

    fn run_movement(&mut self) -> Result<()> {
        self.counters[1].ticks += 1;
        let visited = self.storage.move_all()?;
        self.counters[1].entities_visited += visited;
        self.counters[1].entities_modified += visited;
        Ok(())
    }

    pub fn apply_event(&mut self, event: &InputEvent) -> Result<()> {
        match event.command {
            InputCommand::SetVelocity { x, y } => {
                *self.velocity_mut(event.entity_id)? = Vec2 { x, y }
            }
            InputCommand::Impulse { x, y } => {
                let value = self.velocity_mut(event.entity_id)?;
                value.x = value.x.checked_add(x).ok_or("impulsion X hors limite")?;
                value.y = value.y.checked_add(y).ok_or("impulsion Y hors limite")?;
            }
            InputCommand::Translate { x, y } => {
                let value = self.position_mut(event.entity_id)?;
                value.x = value.x.checked_add(x).ok_or("translation X hors limite")?;
                value.y = value.y.checked_add(y).ok_or("translation Y hors limite")?;
            }
            InputCommand::Stop => *self.velocity_mut(event.entity_id)? = Vec2 { x: 0, y: 0 },
        }
        Ok(())
    }

    fn position_mut(&mut self, id: EntityId) -> Result<&mut Vec2> {
        self.storage
            .position_mut(id)
            .ok_or_else(|| format!("entité inconnue: {id}").into())
    }

    fn velocity_mut(&mut self, id: EntityId) -> Result<&mut Vec2> {
        self.storage
            .velocity_mut(id)
            .ok_or_else(|| format!("entité inconnue: {id}").into())
    }

    pub fn run(&mut self, ticks: u64) -> Result<()> {
        for _ in 0..ticks {
            self.step()?;
        }
        Ok(())
    }

    fn snapshot(&self) -> Snapshot<'_> {
        Snapshot {
            schema: self.schema,
            project: &self.project,
            seed: self.seed,
            tick_rate: self.tick_rate,
            tick: self.tick,
            entities: self.entities().collect(),
        }
    }

    pub fn snapshot_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.snapshot())
            .map_err(|error| format!("sérialisation du snapshot: {error}").into())
    }

    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in serde_json::to_vec(&self.snapshot()).unwrap_or_default() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    pub fn telemetry(&self) -> Telemetry {
        Telemetry {
            schema: TELEMETRY_SCHEMA,
            tick: self.tick,
            checksum: self.checksum(),
            system_order: self.scheduler.order().to_vec(),
            systems: self.counters.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::from_project(toml::from_str(Project::example()).unwrap())
    }

    #[test]
    fn movement_is_deterministic() {
        let mut a = world();
        let mut b = world();
        a.run(100).unwrap();
        b.run(100).unwrap();
        assert_eq!(
            a.entities().collect::<Vec<_>>(),
            b.entities().collect::<Vec<_>>()
        );
        assert_eq!(a.checksum(), b.checksum());
        assert_eq!(a.position(1).unwrap().x, 100);
    }

    #[test]
    fn zero_ticks_preserves_initial_state() {
        let mut state = world();
        let checksum = state.checksum();
        state.run(0).unwrap();
        assert_eq!(state.tick, 0);
        assert_eq!(state.checksum(), checksum);
    }

    #[test]
    fn telemetry_is_stable_and_outside_the_checksum() {
        let mut first = world();
        let mut second = world();
        first.run(3).unwrap();
        second.run(3).unwrap();
        assert_eq!(first.telemetry(), second.telemetry());
        assert_eq!(first.telemetry().system_order, vec!["input", "movement"]);
        assert_eq!(first.telemetry().systems[0].ticks, 3);
        assert_eq!(first.telemetry().systems[1].entities_visited, 6);
        let checksum = first.checksum();
        first.next_random_u64();
        assert_eq!(first.checksum(), checksum);
        assert_eq!(first.telemetry().systems[0].prng_calls, 1);
    }
}
