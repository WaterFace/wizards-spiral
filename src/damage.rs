use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Default)]
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MeleeAttackEvent>().add_systems(
            Update,
            (detect_melee_attacks, resolve_melee_attacks)
                .run_if(in_state(crate::states::GameState::InGame)),
        );
    }
}

#[derive(Debug, Clone, Event)]
pub struct MeleeAttackEvent {
    pub player: Entity,
    pub enemy: Entity,
}

fn detect_melee_attacks(
    mut collisions: EventReader<CollisionEvent>,
    player_query: Query<Entity, With<crate::player::Player>>,
    enemy_query: Query<Entity, With<crate::enemy::Enemy>>,
    mut writer: EventWriter<MeleeAttackEvent>,
) {
    for ev in collisions.read() {
        let CollisionEvent::Started(e1, e2, _flags) = ev else {
            // we only care about the `Started` events here
            continue;
        };

        let Ok(player) = player_query.get(*e1).or(player_query.get(*e2)) else {
            continue;
        };

        let Ok(enemy) = enemy_query.get(*e1).or(enemy_query.get(*e2)) else {
            continue;
        };

        writer.send(MeleeAttackEvent { player, enemy });
    }
}

fn resolve_melee_attacks(
    mut reader: EventReader<MeleeAttackEvent>,
    mut player_query: Query<
        (&mut ExternalImpulse, &GlobalTransform),
        (With<crate::player::Player>, Without<crate::enemy::Enemy>),
    >,
    mut enemy_query: Query<
        (
            &mut ExternalImpulse,
            &GlobalTransform,
            &crate::enemy::EnemyStats,
        ),
        (With<crate::enemy::Enemy>, Without<crate::player::Player>),
    >,
    player_skills: Res<crate::skills::PlayerSkills>,
) {
    for MeleeAttackEvent { player, enemy } in reader.read() {
        info!("player: {player:?}, enemy: {enemy:?}");
        let Ok((mut player_impulse, player_transform)) = player_query.get_mut(*player) else {
            warn!("resolve_melee_attacks: player query unsucessful");
            continue;
        };

        let Ok((mut enemy_impulse, enemy_transform, enemy_stats)) = enemy_query.get_mut(*enemy)
        else {
            warn!("resolve_melee_attacks: enemy query unsucessful");
            continue;
        };

        let player_pos = player_transform.translation().truncate();
        let enemy_pos = enemy_transform.translation().truncate();

        let dir = (player_pos - enemy_pos).normalize_or_zero();

        // mass, knockback
        let player_mass = player_skills.mass();
        let enemy_mass = enemy_stats.mass;
        player_impulse.impulse += dir * 300.0 * enemy_mass / player_mass;
        enemy_impulse.impulse -= dir * 300.0 * player_mass / enemy_mass;
    }
}
