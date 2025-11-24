use actix_web::web;

use crate::handlers::ranking::get_global_ranking;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/rankings").route("/global", web::get().to(get_global_ranking)));
}
