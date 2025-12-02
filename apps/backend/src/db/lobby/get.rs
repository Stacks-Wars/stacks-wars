use std::collections::{HashMap, HashSet};

use bb8::PooledConnection;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    db::{
        game::get::get_game,
        user::get::{get_user_by_id, get_user_by_id_with_conn},
    },
    errors::AppError,
    models::{
        game::{
            ClaimState, LobbyExtended, LobbyInfo, LobbyState, Player, PlayerLobbyInfo, PlayerState,
        },
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};

pub async fn get_lobbies_by_game_id(
    game_id: Uuid,
    lobby_filters: Option<Vec<LobbyState>>,
    page: u32,
    limit: u32,
    redis: RedisClient,
) -> Result<Vec<LobbyInfo>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;
    let offset = ((page.saturating_sub(1)) as usize).saturating_mul(limit as usize);
    let end = offset + (limit as usize) - 1;

    // 1) build the list of lobby IDs (filtered by state if provided)
    let lobby_ids: Vec<String> = if let Some(states) = lobby_filters {
        // Union all the per‐state sorted sets
        let state_keys: Vec<String> = states
            .iter()
            .map(|state| RedisKey::lobbies_state(state))
            .collect();
        let union_key = RedisKey::temp_union();
        let _: () = redis::cmd("ZUNIONSTORE")
            .arg(&union_key)
            .arg(state_keys.len())
            .arg(&state_keys)
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;
        let _: Option<()> = redis::cmd("EXPIRE")
            .arg(&union_key)
            .arg(30)
            .query_async(&mut *conn)
            .await
            .ok();

        // Now intersect with the game‐specific set
        let game_key = RedisKey::game_lobbies(KeyPart::Id(game_id));
        let inter_key = RedisKey::temp_inter();
        let _: () = redis::cmd("ZINTERSTORE")
            .arg(&inter_key)
            .arg(2)
            .arg(&game_key)
            .arg(&union_key)
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;
        let _: Option<()> = redis::cmd("EXPIRE")
            .arg(&inter_key)
            .arg(30)
            .query_async(&mut *conn)
            .await
            .ok();

        // Page through the intersection
        let ids: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(&inter_key)
            .arg(offset)
            .arg(end)
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        // Cleanup
        let _: Option<()> = redis::cmd("DEL")
            .arg(&union_key)
            .query_async(&mut *conn)
            .await
            .ok();
        let _: Option<()> = redis::cmd("DEL")
            .arg(&inter_key)
            .query_async(&mut *conn)
            .await
            .ok();
        ids
    } else {
        // No state filter → page straight out of game:{game_id}:lobbies
        redis::cmd("ZREVRANGE")
            .arg(RedisKey::game_lobbies(KeyPart::Id(game_id)))
            .arg(offset)
            .arg(end)
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?
    };

    // Filter and collect only valid UUIDs
    let valid_ids: Vec<Uuid> = lobby_ids
        .iter()
        .filter_map(|id_str| Uuid::parse_str(id_str).ok())
        .collect();

    // Batch all HGETALLs using a Redis pipeline
    let mut pipe = redis::pipe();
    for lobby_id in &valid_ids {
        let key = RedisKey::lobby(KeyPart::Id(*lobby_id));
        pipe.cmd("HGETALL").arg(key);
    }

    // Execute pipeline and collect responses
    let results: Vec<HashMap<String, String>> = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Parse partial lobbies and collect unique creator and game IDs
    let mut partial_lobbies = Vec::new();
    let mut creator_ids = HashSet::new();
    let mut game_ids = HashSet::new();

    for map in results {
        if let Ok((lobby, creator_id, game_id)) = LobbyInfo::from_redis_hash_partial(&map) {
            creator_ids.insert(creator_id);
            game_ids.insert(game_id);
            partial_lobbies.push((lobby, creator_id, game_id));
        }
    }

    // Batch fetch all creators and games
    let mut creators = HashMap::new();
    let mut games = HashMap::new();

    // Fetch creators
    for creator_id in creator_ids {
        if let Ok(creator) = get_user_by_id(creator_id, redis.clone()).await {
            creators.insert(creator_id, creator);
        }
    }

    // Fetch games
    for game_id in game_ids {
        if let Ok(game) = get_game(game_id, redis.clone()).await {
            games.insert(game_id, game);
        }
    }

    // Hydrate creators and games
    let mut out = Vec::new();
    for (mut lobby, creator_id, game_id) in partial_lobbies {
        if let (Some(creator), Some(game)) = (creators.get(&creator_id), games.get(&game_id)) {
            lobby.creator = creator.clone();
            lobby.game = game.clone();
            out.push(lobby);
        }
    }

    Ok(out)
}

pub async fn get_lobby_info(lobby_id: Uuid, redis: RedisClient) -> Result<LobbyInfo, AppError> {
    let redis_clone = redis.clone();
    let mut conn = redis_clone.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::lobby(KeyPart::Id(lobby_id));
    let map: HashMap<String, String> = redis::cmd("HGETALL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    if map.is_empty() {
        return Err(AppError::NotFound(format!("Lobby {} not found", lobby_id)));
    }

    // Get partial lobby info and creator ID
    let (mut info, creator_id, game_id) = LobbyInfo::from_redis_hash_partial(&map)?;

    // Hydrate both creator and game concurrently
    let (creator_result, game_result) = tokio::try_join!(
        get_user_by_id(creator_id, redis.clone()),
        get_game(game_id, redis.clone())
    )?;

    info.creator = creator_result;
    info.game = game_result;

    tracing::debug!("{info:?}");

    Ok(info)
}

pub async fn get_connected_players_ids(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Vec<Uuid>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let connected_key = RedisKey::lobby_connected_players(KeyPart::Id(lobby_id));

    // Get all connected player IDs from the set
    let player_id_strings: Vec<String> = conn
        .smembers(&connected_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Convert to UUIDs
    let player_ids: Vec<Uuid> = player_id_strings
        .into_iter()
        .filter_map(|id_str| Uuid::parse_str(&id_str).ok())
        .collect();

    Ok(player_ids)
}

pub async fn get_current_players_ids(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Vec<Uuid>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let current_key = RedisKey::lobby_current_players(KeyPart::Id(lobby_id));

    // Get all current player IDs from the set
    let player_id_strings: Vec<String> = conn
        .smembers(&current_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Convert to UUIDs
    let player_ids: Vec<Uuid> = player_id_strings
        .into_iter()
        .filter_map(|id_str| Uuid::parse_str(&id_str).ok())
        .collect();

    Ok(player_ids)
}

pub async fn _get_connected_players(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Vec<Player>, AppError> {
    let redis_clone = redis.clone();
    let mut conn = redis_clone.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let connected_key = RedisKey::lobby_connected_players(KeyPart::Id(lobby_id));

    // Get all connected player IDs from the set
    let player_id_strings: Vec<String> = conn
        .smembers(&connected_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if player_id_strings.is_empty() {
        return Ok(Vec::new());
    }

    // Convert to UUIDs and get the actual player data
    let mut players = Vec::new();
    for id_str in player_id_strings {
        if let Ok(player_id) = Uuid::parse_str(&id_str) {
            // Get player data from the main player hash
            let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));
            let player_data: std::collections::HashMap<String, String> = conn
                .hgetall(&player_key)
                .await
                .map_err(AppError::RedisCommandError)?;

            if !player_data.is_empty() {
                if let Ok(player) = Player::from_redis_hash(&player_data) {
                    players.push(player);
                }
            }
        }
    }

    // Hydrate all players with user data
    Ok(hydrate_players(players, redis).await)
}

pub async fn _get_connected_players_count(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<usize, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let connected_key = RedisKey::lobby_connected_players(KeyPart::Id(lobby_id));

    let count: usize = conn
        .scard(&connected_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(count)
}

pub async fn _is_player_connected(
    lobby_id: Uuid,
    player_id: Uuid,
    redis: RedisClient,
) -> Result<bool, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let connected_key = RedisKey::lobby_connected_players(KeyPart::Id(lobby_id));

    let is_member: bool = conn
        .sismember(&connected_key, player_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(is_member)
}

pub async fn get_all_lobbies_info(
    lobby_filters: Option<Vec<LobbyState>>,
    page: u32,
    limit: u32,
    redis: RedisClient,
) -> Result<Vec<LobbyInfo>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis conn timed out".into()),
    })?;

    let offset = ((page.saturating_sub(1)) as usize).saturating_mul(limit as usize);
    let end = offset + (limit as usize) - 1;
    let uuids: Vec<Uuid> = fetch_lobby_uuids(&mut conn, lobby_filters, offset, end).await?;

    if uuids.is_empty() {
        return Ok(Vec::new());
    }

    // Batch all HGETALLs using a Redis pipeline
    let mut pipe = redis::pipe();
    for uuid in &uuids {
        let key = RedisKey::lobby(KeyPart::Id(*uuid));
        pipe.cmd("HGETALL").arg(key);
    }

    // Execute pipeline and collect responses
    let results: Vec<HashMap<String, String>> = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Parse partial lobbies and collect unique creator and game IDs
    let mut partial_lobbies = Vec::new();
    let mut creator_ids = HashSet::new();
    let mut game_ids = HashSet::new();

    for map in results {
        if let Ok((lobby, creator_id, game_id)) = LobbyInfo::from_redis_hash_partial(&map) {
            creator_ids.insert(creator_id);
            game_ids.insert(game_id);
            partial_lobbies.push((lobby, creator_id, game_id));
        }
    }

    // Batch fetch all creators and games
    let mut creators = HashMap::new();
    let mut games = HashMap::new();

    // Fetch creators
    for creator_id in creator_ids {
        if let Ok(creator) = get_user_by_id(creator_id, redis.clone()).await {
            creators.insert(creator_id, creator);
        }
    }

    // Fetch games
    for game_id in game_ids {
        if let Ok(game) = get_game(game_id, redis.clone()).await {
            games.insert(game_id, game);
        }
    }

    // Hydrate creators and games
    let mut out = Vec::new();
    for (mut lobby, creator_id, game_id) in partial_lobbies {
        if let (Some(creator), Some(game)) = (creators.get(&creator_id), games.get(&game_id)) {
            lobby.creator = creator.clone();
            lobby.game = game.clone();
            out.push(lobby);
        }
    }

    Ok(out)
}

pub async fn hydrate_players(players: Vec<Player>, redis: RedisClient) -> Vec<Player> {
    // Monitor pool health
    let pool_state = redis.state();
    tracing::debug!(
        "Redis pool before hydration - connections: {}, idle: {}",
        pool_state.connections,
        pool_state.idle_connections
    );

    // Get a single connection for all operations to reduce pool pressure
    let mut conn = match redis.get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Failed to get Redis connection for hydration: {}", e);
            return players; // Return unhydrated players rather than failing
        }
    };

    let mut hydrated = Vec::new();

    // Collect users that need hydration
    let user_ids_to_hydrate: Vec<Uuid> = players
        .iter()
        .filter(|p| p.user.is_none())
        .map(|p| p.id)
        .collect();

    // Batch hydrate users using the same connection
    let mut users_map = HashMap::new();
    for user_id in user_ids_to_hydrate {
        match get_user_by_id_with_conn(user_id, &mut conn).await {
            Ok(user) => {
                users_map.insert(user_id, user);
            }
            Err(e) => {
                tracing::warn!("Failed to hydrate user {}: {}", user_id, e);
            }
        }
    }

    // Apply user data to players
    for mut player in players {
        if player.user.is_none() {
            if let Some(user) = users_map.remove(&player.id) {
                player.user = Some(user);
            }
        }
        hydrated.push(player);
    }

    hydrated
}

pub async fn get_lobby_player(
    lobby_id: Uuid,
    user_id: Uuid,
    redis: RedisClient,
) -> Result<Player, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(user_id));

    if !conn
        .exists(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?
    {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            user_id, lobby_id
        )));
    }

    let player_map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if player_map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            user_id, lobby_id
        )));
    }

    let player = Player::from_redis_hash(&player_map)?;
    Ok(player)
}

pub async fn get_lobby_players(
    lobby_id: Uuid,
    players_filter: Option<PlayerState>,
    redis: RedisClient,
) -> Result<Vec<Player>, AppError> {
    let redis_clone = redis.clone();
    let mut conn = redis_clone.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let pattern = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Wildcard);
    let player_keys: Vec<String> = redis::cmd("KEYS")
        .arg(&pattern)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    if player_keys.is_empty() {
        return Ok(Vec::new());
    }

    // Batch fetch all player data
    let mut pipe = redis::pipe();
    for key in &player_keys {
        pipe.cmd("HGETALL").arg(key);
    }

    let player_results: Vec<HashMap<String, String>> = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Parse players without user data and apply filter
    let mut players = Vec::new();
    for player_data in player_results {
        if let Ok(player) = Player::from_redis_hash(&player_data) {
            // Apply player state filter if specified
            let passes_filter = match &players_filter {
                Some(filter_state) => player.state == *filter_state,
                None => true,
            };

            if passes_filter {
                players.push(player);
            }
        }
    }

    // Hydrate all players with user data
    Ok(hydrate_players(players, redis).await)
}

pub async fn get_lobby_extended(
    lobby_id: Uuid,
    players_filter: Option<PlayerState>,
    redis: RedisClient,
) -> Result<LobbyExtended, AppError> {
    let lobby = get_lobby_info(lobby_id, redis.clone()).await?;
    let players = get_lobby_players(lobby_id, players_filter, redis.clone()).await?;

    Ok(LobbyExtended { lobby, players })
}

pub async fn get_all_lobbies_extended(
    lobby_filters: Option<Vec<LobbyState>>,
    players_filter: Option<PlayerState>,
    page: u32,
    limit: u32,
    redis: RedisClient,
) -> Result<Vec<LobbyExtended>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let offset = ((page.saturating_sub(1)) as usize).saturating_mul(limit as usize);
    let end = offset + (limit as usize) - 1;

    let uuids: Vec<Uuid> = fetch_lobby_uuids(&mut conn, lobby_filters, offset, end).await?;

    if uuids.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::with_capacity(uuids.len());
    for lobby_id in uuids {
        match get_lobby_info(lobby_id, redis.clone()).await {
            Ok(lobby) => {
                match get_lobby_players(lobby_id, players_filter.clone(), redis.clone()).await {
                    Ok(players) => {
                        out.push(LobbyExtended { lobby, players });
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get players for lobby {}: {}", lobby_id, e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to get lobby info for {}: {}", lobby_id, e);
            }
        }
    }

    Ok(out)
}

pub async fn get_player_lobbies(
    user_id: Uuid,
    claim_filter: Option<ClaimState>,
    lobby_filters: Option<Vec<LobbyState>>,
    page: u32,
    limit: u32,
    redis: RedisClient,
) -> Result<Vec<PlayerLobbyInfo>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    // Get all lobby player keys for this user
    let player_pattern = RedisKey::lobby_player(KeyPart::Wildcard, KeyPart::Id(user_id));
    let player_keys: Vec<String> = redis::cmd("KEYS")
        .arg(&player_pattern)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    if player_keys.is_empty() {
        return Ok(Vec::new());
    }

    // Batch fetch all player data using pipeline
    let mut pipe = redis::pipe();
    for key in &player_keys {
        pipe.cmd("HGETALL").arg(key);
    }

    let player_results: Vec<HashMap<String, String>> = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Filter by claim state and collect player data with lobby IDs
    let mut filtered_data = Vec::new();

    for (key, player_data) in player_keys.iter().zip(player_results.iter()) {
        if let Ok(player) = Player::from_redis_hash(player_data) {
            // Skip lobbies without prizes when filtering by claim state
            if claim_filter.is_some() && player.prize.is_none() {
                continue;
            }

            // Apply claim filter if specified
            let passes_claim_filter = match (&claim_filter, &player.claim) {
                (Some(ClaimState::NotClaimed), Some(ClaimState::NotClaimed)) => true,
                (Some(ClaimState::NotClaimed), None) => player.prize.is_some(),
                (Some(ClaimState::Claimed { .. }), Some(ClaimState::Claimed { .. })) => true,
                (None, _) => true,
                _ => false,
            };

            if passes_claim_filter {
                if let Some(lobby_id) = RedisKey::extract_lobby_id_from_player_key(key) {
                    filtered_data.push((lobby_id, player.prize, player.rank, player.claim));
                }
            }
        }
    }

    if filtered_data.is_empty() {
        return Ok(Vec::new());
    }

    // Extract unique lobby IDs for fetching lobby info
    let lobby_ids: Vec<Uuid> = filtered_data.iter().map(|(id, _, _, _)| *id).collect();

    // Batch fetch lobby info
    let mut pipe = redis::pipe();
    for lobby_id in &lobby_ids {
        let key = RedisKey::lobby(KeyPart::Id(*lobby_id));
        pipe.cmd("HGETALL").arg(key);
    }

    let lobby_results: Vec<HashMap<String, String>> = pipe
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Parse and collect with timestamps and player data
    let mut lobbies_with_data = Vec::new();
    for ((_lobby_id, prize, rank, claim), lobby_data) in
        filtered_data.iter().zip(lobby_results.iter())
    {
        if let Ok((partial_lobby, creator_id, game_id)) =
            LobbyInfo::from_redis_hash_partial(lobby_data)
        {
            // Apply lobby state filter here
            let passes_lobby_filter = match &lobby_filters {
                Some(states) => states.contains(&partial_lobby.state),
                None => true,
            };

            if passes_lobby_filter {
                lobbies_with_data.push((
                    partial_lobby,
                    creator_id,
                    game_id,
                    *prize,
                    *rank,
                    claim.clone(),
                ));
            }
        }
    }

    // Sort by created_at (newest first)
    lobbies_with_data.sort_by(|a, b| b.0.created_at.cmp(&a.0.created_at));

    // Apply pagination
    let offset = ((page.saturating_sub(1)) as usize).saturating_mul(limit as usize);
    let paginated_lobbies: Vec<_> = lobbies_with_data
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .collect();

    if paginated_lobbies.is_empty() {
        return Ok(Vec::new());
    }

    // Collect unique creator and game IDs for batch fetching
    let mut creator_ids = HashSet::new();
    let mut game_ids = HashSet::new();

    for (_, creator_id, game_id, _, _, _) in &paginated_lobbies {
        creator_ids.insert(*creator_id);
        game_ids.insert(*game_id);
    }

    // Batch fetch creators and games
    let mut creators = HashMap::new();
    let mut games = HashMap::new();

    // Fetch creators
    for creator_id in creator_ids {
        if let Ok(creator) = get_user_by_id(creator_id, redis.clone()).await {
            creators.insert(creator_id, creator);
        }
    }

    // Fetch games
    for game_id in game_ids {
        if let Ok(game) = get_game(game_id, redis.clone()).await {
            games.insert(game_id, game);
        }
    }

    // Hydrate and build final result with player data
    let mut result = Vec::new();
    for (mut lobby, creator_id, game_id, prize, rank, claim) in paginated_lobbies {
        if let (Some(creator), Some(game)) = (creators.get(&creator_id), games.get(&game_id)) {
            lobby.creator = creator.clone();
            lobby.game = game.clone();

            result.push(PlayerLobbyInfo {
                lobby,
                prize_amount: prize,
                rank,
                claim_state: claim,
            });
        }
    }

    Ok(result)
}

async fn fetch_lobby_uuids(
    conn: &mut PooledConnection<'_, RedisConnectionManager>,
    lobby_filters: Option<Vec<LobbyState>>,
    offset: usize,
    end: usize,
) -> Result<Vec<Uuid>, AppError> {
    let ids: Vec<String> = if let Some(states) = lobby_filters {
        let keys: Vec<String> = states
            .iter()
            .map(|state| RedisKey::lobbies_state(state))
            .collect();

        // Check if any of the state sets exist before trying union
        let mut existing_keys = Vec::new();
        for key in &keys {
            let exists: bool = redis::cmd("EXISTS")
                .arg(key)
                .query_async(&mut **conn)
                .await
                .map_err(AppError::RedisCommandError)?;
            if exists {
                existing_keys.push(key);
            }
        }

        // If no state sets exist, return empty
        if existing_keys.is_empty() {
            return Ok(Vec::new());
        }

        let union = RedisKey::temp_union();
        let _: () = redis::cmd("ZUNIONSTORE")
            .arg(&union)
            .arg(existing_keys.len())
            .arg(&existing_keys)
            .query_async(&mut **conn)
            .await
            .map_err(AppError::RedisCommandError)?;
        let _: Option<()> = redis::cmd("EXPIRE")
            .arg(&union)
            .arg(30)
            .query_async(&mut **conn)
            .await
            .ok();

        let out: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(&union)
            .arg(offset)
            .arg(end)
            .query_async(&mut **conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        // cleanup
        let _: Option<()> = redis::cmd("DEL")
            .arg(&union)
            .query_async(&mut **conn)
            .await
            .ok();
        out
    } else {
        // Check if "lobbies:all" exists before trying to access it
        let exists: bool = redis::cmd("EXISTS")
            .arg("lobbies:all")
            .query_async(&mut **conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        if !exists {
            return Ok(Vec::new());
        }

        redis::cmd("ZREVRANGE")
            .arg("lobbies:all")
            .arg(offset)
            .arg(end)
            .query_async(&mut **conn)
            .await
            .map_err(AppError::RedisCommandError)?
    };

    let mut uuids: Vec<Uuid> = ids
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();
    uuids.dedup();
    Ok(uuids)
}

pub async fn get_spectators(lobby_id: Uuid, redis: RedisClient) -> Result<Vec<Uuid>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let spectators_key = RedisKey::lobby_spectators(KeyPart::Id(lobby_id));
    let spectator_id_strings: Vec<String> = conn
        .smembers(&spectators_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    let spectator_ids: Vec<Uuid> = spectator_id_strings
        .into_iter()
        .filter_map(|id_str| Uuid::parse_str(&id_str).ok())
        .collect();

    Ok(spectator_ids)
}
