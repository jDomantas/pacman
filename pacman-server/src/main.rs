mod config;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use actix_web::{App, HttpResponse, Json, Path, State};
use chrono::Duration;
use pacman_core::{contract, GameConfig, PacmanGame};
use structopt::StructOpt;
use crate::config::User;

#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<PacmanGame>>,
    users: Arc<[User]>,
    admin_token: Arc<str>,
}

impl AppState {
    fn is_password_correct(&self, user: &str, password: &str) -> bool {
        self.users.iter().any(|u| u.name == user && u.password == password)
    }
}

fn submit(state: State<AppState>, submit: Json<contract::Submit>) -> Json<contract::SubmitResponse> {
    let submit = submit.into_inner();
    log::debug!("POST /submit by {} (password {})", submit.user, submit.password);
    if !state.is_password_correct(&submit.user, &submit.password) {
        log::debug!("POST /submit by {} - unauthorized", submit.user);
        return Json(contract::SubmitResponse::Unauthorized);
    }
    let mut game = state.game.lock().unwrap();
    let now = chrono::Utc::now();
    let result = game.submit_program(&submit.user, &submit.program, now);
    Json(result)
}

fn get_submissions(state: State<AppState>) -> Json<contract::Submissions> {
    let game = state.game.lock().unwrap();
    let submissions = game.all_submissions();
    Json(submissions)
}

fn get_submission(state: State<AppState>, id: Path<u64>) -> HttpResponse {
    let game = state.game.lock().unwrap();
    let details = game.submission_details(id.into_inner());
    match details {
        Some(details) => HttpResponse::Ok().json(details),
        None => HttpResponse::NotFound().finish(),
    }
}

fn scoreboard(state: State<AppState>) -> Json<contract::Scoreboards> {
    let game = state.game.lock().unwrap();
    let scoreboards = game.get_scores();
    Json(scoreboards)
}

fn set_level(state: State<AppState>, set: Json<contract::SetLevel>) -> HttpResponse {
    let set = set.into_inner();
    if set.admin_token != state.admin_token.as_ref() {
        log::debug!("invalid admin token: {:?}", set.admin_token);
        return HttpResponse::Unauthorized().finish();
    }
    let mut game = state.game.lock().unwrap();
    let now = chrono::Utc::now();
    game.set_level(set.level, now);
    HttpResponse::Ok().finish()
}

fn set_level_state(state: State<AppState>, set: Json<contract::SetLevelState>) -> HttpResponse {
    let set = set.into_inner();
    if set.admin_token != state.admin_token.as_ref() {
        log::debug!("invalid admin token: {:?}", set.admin_token);
        return HttpResponse::Unauthorized().finish();
    }
    let mut game = state.game.lock().unwrap();
    game.set_level_state(set.is_closed);
    HttpResponse::Ok().finish()
}

fn reset(state: State<AppState>, reset: Json<contract::Reset>) -> HttpResponse {
    let reset = reset.into_inner();
    if reset.admin_token != state.admin_token.as_ref() {
        log::debug!("invalid admin token: {:?}", reset.admin_token);
        return HttpResponse::Unauthorized().finish();
    }
    let mut game = match state.game.lock() {
        Ok(game) => game,
        Err(poisoned) => poisoned.into_inner(),
    };
    *game = PacmanGame::new(game_config());
    HttpResponse::Ok().finish()
}

#[derive(StructOpt)]
struct Opt {
    /// Verbose logging
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
    /// Listen port (defaults to 8000)
    #[structopt(short = "p", long = "port")]
    port: Option<u16>,
    /// Path to a file containing list of user credentials
    #[structopt(long = "users", parse(from_os_str))]
    users: Option<PathBuf>,
    /// Admin token (defaults to "admin")
    #[structopt(long = "admin")]
    admin_token: Option<String>,
}

fn game_config() -> GameConfig {
    GameConfig {
        max_steps: 100,
        rate_limit_count: 2,
        rate_limit_window: Duration::seconds(10),
    }
}

fn main() {
    let opt = Opt::from_args();
    setup_logger(opt.verbose);

    let actor_system = actix::System::new("pacman-server");

    let admin_token = opt.admin_token.as_ref().map(String::as_ref).unwrap_or("admin");

    let users = if let Some(path) = &opt.users {
        match config::read_from_file(path) {
            Ok(users) => users,
            Err(e) => {
                log::error!("failed to read user file: {}", e);
                return;
            }
        }
    } else {
        const DEFAULT_USER_NAME: &str = "labas";
        const DEFAULT_USER_PASSWORD: &str = "rytas";
        log::info!("no user file given, adding a default user:");
        log::info!("  name: {}, password: {}", DEFAULT_USER_NAME, DEFAULT_USER_PASSWORD);
        vec![
            User {
                name: DEFAULT_USER_NAME.to_owned(),
                password: DEFAULT_USER_PASSWORD.to_owned(),
            },
        ]
    };

    let state = AppState {
        game: Arc::new(Mutex::new(PacmanGame::new(game_config()))),
        users: users.into(),
        admin_token: admin_token.into(),
    };

    let app_factory = move || App::with_state(state.clone())
        .prefix("/api")
        .resource("/submit", |r| r.post().with(submit))
        .resource("/submissions", |r| r.get().with(get_submissions))
        .resource("/submissions/{id}", |r| r.get().with(get_submission))
        .resource("/scoreboard", |r| r.get().with(scoreboard))
        .resource("/admin/level", |r| r.post().with(set_level))
        .resource("/admin/levelstate", |r| r.post().with(set_level_state))
        .resource("/admin/reset", |r| r.post().with(reset));

    let port = opt.port.unwrap_or(8000);
    let listen_on = &format!("0.0.0.0:{}", port);

    actix_web::server::HttpServer::new(app_factory)
        .bind(listen_on)
        .expect("failed to bind actix server to address")
        .start();
    
    log::info!("Server listening on {}", listen_on);

    let _ = actor_system.run();
}

fn setup_logger(verbose: bool) {
    let filter = if verbose {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Utc::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("pacman_server", filter)
        .chain(std::io::stderr())
        .apply()
        .expect("failed to setup logger");
}
