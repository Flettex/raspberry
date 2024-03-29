use actix_web::{cookie::SameSite, HttpRequest, HttpResponse};

pub async fn get(req: HttpRequest) -> HttpResponse {
    if let Some(mut co) = req.cookie("auth-cookie") {
        co.set_same_site(SameSite::None);
        return HttpResponse::Ok().cookie(co).finish();
    }
    HttpResponse::Ok().finish()
}
