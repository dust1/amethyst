use std::{cmp::Ordering, marker::PhantomData};

use amethyst_core::ecs::*;
use derive_new::new;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{Selectable, Selected};
use amethyst_core::ecs::systems::ParallelRunnable;
use std::collections::HashSet;

// TODO: Optimize by using a tree. Should we enforce tab order = unique? Sort on insert.
/// A cache sorted by tab order and then by Entity.
/// Used to quickly find the next or previous selectable entities.
#[derive(Debug, Clone, Default)]
pub struct CachedSelectionOrder {
    pub cached: HashSet<Entity>,
    /// The cache holding the selection order and the corresponding entity.
    pub cache: Vec<(u32, Entity)>,
}

impl CachedSelectionOrder {
    /// Returns the index of the highest cached element (index in the cache!) that is currently selected.
    pub fn highest_order_selected_index<T>(
        &self,
        selected_storage: &T,
    ) -> Option<usize>
    where T: Iterator<Item=Selected> {
        self.cache
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (_, e))| selected_storage.get(*e).is_some())
            .map(|t| t.0)
    }

    /// Returns the index in the cache for the specified entity.
    pub fn index_of(&self, entity: Entity) -> Option<usize> {
        self.cache
            .iter()
            .enumerate()
            .find(|(_, (_, e))| *e == entity)
            .map(|t| t.0)
    }
}

/// System in charge of updating the CachedSelectionOrder resource on each frame.
#[derive(Debug, Default, new)]
pub struct CacheSelectionOrderSystem<G> {
    phantom: PhantomData<G>,
}

impl<G> CacheSelectionOrderSystem<G> {
    pub fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("CacheSelectionOrderSystem")
                .write_resource::<CachedSelectionOrder>()
                .with_query(<(Entity, &Selectable<G>)>::query())
                .build(move |_commands, world, cache,
                             selectables| {
                    #[cfg(feature = "profiler")]
                    profile_scope!("cache_selection_order_system");

                    {
                        let mut rm = vec![];
                        cache.cache.retain(|&(_t, entity)| {
                            let keep = selectables.get(world, entity).is_ok();
                            if !keep {
                                rm.push(entity);
                            }
                            keep
                        });
                        rm.iter().for_each(|e| {
                            cache.cached.remove(*e);
                        });
                    }

                    for &mut (ref mut t, entity) in &mut cache.cache {
                        *t = selectables.get(world, entity).unwrap().1.order;
                    }

                    // ---------

                    // Attempt to insert the new entities in sorted position.  Should reduce work during
                    // the sorting step.
                    {
                        let mut inserts = vec![];
                        let mut pushes = vec![];
                        {
                            selectables.for_each(world, | (entity, selectable)| {
                                // We only want the new ones.
                                // The old way (pre legion) to do it was with bitset :
                                // let new = (&transform_set ^ &cache.cached) & &transform_set;
                                if !cache.cached.contains(entity) {
                                    let pos = cache
                                        .cache
                                        .iter()
                                        .position(|&(cached_t, _)| selectable.order < cached_t);

                                    match pos {
                                        Some(pos) => inserts.push((pos, (selectable.order, entity))),
                                        None => pushes.push((selectable.order, entity)),
                                    }
                                }
                            });
                        }
                        inserts.iter().for_each(|e| cache.cache.insert(e.0, e.1));
                        pushes.iter().for_each(|e| cache.cache.push(*e));
                    }
                    // Update the cached with all entities

                    cache.cached.clear();
                    selectables.for_each(world, | (entity, selectable)| {
                        cache.cached.insert(entity);
                    });

                    cache
                        .cache
                        .sort_unstable_by(|&(t1, ref e1), &(t2, ref e2)| {
                            let ret = t1.cmp(&t2);
                            if ret == Ordering::Equal {
                                return e1.cmp(e2);
                            }
                            ret
                        });
                })
        )
    }
}
