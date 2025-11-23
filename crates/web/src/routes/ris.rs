use actix_web::web;

use crate::handlers::ris::{
    compute_ris, get_current_formula, get_formula_by_year, get_participant_ris_history,
    list_ris_formulas, recompute_all_ris,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ris")
            .route("/formulas", web::get().to(list_ris_formulas))
            .route("/formulas/current", web::get().to(get_current_formula))
            .route("/formulas/{year}", web::get().to(get_formula_by_year))
            .route("/compute", web::post().to(compute_ris)),
    )
    .service(web::scope("/participants").route(
        "/{participant_id}/ris-history",
        web::get().to(get_participant_ris_history),
    ))
    .service(web::scope("/admin/ris").route("/recompute-all", web::post().to(recompute_all_ris)));
}
