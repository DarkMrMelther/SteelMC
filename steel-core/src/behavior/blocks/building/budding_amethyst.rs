use crate::{
    behavior::{BlockBehavior, BlockPlaceContext, BlockStateBehaviorExt},
    world::World,
};
use std::sync::Arc;
use steel_macros::block_behavior;
use steel_registry::{
    REGISTRY, RegistryEntry,
    blocks::{BlockRef, block_state_ext::BlockStateExt, properties::BlockStateProperties},
    vanilla_blocks,
};
use steel_utils::{BlockPos, BlockStateId, Direction, types::UpdateFlags};

/// Behavior for vanilla budding amethyst blocks.
#[block_behavior]
pub struct BuddingAmethystBlock {
    block: BlockRef,
}

impl BuddingAmethystBlock {
    /// Creates a new budding amethyst block behavior.
    #[must_use]
    pub fn new(block: BlockRef) -> Self {
        Self { block }
    }

    fn can_cluster_grow_at_state(state: BlockStateId, block_id: usize) -> bool {
        state.is_air()
            || (block_id == vanilla_blocks::WATER.id() && state.get_fluid_state().is_source())
    }

    fn check_cluster(
        state: BlockStateId,
        block_id: usize,
        direction: Direction,
        block: BlockRef,
    ) -> bool {
        block_id == block.id() && state.get_value(&BlockStateProperties::FACING) == direction
    }
}

impl BlockBehavior for BuddingAmethystBlock {
    fn get_state_for_placement(&self, _context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(self.block.default_state())
    }

    fn is_randomly_ticking(&self, _state: BlockStateId) -> bool {
        true
    }

    fn random_tick(&self, _state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if rand::random_range(0..5) == 0 {
            let direction = Direction::random();
            let grow_pos = pos.relative(direction);
            let state = world.get_block_state(grow_pos);
            let Some(&block_id) = REGISTRY.blocks.state_to_block_id.get(state.0 as usize) else {
                panic!(
                    "budding amethyst received invalid block state id {}",
                    state.0
                );
            };

            let mut stage: Option<BlockRef> = None;
            if Self::can_cluster_grow_at_state(state, block_id) {
                stage = Some(&vanilla_blocks::SMALL_AMETHYST_BUD);
            } else if Self::check_cluster(
                state,
                block_id,
                direction,
                &vanilla_blocks::SMALL_AMETHYST_BUD,
            ) {
                stage = Some(&vanilla_blocks::MEDIUM_AMETHYST_BUD);
            } else if Self::check_cluster(
                state,
                block_id,
                direction,
                &vanilla_blocks::MEDIUM_AMETHYST_BUD,
            ) {
                stage = Some(&vanilla_blocks::LARGE_AMETHYST_BUD);
            } else if Self::check_cluster(
                state,
                block_id,
                direction,
                &vanilla_blocks::LARGE_AMETHYST_BUD,
            ) {
                stage = Some(&vanilla_blocks::AMETHYST_CLUSTER);
            }

            if let Some(stage) = stage {
                let block_state = stage
                    .default_state()
                    .set_value(&BlockStateProperties::FACING, direction)
                    .set_value(
                        &BlockStateProperties::WATERLOGGED,
                        state
                            .try_get_value(&BlockStateProperties::WATERLOGGED)
                            .unwrap_or(false),
                    );
                world.set_block(grow_pos, block_state, UpdateFlags::UPDATE_ALL_IMMEDIATE);
            }
        }
    }
    // TODO: OnProjectile hit from AmethystBlock inheritance
}
