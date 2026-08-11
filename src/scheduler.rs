use std::collections::HashSet;

use crate::Result;

pub const INPUT_SYSTEM: &str = "input";
pub const MOVEMENT_SYSTEM: &str = "movement";
pub const DEFAULT_SYSTEM_ORDER: [&str; 2] = [INPUT_SYSTEM, MOVEMENT_SYSTEM];

#[derive(Clone, Debug)]
pub struct Scheduler {
    order: Vec<&'static str>,
}

impl Scheduler {
    pub fn new(order: Vec<&'static str>) -> Result<Self> {
        if order.is_empty() {
            return Err("l'ordonnanceur doit contenir au moins un système".into());
        }
        let known = [INPUT_SYSTEM, MOVEMENT_SYSTEM];
        let mut seen = HashSet::new();
        for name in &order {
            if !known.contains(name) {
                return Err(format!("système inconnu: {name}").into());
            }
            if !seen.insert(*name) {
                return Err(format!("système dupliqué: {name}").into());
            }
        }
        if !seen.contains(INPUT_SYSTEM) || !seen.contains(MOVEMENT_SYSTEM) {
            return Err("les systèmes input et movement sont obligatoires".into());
        }
        let input = order
            .iter()
            .position(|name| *name == INPUT_SYSTEM)
            .unwrap_or(usize::MAX);
        let movement = order
            .iter()
            .position(|name| *name == MOVEMENT_SYSTEM)
            .unwrap_or(usize::MAX);
        if input > movement {
            return Err("dépendance impossible: input doit précéder movement".into());
        }
        Ok(Self { order })
    }

    pub fn standard() -> Self {
        Self {
            order: DEFAULT_SYSTEM_ORDER.to_vec(),
        }
    }

    pub fn order(&self) -> &[&'static str] {
        &self.order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_order_is_inspectable() {
        assert_eq!(Scheduler::standard().order(), DEFAULT_SYSTEM_ORDER);
    }

    #[test]
    fn invalid_orders_are_rejected() {
        assert!(Scheduler::new(vec![MOVEMENT_SYSTEM, INPUT_SYSTEM]).is_err());
        assert!(Scheduler::new(vec![INPUT_SYSTEM, INPUT_SYSTEM]).is_err());
        assert!(Scheduler::new(vec!["unknown", MOVEMENT_SYSTEM]).is_err());
    }
}
