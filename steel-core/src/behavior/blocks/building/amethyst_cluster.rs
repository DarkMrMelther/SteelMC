use crate::{
    behavior::{BlockBehavior, BlockPlaceContext},
    world::LevelReader,
};
use steel_macros::block_behavior;
use steel_registry::{
    blocks::{BlockRef, block_state_ext::BlockStateExt, properties::BlockStateProperties},
    vanilla_blocks, vanilla_fluids,
};
use steel_utils::{BlockPos, BlockStateId, Direction};

/// Behavior for vanilla amethyst clusters blocks.
#[block_behavior]
pub struct AmethystClusterBlock {
    block: BlockRef,
}

impl AmethystClusterBlock {
    /// Creates a new cluster block behavior.
    #[must_use]
    pub fn new(block: BlockRef) -> Self {
        Self { block }
    }
}

impl BlockBehavior for AmethystClusterBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        Some(
            self.block
                .default_state()
                .set_value(
                    &BlockStateProperties::WATERLOGGED,
                    context.is_water_source(),
                )
                .set_value(&BlockStateProperties::FACING, context.clicked_face),
        )
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn crate::world::ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if state.get_value(&BlockStateProperties::WATERLOGGED) {
            let delay = world.fluid_tick_delay(&vanilla_fluids::WATER);
            let _ = world.schedule_fluid_tick_default(pos, &vanilla_fluids::WATER, delay);
        }

        if direction == state.get_value(&BlockStateProperties::FACING).opposite()
            && !self.can_survive(state, world, pos)
        {
            vanilla_blocks::AIR.default_state()
        } else {
            state
        }
    }

    fn can_survive(&self, state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        let direction = state.get_value(&BlockStateProperties::FACING);
        let adjacent = pos.relative(direction.opposite());
        world.get_block_state(adjacent).is_face_sturdy(direction)
    }
    // TODO: OnProjectile hit from AmethystBlock inheritance
    // TODO: Mirror and Rotate functions
}
