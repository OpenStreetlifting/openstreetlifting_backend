use actix_web::web;

pub mod athletes;
pub mod competitions;
pub mod ranking;
pub mod ris;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(competitions::configure)
            .configure(athletes::configure)
            .configure(ranking::configure)
            .configure(ris::configure),
    );
}
