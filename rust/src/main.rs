#![allow(unused)]
use std::{fmt::Display, net::SocketAddr, str::FromStr};

mod oauth;

use aide::{
    axum::ApiRouter,
    openapi::{Info, OpenApi},
    redoc::Redoc,
};
use anyhow::Context;
use axum::{
    Extension, Json,
    extract::{Path, Query},
    response::Response,
};
use clap::Parser;
use schemars::JsonSchema;
use sqlx::PgPool;

async fn serve_file(file_path: &std::path::Path) -> anyhow::Result<Response> {
    let file_contents = tokio::fs::read(file_path).await?;

    let response = Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(axum::body::Body::from(file_contents))?;

    Ok(response)
}

pub async fn open_api_json(Extension(api): Extension<OpenApi>) -> impl aide::axum::IntoApiResponse {
    dbg!(&api);
    Json(api)
}

#[derive(Debug, Clone, clap::Parser)]
struct Config {
    #[clap(long, default_value = "3000")]
    pub port: u16,
    //#[clap(long, short, env)]
    //pub database_url: String,
    #[clap(long, env, default_value = "info")]
    pub rust_log: String,

    #[clap(
        long,
        env,
        default_value = "postgres://postgres@localhost:5432/openfrontpro"
    )]
    pub database_url: String,

    #[clap(long, env)]
    pub useragent: Option<String>,

    #[clap(long, env)]
    pub cookie: Option<String>,

    #[clap(long, env, default_value = "https://openfront.io/api/public_lobbies")]
    pub openfront_lobby_url: String,

    #[clap(long, env, default_value = "./frontend")]
    pub frontend_folder: String,

    #[clap(long, env)]
    pub discord_client_id: Option<String>,

    #[clap(long, env)]
    pub discord_client_secret: Option<String>,

    #[clap(long, env, default_value = "http://localhost:3000/auth/discord/callback")]
    pub discord_redirect_uri: String,
}

// Example Response
//
// {"lobbies":[{"gameID":"8vpnPq5G","numClients":29,"gameConfig":{"gameMap":"Faroe Islands","gameType":"Public","difficulty":"Medium","disableNPCs":false,"infiniteGold":false,"infiniteTroops":false,"instantBuild":false,"gameMode":"Free For All","bots":400,"disabledUnits":[],"maxPlayers":40},"msUntilStart":13941}]}
// {"lobbies":[{"gameID":"U91rErJL","numClients":0,"gameConfig":{"gameMap":"Australia","gameType":"Public","difficulty":"Medium","disableNPCs":false,"infiniteGold":false,"infiniteTroops":false,"instantBuild":false,"gameMode":"Free For All","bots":400,"disabledUnits":[],"maxPlayers":50},"msUntilStart":59901}]}
// {"lobbies":[{"gameID":"PQaySEuD","numClients":33,"gameConfig":{"gameMap":"Gateway to the Atlantic","gameType":"Public","difficulty":"Medium","disableNPCs":true,"infiniteGold":false,"infiniteTroops":false,"instantBuild":false,"gameMode":"Team","bots":400,"disabledUnits":[],"maxPlayers":80,"playerTeams":4},"msUntilStart":45184}]}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct PublicLobbiesResponse {
    lobbies: Vec<Lobby>,
}

#[derive(Debug, Clone, serde::Serialize, JsonSchema)]
#[serde(tag = "group")]
enum PlayerTeams {
    FFA,
    Teams {
        num_teams: u8
    },
    Parties {
        party_size: u8
    },
}
// PlayerTeams fromStr:
impl PlayerTeams {
    fn from_str_or_int(s: &StringOrInt) -> Option<Self> {
        if let StringOrInt::String(f) = &s {
            return match f.as_ref() {
                "Duos" => Some(PlayerTeams::Parties { party_size: 2 }),
                "Trios" => Some(PlayerTeams::Parties { party_size: 3 }),
                "Quads" => Some(PlayerTeams::Parties { party_size: 4 }),
                _ => None
            };
        } else if let StringOrInt::Int(i) = s {
            return Some(PlayerTeams::Teams { num_teams: *i as u8 });
        }

        None
    }
}


struct PlayerTeamsVisitor;

impl<'de> serde::de::Visitor<'de> for PlayerTeamsVisitor {
    type Value = PlayerTeams;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer representing the number of teams or parties")
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PlayerTeams::from(value))
    }
}

impl<'d> serde::Deserialize<'d> for PlayerTeams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        deserializer.deserialize_i32(PlayerTeamsVisitor)
    }
}

impl From<i32> for PlayerTeams {
    fn from(num_teams: i32) -> Self {
        if num_teams == 0 {
            PlayerTeams::FFA
        } else if num_teams < 0 {
            PlayerTeams::Parties { party_size: -num_teams as u8 }
        } else {
            PlayerTeams::Teams { num_teams: num_teams as u8 }
        }
    }
}

impl From<PlayerTeams> for i32 {
    fn from(teams: PlayerTeams) -> Self {
        match teams {
            PlayerTeams::FFA => 0,
            PlayerTeams::Teams { num_teams } => num_teams as _,
            PlayerTeams::Parties { party_size } => -(party_size as i32),
        }
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for PlayerTeams {
    fn decode(
        value: sqlx::postgres::PgValueRef<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s: i32 = sqlx::decode::Decode::<sqlx::Postgres>::decode(value)?;
        Ok(PlayerTeams::from(s))
    }
}

impl Display for PlayerTeams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerTeams::FFA => write!(f, "FFA"),
            PlayerTeams::Teams { num_teams } => write!(f, "{} Teams", num_teams),
            PlayerTeams::Parties { party_size } => write!(f, "Parties of {}", party_size),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Lobby {
    #[serde(rename = "gameID")]
    game_id: String,
    num_clients: i32,
    game_config: GameConfig,
    ms_until_start: u64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
#[serde(untagged)]
enum StringOrInt {
    String(String),
    Int(i32),
}

impl ToString for StringOrInt {
    fn to_string(&self) -> String {
        match self {
            StringOrInt::String(s) => s.clone(),
            StringOrInt::Int(i) => i.to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GameConfig {
    game_map: String,
    game_type: String,
    difficulty: String,
    #[serde(rename = "disableNPCs")]
    disable_npcs: bool,
    infinite_gold: bool,
    infinite_troops: bool,
    instant_build: bool,
    game_mode: String,
    bots: i32,
    disabled_units: Vec<String>,
    max_players: i32,
    player_teams: Option<StringOrInt>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_parse_lobby() {
        let jsons = [
            r#"{"lobbies":[{"gameID":"mieEQtXo","numClients":1,"gameConfig":{"gameMap":"World","gameType":"Public","difficulty":"Medium","disableNPCs":false,"infiniteGold":false,"infiniteTroops":false,"instantBuild":false,"gameMode":"Free For All","bots":400,"disabledUnits":[],"maxPlayers":50},"msUntilStart":57198}]}"#,
            r#"{"lobbies":[{"gameID":"Q7GMHhAv","numClients":16,"gameConfig":{"gameMap":"Africa","gameType":"Public","difficulty":"Medium","disableNPCs":false,"infiniteGold":false,"infiniteTroops":false,"instantBuild":false,"gameMode":"Free For All","bots":400,"disabledUnits":[],"maxPlayers":80},"msUntilStart":31378}]}"#,
        ];

        for json in jsons {
            let response: PublicLobbiesResponse = serde_json::from_str(json).unwrap();
            assert!(
                !response.lobbies.is_empty(),
                "Lobby list should not be empty"
            );
            assert!(
                response.lobbies[0].ms_until_start > 0,
                "ms_until_start should be greater than 0"
            );
            assert!(
                !response.lobbies[0].game_config.game_map.is_empty(),
                "game_map should not be empty"
            );
        }
    }
}

use std::future::Future;

use serde::de::DeserializeOwned;

pub trait ReqwestErrorHandlingExtension
where
    Self: Sized + Send,
{
    fn anyhow_error_text(self) -> impl Future<Output = anyhow::Result<String>> + Send;

    fn anyhow_error_json<T: DeserializeOwned>(
        self,
    ) -> impl Future<Output = anyhow::Result<T>> + Send {
        async move {
            let text = self.anyhow_error_text().await?;
            Ok(serde_json::from_str(&text)?)
        }
    }
}

impl ReqwestErrorHandlingExtension for reqwest::Response {
    async fn anyhow_error_text(self) -> anyhow::Result<String> {
        let status = self.status();
        let url = self.url().to_string();
        let mut text = self.text().await?;

        if !status.is_success() {
            if let Ok(t) = serde_json::from_str::<serde_json::Value>(&text) {
                text = serde_json::to_string_pretty(&t).unwrap();
            }
            tracing::error!(text);
            anyhow::bail!(
                "API Call failed {:?} with code {}: {}",
                url,
                status.as_u16(),
                text
            );
        }

        Ok(text)
    }
}

async fn get_new_games(cfg: &Config) -> anyhow::Result<Vec<Lobby>> {
    let mut base = reqwest::Client::new().get(&cfg.openfront_lobby_url);
    if let Some(ref useragent) = cfg.useragent {
        base = base.header(reqwest::header::USER_AGENT, useragent);
    }
    if let Some(ref cookie) = cfg.cookie {
        base = base.header(reqwest::header::COOKIE, cookie);
    }

    let new_games = base.send().await?.anyhow_error_json::<PublicLobbiesResponse>().await?;

    Ok(new_games.lobbies)
}

/// This is put into the database for every lobby we see
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow, JsonSchema)]
struct LobbyDBEntry {
    game_id: String,
    teams: PlayerTeams,
    max_players: i32,
    game_map: String,
    approx_num_players: i32,
    /// Last seen timestamp in seconds
    first_seen_unix_sec: i64,
    /// Last seen timestamp in seconds
    last_seen_unix_sec: i64,
    completed: bool,
    lobby_config_json: serde_json::Value,
    analysis_complete: bool
}

/// This is put into the database for every lobby we see
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, sqlx::FromRow, JsonSchema)]
struct LobbyDBEntryNoConfig {
    game_id: String,
    teams: PlayerTeams,
    max_players: i32,
    game_map: String,
    approx_num_players: i32,
    /// Last seen timestamp in seconds
    first_seen_unix_sec: i64,
    /// Last seen timestamp in seconds
    last_seen_unix_sec: i64,
    completed: bool,
    analysis_complete: bool,
}

impl<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow> for LobbyDBEntryNoConfig {
    fn from_row(row: &'a sqlx::postgres::PgRow) -> Result<Self, sqlx_core::Error> {
        use sqlx::Row;
        let teams_val: i32 = row.try_get("teams")?;
        let teams = PlayerTeams::from(teams_val);

        Ok(LobbyDBEntryNoConfig {
            game_id: row.try_get("game_id")?,
            teams,
            max_players: row.try_get("max_players")?,
            game_map: row.try_get("game_map")?,
            approx_num_players: row.try_get("approx_num_players")?,
            first_seen_unix_sec: row.try_get("first_seen_unix_sec")?,
            last_seen_unix_sec: row.try_get("last_seen_unix_sec")?,
            completed: row.try_get("completed")?,
            analysis_complete: row.try_get("analysis_complete!")?,
        })
    }
}


impl LobbyDBEntry {
    pub fn lobby_config(&self) -> GameConfig {
        serde_json::from_value(self.lobby_config_json.clone())
            .expect("Invalid lobby config JSON in database")
    }
}

impl GameConfig {
    pub fn player_teams(&self) -> PlayerTeams {
        if let Some(ref teams) = self.player_teams {
            PlayerTeams::from_str_or_int(teams).unwrap()
        } else {
            PlayerTeams::FFA
        }
    }
}

fn now_unix_sec() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time before UNIX EPOCH")
        .as_secs() as i64
}

async fn look_for_new_games(database: PgPool, cfg: &Config) -> anyhow::Result<()> {
    let mut expected_to_be_new_game_next_check = true;
    let mut last_game_id = String::new();
    loop {
        let new_games = get_new_games(cfg).await?;
        let first = new_games.first().context("No new games found...")?;

        if first.game_id != last_game_id {
            tracing::info!(
                expected_to_be_new_game_next_check,
                "New game found: {}",
                first.game_id
            );
            if !expected_to_be_new_game_next_check {
                // We got a new game earlier than expected. The last one must have been full.
                sqlx::query!(
                    "UPDATE lobbies SET approx_num_players = max_players WHERE game_id = $1",
                    last_game_id
                )
                .execute(&database)
                .await?;
            }
            last_game_id = first.game_id.clone();
        }

        let player_teams_as_int: i32 = first.game_config.player_teams().into();

        sqlx::query!(
            "INSERT INTO
                lobbies (game_id, teams, max_players, game_map, approx_num_players, first_seen_unix_sec, last_seen_unix_sec, lobby_config_json)
            VALUES
                ($1, $2, $3, $4, $5, $6, $6, $7)
            ON CONFLICT (game_id)
            DO UPDATE
                SET approx_num_players = $5
                , last_seen_unix_sec = $6
            ",
            first.game_id,
            player_teams_as_int,
            first.game_config.max_players,
            first.game_config.game_map,
            first.num_clients,
            now_unix_sec(),
            serde_json::to_value(&first.game_config).unwrap()
        ).execute(&database).await?;

        let num_players_left = (first.game_config.max_players - first.num_clients).max(0);

        // Wait between 3 and 15 seconds before checking again.
        let next_time = (first.ms_until_start)
            .min(15500)
            .min(num_players_left as u64 * 1000)
            .max(3500)
            - 500;

        expected_to_be_new_game_next_check = next_time > first.ms_until_start;

        tracing::info!(
            "Lobby {} {} ({}) has {}/{} players. Starts in {}ms. Next check in {}ms.",
            first.game_id,
            first.game_config.game_map,
            first.game_config.player_teams(),
            first.num_clients,
            first.game_config.max_players,
            first.ms_until_start,
            next_time

        );
        tokio::time::sleep(tokio::time::Duration::from_millis(next_time)).await;
    }
}

async fn check_if_game_finished(game_id: &str) -> anyhow::Result<(serde_json::Value, bool)> {
    let finished = reqwest::get(format!("https://api.openfront.io/game/{}", game_id))
        .await?
        .json::<serde_json::Value>()
        .await?;

    if finished.get("error").is_some() {
        if finished["error"] == "Not found" {
            return Ok((finished, false));
        }
    }

    if finished.get("gitCommit").is_some() {
        // Game is finished!
        let winning_id = finished["info"]["winner"][1].as_str();
        tracing::info!(winning_id, game_id, "Game is finished.");

        for player in finished["info"]["players"].as_array().unwrap() {
            if player["clientID"].as_str() == winning_id {
                tracing::info!("Winning player: {}", player["username"]);
            }
        }

        return Ok((finished, true));
    }

    tracing::error!("Game {} is in an unknown other state.", game_id);

    anyhow::bail!("Game {} is in an unknown state: {:?}", game_id, finished);
}

async fn look_for_finished_games(database: PgPool) -> anyhow::Result<()> {
    loop {
        let unfinished_games = sqlx::query!(
            "SELECT
                game_id
            FROM lobbies
            WHERE
                completed = false
                AND last_seen_unix_sec < extract(epoch from (NOW() - INTERVAL '15 minutes'))
                -- AND last_seen_unix_sec > extract(epoch from (NOW() - INTERVAL '2 hours'))
            "
        ).fetch_all(&database).await?;

        tracing::info!(
            "Found {} unfinished games, checking if they are finished...",
            unfinished_games.len()
        );

        for game in unfinished_games {
            let game_id = &game.game_id;
            let (result_json, finished) = check_if_game_finished(game_id).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            if !finished {
                tracing::info!("Game {} is still unfinished.", game_id);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }

            let mut txn = database.begin().await?;
            sqlx::query!(
                "UPDATE lobbies SET completed = true WHERE game_id = $1",
                game_id
            )
            .execute(&mut *txn)
            .await?;

            sqlx::query!(
                "INSERT INTO finished_games (game_id, result_json) VALUES ($1, $2)",
                game_id,
                result_json
            )
            .execute(&mut *txn)
            .await?;

            txn.commit().await?;

            let dur_secs = result_json["info"]["duration"].as_i64().unwrap_or(0);
            let num_turns = result_json["info"]["num_turns"].as_i64().unwrap_or(0);
            tracing::info!(
                dur_secs,
                num_turns,
                "Game {} is finished. Adding results to db.",
                game_id
            );
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 5)).await;
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
struct LobbyQueryParams {
    completed: Option<bool>,
    game_map: Option<String>,
    /// Unix timestamp in seconds
    after: Option<i64>,
    /// Unix timestamp in seconds
    before: Option<i64>,

}
async fn lobbies_id_handler(
    Extension(database): Extension<PgPool>,
    Query(params): Query<LobbyQueryParams>,
    Path(id): Path<String>,
) -> Result<Json<LobbyDBEntry>, Response> {
    let d = sqlx::query_as!(
        LobbyDBEntry,
        r#"SELECT
            lo.*,
            (co.inserted_at_unix_sec IS NOT NULL) AS "analysis_complete!"
        FROM
            lobbies lo
            LEFT JOIN analysis_1.completed_analysis co
            ON lo.game_id = co.game_id
        WHERE lo.game_id = $1"#,
        id
    );

    let lobby = d
        .fetch_one(&database)
        .await
        .map_err(|e| {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body(axum::body::Body::from(format!("Lobby not found: {}", e)))
                .expect("Failed to build response for error message")
        })?;

    Ok(Json(lobby))
}

async fn lobbies_handler(
    Extension(database): Extension<PgPool>,
    Query(params): Query<LobbyQueryParams>,
) -> Result<Json<Vec<LobbyDBEntryNoConfig>>, Response> {
    let mut querybuilder = sqlx::query_builder::QueryBuilder::new(
        r#"
        SELECT
            lo.game_id, lo.teams, lo.max_players, lo.game_map, lo.approx_num_players,
            lo.first_seen_unix_sec, lo.last_seen_unix_sec, lo.completed,
            (co.inserted_at_unix_sec IS NOT NULL) AS "analysis_complete!"
        FROM
            public.lobbies lo
            LEFT JOIN analysis_1.completed_analysis co
            ON lo.game_id = co.game_id
        "#,
    );

    let mut _has_where = false;

    if let Some(completed) = params.completed {
        if _has_where {
            querybuilder.push(" AND ");
        } else {
            querybuilder.push(" WHERE ");
        }
        _has_where = true;

        querybuilder.push(" completed = ");
        querybuilder.push_bind(completed);
    }

    if let Some(ref before) = params.before {
        if _has_where {
            querybuilder.push(" AND ");
        } else {
            querybuilder.push(" WHERE ");
        }
        _has_where = true;

        querybuilder.push("last_seen_unix_sec < ");
        querybuilder.push_bind(before);
    }

    if let Some(ref after) = params.after {
        if _has_where {
            querybuilder.push(" AND ");
        } else {
            querybuilder.push(" WHERE ");
        }
        _has_where = true;

        querybuilder.push("first_seen_unix_sec > ");
        querybuilder.push_bind(after);
    }

    if let Some(ref game_map) = params.game_map {
        if _has_where {
            querybuilder.push(" AND ");
        } else {
            querybuilder.push(" WHERE ");
        }
        _has_where = true;

        querybuilder.push("game_map = ");
        querybuilder.push_bind(game_map);
    }

    querybuilder.push(" ORDER BY last_seen_unix_sec DESC LIMIT 100");

    let res: Vec<LobbyDBEntryNoConfig> = querybuilder
        .build_query_as()
        .fetch_all(&database)
        .await
        .map_err(|e| {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::from(format!(
                    "Database query failed: {}",
                    e
                )))
                .expect("Failed to build response for error message")
        })?;

    Ok(Json(res))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
struct FinshedGameDBEntry {
    game_id: String,
    result_json: serde_json::Value,
    inserted_at_unix_sec: i64,
}

async fn game_handler(
    Extension(database): Extension<PgPool>,
    Path(game_id): Path<String>,
) -> Result<Json<serde_json::Value>, Response> {
    let lobby = sqlx::query_as!(
        FinshedGameDBEntry,
        "SELECT game_id, result_json, inserted_at_unix_sec FROM finished_games WHERE game_id = $1",
        game_id
    )
    .fetch_one(&database)
    .await
    .map_err(|e| {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .body(axum::body::Body::from(format!("Lobby not found: {}", e)))
            .expect("Failed to build response for error message")
    })?;

    Ok(Json(lobby.result_json))
}

//CREATE TABLE public.analysis_queue (
    //game_id CHAR(8) NOT NULL,
    //requesting_user_id CHAR(10),
    //requested_unix_sec BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    //started_unix_sec BIGINT,
    //status analysis_queue_status NOT NULL DEFAULT 'Pending',
    //FOREIGN KEY (requesting_user_id) REFERENCES social.registered_users(id) ON DELETE CASCADE
//);
//
//CREATE TYPE analysis_queue_status AS ENUM (
    //'Pending',
    //'Running',
    //'Completed',
    //'Failed',
    //'Stalled',
    //'Cancelled',
    //'CompletedAlready'
//);
//
// On analyze call: insert
// On delete call, set status to cancelled
// On analyze call, if already in queue for your user, return 409 Conflict
//

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema, sqlx::Type)]
#[sqlx(type_name = "analysis_queue_status")]
enum AnalysisQueueStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Stalled,
    Cancelled,
    CompletedAlready,
}

async fn game_analyze_handler(
    Extension(database): Extension<PgPool>,
    Path(game_id): Path<String>,
) -> Result<(), Response> {
    //Insert into analysis_queue
    let res = sqlx::query!(
        "INSERT INTO analysis_queue (game_id)
         VALUES ($1)
         ON CONFLICT (game_id) DO NOTHING",
        game_id,
    ).execute(&database).await;

    match res {
        Ok(_) => {
            Ok(())
        },
        Err(e) => Err(axum::response::Response::builder()
            .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(format!("Failed to queue analysis: {}", e)))
            .expect("Failed to build response for error message")),
    }
}

async fn game_analyze_handler_delete(
    Extension(database): Extension<PgPool>,
    Path(game_id): Path<String>,
) -> Result<(), Response> {
    // Set status to cancelled
    let res = sqlx::query!(
        "UPDATE analysis_queue SET status = 'Cancelled' WHERE game_id = $1",
        game_id,
    ).execute(&database).await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(axum::response::Response::builder()
            .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(format!("Failed to cancel analysis: {}", e)))
            .expect("Failed to build response for error message")),
    }
}




#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let config = std::sync::Arc::new(config);

    let database = PgPool::connect(&config.database_url)
        .await
        .context("Failed to connect to the database")?;

    sqlx::migrate!("./migrations")
        .run(&database)
        .await?;

    tracing::info!("Migrations applied successfully");

    tracing_subscriber::fmt()
        //.with_max_level(tracing::Level::INFO)
        .with_env_filter(&config.rust_log)
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(vec![
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any);

    let api_routes = ApiRouter::new()
        .route("/lobbies", axum::routing::get(lobbies_handler))
        .route("/lobbies/{id}", axum::routing::get(lobbies_id_handler))
        .api_route("/games/{game_id}", aide::axum::routing::get(game_handler))
        .route("/games/{game_id}/analyze", axum::routing::get(game_analyze_handler).delete(game_analyze_handler_delete));
        //.route("/games/{game_id}/analyze", axum::routing::get(game_analyze_handler).delete(game_analyze_handler_delete))


    let routes = ApiRouter::new()
        .route("/health", axum::routing::get(|| async { "ok!" }))
        .nest("/api/v1/", api_routes)
        .route("/openapi.json", axum::routing::get(open_api_json))
        .route("/redoc", Redoc::new("/openapi.json").axum_route());

    let mut openapi = OpenApi {
        info: Info {
            title: "openfront.pro".to_string(),
            version: "1.0.0".to_string(),
            description: Some(
                "This API can be used to access elo data, match data, and more".to_string(),
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    let missing_html = format!("{}/404.html", config.frontend_folder);

    let fin = routes
        .finish_api(&mut openapi)
        .layer(Extension(openapi.clone()))
        .layer(Extension(database.clone()))
        //.layer(NormalizePathLayer::trim_trailing_slash())
        .layer(cors)
        .fallback_service(axum::routing::get_service(
            tower_http::services::ServeDir::new(&*config.frontend_folder)
                .append_index_html_on_directories(true)
                .not_found_service(axum::routing::get(|| async move {
                    serve_file(&std::path::Path::new(&missing_html))
                        .await
                        .unwrap()
                })),
        ));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;

    tracing::info!("Listening on http://{}", listener.local_addr()?);

    let db = database.clone();
    let cfg = config.clone();
    tokio::spawn(async move {
        let mut backoff = 0;
        loop {
            if let Err(e) = look_for_new_games(db.clone(), &cfg).await {
                tracing::error!("Error looking for new games: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5 + backoff.min(11) * 5)).await;
                backoff += 1;
            } else {
                backoff = 0;
            }
        }
    });

    let db = database.clone();
    tokio::spawn(async move {
        let mut backoff = 0;
        loop {
            if let Err(e) = look_for_finished_games(db.clone()).await {
                tracing::error!("Error looking for finished games: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5 + backoff.min(11) * 5)).await;
                backoff += 1;
            } else {
                backoff = 0;
            }
        }
    });

    axum::serve(
        listener,
        fin.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    anyhow::bail!("Server stopped unexpectedly");
}
