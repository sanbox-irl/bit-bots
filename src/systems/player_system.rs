use super::{cardinals::FacingHorizontal, ActionMap, Component, ComponentList, Player, Sprite, Velocity};

pub fn player_update(
    players: &mut ComponentList<Player>,
    sprites: &mut ComponentList<Sprite>,
    velocities: &mut ComponentList<Velocity>,
    action_map: &ActionMap,
) {
    let mut active_player: Option<isize> = None;

    for (i, player) in players.iter_mut().enumerate() {
        let id = player.entity_id();
        let player: &mut Player = player.inner_mut();

        let velocity = velocities.get_mut_or_default(&id);
        let player_veloc: &mut Velocity = velocity.inner_mut();

        if player.active {
            if active_player.is_none() {
                active_player = Some(i as isize);
                player_veloc.intended_direction = action_map.move_direction;
            } else {
                error!("Two players are active! Something has gone wrong!");
            }
        }
    }

    // Check for Active Player
    if let Some(old_active_player) = active_player {
        if let Some(increment_active_player) = action_map.switch_active_player {
            let mut active_player = old_active_player;

            active_player += match increment_active_player {
                FacingHorizontal::Right => 1,
                FacingHorizontal::Left => -1,
            };

            let active_player = if active_player == -1 {
                (players.iter().count() - 1) as isize
            } else if active_player == players.iter().count() as isize {
                0
            } else {
                active_player
            } as usize;

            // The Players
            set_player_active(true, players.iter_mut().nth(active_player).unwrap(), sprites);
            set_player_active(
                false,
                players.iter_mut().nth(old_active_player as usize).unwrap(),
                sprites,
            );
        }
    } else if let Some(zeroeth) = players.iter_mut().nth(0) {
        info!("No players are active!");
        info!("Setting player zero to active!");
        set_player_active(true, zeroeth, sprites);
    }
}

pub fn initialize_players(players: &mut ComponentList<Player>, sprites: &mut ComponentList<Sprite>) {
    for player in players.iter_mut() {
        set_player_active(false, player, sprites);
    }

    if let Some(player) = players.iter_mut().nth(0) {
        set_player_active(true, player, sprites);
    }
}

fn set_player_active(
    active_status: bool,
    player: &mut Component<Player>,
    sprites: &mut ComponentList<Sprite>,
) {
    player.inner_mut().active = active_status;

    if let Some(player_sprite) = sprites.get_mut(&player.entity_id()) {
        player_sprite.inner_mut().running_data.is_animating = active_status;

        if active_status == false {
            player_sprite.inner_mut().running_data.current_frame = 0;
        }
    } else {
        error!("Player {} has no sprite!", player.entity_id());
    }
}
