use dtos::DSErrorDto;
use rocket::{catch, serde::json::Json};

#[catch(404)]
pub async fn not_found() -> Json<DSErrorDto> {
    DSErrorDto::default_404()
}

#[catch(401)]
pub async fn unauthorized() -> Json<DSErrorDto> {
    DSErrorDto::default_401()
}
