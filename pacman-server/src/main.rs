mod config;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use actix_web::{App, HttpResponse, HttpRequest, Json, Path, State};
use actix_web::http::Cookie;
use chrono::Duration;
use time::Duration as Dur;
use pacman_core::{contract, GameConfig, PacmanGame, RateLimit};
use structopt::StructOpt;
use crate::config::User;

#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<PacmanGame>>,
    users: Arc<[User]>,
    admin_token: Arc<str>,
    config: GameConfig,
}

impl AppState {
    fn is_password_correct(&self, user: &str, password: &str) -> bool {
        self.users.iter().any(|u| u.name == user && u.password == password)
    }
}

fn submit(state: State<AppState>, submit: Json<contract::Submit>, request: HttpRequest<AppState>) -> Json<contract::SubmitResponse> {
    let submit = submit.into_inner();
    let user_cookie = request.cookie("user");
    let password_cookie = request.cookie("password");
    let user = user_cookie
        .as_ref()
        .map(|c| c.value())
        .or(submit.user.as_ref().map(|s| s.as_str()))
        .unwrap_or("<missing>");
    let password = password_cookie
        .as_ref()
        .map(|c| c.value())
        .or(submit.password.as_ref().map(|s| s.as_str()))
        .unwrap_or("<missing>");
    log::debug!("POST /submit by {} (password {})", user, password);
    if !state.is_password_correct(user, password) {
        log::debug!("POST /submit by {} - unauthorized", user);
        return Json(contract::SubmitResponse::Unauthorized);
    }
    let mut game = state.game.lock().unwrap();
    let now = chrono::Utc::now();
    let result = game.submit_program(user, &submit.program, now);
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
    *game = PacmanGame::new(state.config.clone());
    HttpResponse::Ok().finish()
}

fn rate_limit(state: State<AppState>, limit: Json<contract::RateLimit>) -> HttpResponse {
    let limit = limit.into_inner();
    if limit.admin_token != state.admin_token.as_ref() {
        log::debug!("invalid admin token: {:?}", limit.admin_token);
        return HttpResponse::Unauthorized().finish();
    }
    if state.users.iter().any(|u| u.name == limit.user) {
        let mut game = state.game.lock().unwrap();
        game.rate_limit_user(&limit.user, RateLimit {
            count: limit.count as usize,
            window: Duration::seconds(i64::from(limit.window)),
        });
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

fn authenticate(state: State<AppState>, auth: Json<contract::Authenticate>) -> HttpResponse {
    let auth = auth.into_inner();
    if state.is_password_correct(&auth.user, &auth.password) {
        HttpResponse::Ok()
            .cookie(Cookie::build("user", auth.user).max_age(Dur::days(1)).finish())
            .cookie(Cookie::build("password", auth.password).max_age(Dur::days(1)).finish())
            .finish()
    } else {
        HttpResponse::Unauthorized().finish()
    }
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
    /// Max steps for user program (defaults to 100)
    #[structopt(long = "max-steps")]
    max_steps: Option<u64>,
    /// Max submissions allowed in rate limit window (defaults to 2)
    #[structopt(long = "rate-limit-count")]
    rate_limit_count: Option<usize>,
    /// Length of rate limit window (in seconds, defaults to 10)
    #[structopt(long = "rate-limit-window")]
    rate_limit_window: Option<u32>,
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

    let config = GameConfig {
        max_steps: opt.max_steps.unwrap_or(100),
        rate_limit: RateLimit {
            count: opt.rate_limit_count.unwrap_or(2),
            window: Duration::seconds(i64::from(opt.rate_limit_window.unwrap_or(10))),
        },
    };

    let state = AppState {
        game: Arc::new(Mutex::new(PacmanGame::new(config.clone()))),
        users: users.into(),
        admin_token: admin_token.into(),
        config,
    };

    let app_factory = move || App::with_state(state.clone())
        .prefix("/api")
        .resource("/submit", |r| r.post().with(submit))
        .resource("/authenticate", |r| r.post().with(authenticate))
        .resource("/submissions", |r| r.get().with(get_submissions))
        .resource("/submissions/{id}", |r| r.get().with(get_submission))
        .resource("/scoreboard", |r| r.get().with(scoreboard))
        .resource("/admin/level", |r| r.post().with(set_level))
        .resource("/admin/levelstate", |r| r.post().with(set_level_state))
        .resource("/admin/reset", |r| r.post().with(reset))
        .resource("/admin/ratelimit", |r| r.post().with(rate_limit));

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
