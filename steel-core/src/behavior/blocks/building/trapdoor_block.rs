//! Trapdoor block behavior implementation.
//!
//! Redstone signal queries are isolated in `has_neighbor_signal`
//! until Steel has a redstone power graph.

use crate::{
    behavior::{
        BlockBehavior, BlockHitResult, BlockPlaceContext, InteractionResult, InventoryAccess,
    },
    entity::Entity,
    player::Player,
    world::{LevelReader, ScheduledTickAccess, World, game_event_context::GameEventContext},
};
use std::sync::Arc;
use steel_macros::block_behavior;
use steel_registry::{
    blocks::{
        BlockRef,
        block_state_ext::BlockStateExt as _,
        properties::{BlockStateProperties, Direction, Half},
    },
    sound_event::SoundEventRef,
    vanilla_fluids, vanilla_game_events,
};
use steel_utils::{BlockPos, BlockStateId, types::UpdateFlags};

/// Behavior for vanilla door blocks.
#[block_behavior]
pub struct TrapDoorBlock {
    block: BlockRef,
    #[json_arg(value, json = "type_can_open_by_hand")]
    can_open_by_hand: bool,
    #[json_arg(sound_events, json = "type_door_open")]
    sound_open: SoundEventRef,
    #[json_arg(sound_events, json = "type_door_close")]
    sound_close: SoundEventRef,
}

impl TrapDoorBlock {
    /// Creates a new door block behavior.
    #[must_use]
    pub const fn new(
        block: BlockRef,
        can_open_by_hand: bool,
        sound_open: SoundEventRef,
        sound_close: SoundEventRef,
    ) -> Self {
        Self {
            block,
            can_open_by_hand,
            sound_open,
            sound_close,
        }
    }

    const fn has_neighbor_signal<L: LevelReader + ?Sized>(_world: &L, _pos: BlockPos) -> bool {
        // TODO: Query redstone neighbor signal once Steel has redstone power propagation.
        false
    }

    fn play_sound(&self, player: Option<&Player>, world: &Arc<World>, pos: BlockPos, open: bool) {
        let sound = if open {
            self.sound_open
        } else {
            self.sound_close
        };
        world.play_block_sound(sound, pos, 1.0, 1.0, player.and_then(|p| Some(p.id())));
        world.game_event(
            if open {
                &vanilla_game_events::BLOCK_OPEN
            } else {
                &vanilla_game_events::BLOCK_CLOSE
            },
            pos,
            &GameEventContext::new(
                if let Some(player) = player {
                    Some(player)
                } else {
                    None
                },
                None,
            ),
        );
    }

    fn toggle(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos, player: &Player) {
        let block_state = state.set_value(
            &BlockStateProperties::OPEN,
            !state.get_value(&BlockStateProperties::OPEN),
        );
        world.set_block(pos, block_state, UpdateFlags::from_bits_retain(2));
        if block_state.get_value(&BlockStateProperties::WATERLOGGED) {
            let delay = world.fluid_tick_delay(&vanilla_fluids::WATER);
            let _ = world.schedule_fluid_tick_default(pos, &vanilla_fluids::WATER, delay);
        }
        self.play_sound(
            Some(player),
            world,
            pos,
            block_state.get_value(&BlockStateProperties::OPEN),
        );
    }
}

impl BlockBehavior for TrapDoorBlock {
    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let mut state = self.block.default_state();
        let face = context.clicked_face;
        if !context.replace_clicked && face.is_horizontal() {
            state = state
                .set_value(&BlockStateProperties::FACING, face)
                .set_value(
                    &BlockStateProperties::HALF,
                    if context.click_location.y - context.clicked_pos.y() as f64 > 0.5 {
                        Half::Top
                    } else {
                        Half::Bottom
                    },
                );
        } else {
            state = state
                .set_value(
                    &BlockStateProperties::FACING,
                    context.horizontal_direction.opposite(),
                )
                .set_value(
                    &BlockStateProperties::HALF,
                    if face == Direction::Up {
                        Half::Bottom
                    } else {
                        Half::Top
                    },
                );
        };

        if Self::has_neighbor_signal(context.world, context.clicked_pos) {
            state = state
                .set_value(&BlockStateProperties::OPEN, true)
                .set_value(&BlockStateProperties::POWERED, true);
        };

        Some(state.set_value(
            &BlockStateProperties::WATERLOGGED,
            context.is_water_source(),
        ))
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if state.get_value(&BlockStateProperties::WATERLOGGED) {
            let delay = world.fluid_tick_delay(&vanilla_fluids::WATER);
            let _ = world.schedule_fluid_tick_default(pos, &vanilla_fluids::WATER, delay);
        }
        state
    }

    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        _hit_result: &BlockHitResult,
        _inv: &mut InventoryAccess,
    ) -> InteractionResult {
        if !self.can_open_by_hand {
            InteractionResult::Pass
        } else {
            self.toggle(state, world, pos, player);
            InteractionResult::Success
        }
    }

    fn handle_neighbor_changed(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        _source_block: BlockRef,
        _moved_by_piston: bool,
    ) {
        let signal = Self::has_neighbor_signal(world, pos);
        let mut block_state = state;
        if signal != state.get_value(&BlockStateProperties::POWERED) {
            if signal != state.get_value(&BlockStateProperties::OPEN) {
                block_state = block_state.set_value(&BlockStateProperties::OPEN, signal);
                self.play_sound(None, world, pos, signal);
            }
        }
        world.set_block(
            pos,
            block_state.set_value(&BlockStateProperties::POWERED, signal),
            UpdateFlags::from_bits_retain(2),
        );
        if state.get_value(&BlockStateProperties::WATERLOGGED) {
            let delay = world.fluid_tick_delay(&vanilla_fluids::WATER);
            let _ = world.schedule_fluid_tick_default(pos, &vanilla_fluids::WATER, delay);
        }
    }
}
