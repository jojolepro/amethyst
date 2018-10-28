use amethyst_core::specs::WriteStorage;
use std::marker::PhantomData;
use amethyst_core::specs::System;
use amethyst_core::specs::Write;
use amethyst_core::specs::Entities;
use amethyst_core::specs::Join;
use hibitset::BitSet;
use std::cmp::Ordering;
use amethyst_core::specs::Entity;

use amethyst_core::specs::ReadStorage;

use {Selected, Selectable};

/// A cache sorted by tab order, and then by Entity.
#[derive(Debug, Clone, Default)]
pub struct CachedSelectionOrder {
    pub cached: BitSet,
    pub cache: Vec<(u32, Entity)>,
}

impl CachedSelectionOrder {
    // TODO: Change WriteStorage to ReadStorage
    /// Returns the index of the highest cached element (index in the cache!) that is currently selected.
    pub fn highest_order_selected_index(&self, selected_storage: &WriteStorage<Selected>) -> Option<usize>{
        self.cache.iter().enumerate().rev().find(|(_,(_, e))| selected_storage.get(*e).is_some()).map(|t| t.0)
    }

    pub fn index_of(&self, entity: Entity) -> Option<usize> {
        self.cache.iter().enumerate().find(|(_, (_, e))| *e == entity).map(|t| t.0)
    }
}

#[derive(Debug, Default)]
pub struct CacheSelectionOrderSystem<G> {
    phantom: PhantomData<G>,
}

impl<'a, G> System<'a> for CacheSelectionOrderSystem<G> 
where
    G: PartialEq + Clone + Send + Sync + 'static
{
	type SystemData = (
		Entities<'a>,
		Write<'a, CachedSelectionOrder>,
		ReadStorage<'a, Selectable<G>>,
	);
	fn run(&mut self, (entities, mut cache, selectables): Self::SystemData) {
		{
            let mut rm = vec![];
            cache.cache.retain(|&(_t, entity)| {
                let keep = selectables.contains(entity);
                if !keep {
                    rm.push(entity.id());
                }
                keep
            });
            rm.iter().for_each(|e| {&mut cache.cached.remove(*e); ()});
        }

        for &mut (ref mut t, entity) in &mut cache.cache {
            *t = selectables.get(entity).unwrap().order;
        }

        // Attempt to insert the new entities in sorted position.  Should reduce work during
        // the sorting step.
        let transform_set = selectables.mask().clone();
        {
            let mut inserts = vec![];
            let mut pushes = vec![];
            {
                // Create a bitset containing only the new indices.
                let new = (&transform_set ^ &cache.cached) & &transform_set;
                for (entity, selectable, _new) in (&*entities, &selectables, &new).join() {
                    let pos = cache
                        .cache
                        .iter()
                        .position(|&(cached_t, _)| selectable.order < cached_t);
                    match pos {
                        Some(pos) => inserts.push((pos, (selectable.order, entity))),
                        None => pushes.push((selectable.order, entity)),
                    }
                }
            }
            inserts.iter().for_each(|e| cache.cache.insert(e.0, e.1));
            pushes.iter().for_each(|e| cache.cache.push(*e));
        }
        cache.cached = transform_set;

        // Sort from smallest tab order to largest tab order, then by entity creation time.
        // Most of the time this shouldn't do anything but you still need it for if the tab orders
        // change.
        cache
            .cache
            .sort_unstable_by(|&(t1, ref e1), &(t2, ref e2)| {
                let ret = t1.cmp(&t2);
                if ret == Ordering::Equal {
                    return e1.cmp(e2);
                }
                ret
            });
	}
}