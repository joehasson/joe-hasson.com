pub fn e500<T>(e: T) -> actix_web::Error 
where T: 'static + std::fmt::Debug + std::fmt::Display
{
    actix_web::error::ErrorInternalServerError(e)
}
