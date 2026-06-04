use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware};
use actix_web::ResponseError as _;

use crate::{
    handlers::{auth, formats, health, jobs, progress, upload},
    state::AppState,
};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Health
        .route("/health", web::get().to(health::health))
        .route("/ready", web::get().to(health::readiness))

        // Auth
        .service(
            web::scope("/auth")
                .route("/register", web::post().to(auth::register))
                .route("/login", web::post().to(auth::login))
                .route("/refresh", web::post().to(auth::refresh))
                .route("/logout", web::post().to(auth::logout))
                .route("/me", web::get().to(auth::me))
        )

        // API v1
        .service(
            web::scope("/v1")
                // Formats
                .route("/formats", web::get().to(formats::list_formats))
                .route("/formats/graph", web::get().to(formats::conversion_graph))

                // Upload
                .route("/upload", web::post().to(upload::upload_file))

                // Jobs
                .service(
                    web::scope("/jobs")
                        .route("", web::post().to(jobs::create_job))
                        .route("", web::get().to(jobs::list_jobs))
                        .route("/{id}", web::get().to(jobs::get_job))
                        .route("/{id}", web::delete().to(jobs::cancel_job))
                        .route("/{id}/download", web::get().to(jobs::download_job))
                        .route("/{id}/progress", web::get().to(progress::job_progress_sse))
                )
        );
}

pub async fn create_server(
    state: AppState,
    host: String,
    port: u16,
    cors_origin: String,
) -> std::io::Result<()> {
    let state = web::Data::new(state);

    tracing::info!("Starting UMC API on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .expose_headers(vec!["Content-Disposition"])
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(10 * 1024 * 1024) // 10 MiB JSON limit
                    .error_handler(|err, _| {
                        let response = crate::errors::ApiError::BadRequest(err.to_string())
                            .error_response();
                        actix_web::error::InternalError::from_response(err, response).into()
                    }),
            )
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
    })
    .workers(num_cpus::get())
    .bind((host.as_str(), port))?
    .run()
    .await
}
