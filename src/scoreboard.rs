use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use std::error::Error;

// CREATE NEW SCORE ENTRY

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
struct ScoreInput {
    pub playername: String,
    pub score: String,
}

#[post("/scoreboard")] //call with params(Query): http://xxx/scoreboard?playername=Andi&score=10
async fn set_scoreboard(pool: web::Data<PgPool>, score: web::Query<ScoreInput>) -> impl Responder {
    match create_score(&score, &pool).await {
        Ok(message) => HttpResponse::Ok().body(message),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[post("/scoreboardform")] //call from form
async fn set_scoreboard_form(
    pool: web::Data<PgPool>,
    score: web::Form<ScoreInput>,
) -> impl Responder {
    match create_score(&score, &pool).await {
        Ok(message) => {
            HttpResponse::Ok()
                .append_header(("HX-Trigger", "new-score-saved")) // htmx event to trigger alpinejs data to fetch again (https://alexanderzeitler.com/articles/listening-to-htmx-hx-trigger-response-header-events-from-alpine-js/)
                .body(message)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

async fn create_score(
    score: &ScoreInput,
    pool: &sqlx::PgPool,
) -> Result<&'static str, Box<dyn Error>> {
    let parsed_score = score.score.parse::<i32>()?;
    let query = "INSERT INTO score VALUES ($1, $2)";

    sqlx::query(query)
        .bind(&score.playername)
        .bind(parsed_score)
        .execute(pool)
        .await?;

    Ok("Score saved successfully.")
}

// GET THE WHOLE SCOREBOARD (TOP 10 - all logic handled in PostgreSQL with views, triggers and functions - see psql_helper.txt)

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
struct Score {
    pub rank: i64,
    pub name: String,
    pub score: i32,
}

#[get("/scoreboard")]
pub async fn get_scoreboard(pool: web::Data<PgPool>) -> impl Responder {
    let q = "SELECT rank, player_name, score FROM ranked_scores";
    let rows = match sqlx::query(q).fetch_all(pool.get_ref()).await {
        Ok(rows) => rows,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    };

    let scores = rows
        .iter()
        .map(|row| Score {
            rank: row.get("rank"),
            name: row.get("player_name"),
            score: row.get("score"),
        })
        .collect::<Vec<Score>>();

    HttpResponse::Ok().json(scores)
}

// GET THE HIGHSCORE VIEW

#[derive(Serialize, Deserialize, FromRow)]
struct HighScore {
    player_name: String,
    score: i32,
}

#[get("/topscorer")]
pub async fn get_topscorer(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    let row: HighScore = sqlx::query_as("SELECT player_name, score FROM highscore_view")
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(row))
}
