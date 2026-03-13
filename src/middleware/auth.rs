use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, StatusCode, Request},
    middleware::Next,
    response::Response,
    body::Body,
};
use crate::utils::auth::verify_token;
use crate::permissions::UserClaims;

pub struct AuthMiddleware(pub crate::utils::auth::Claims);

pub struct OptionalAuthMiddleware(pub Option<crate::utils::auth::Claims>);

pub struct UserClaimsExtractor(pub UserClaims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthMiddleware
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION)
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        let auth_str = auth_header.to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid authorization header"))?;

        if auth_str.starts_with("Bearer ") {
            let token = &auth_str[7..];
            match verify_token(token) {
                Ok(claims) => Ok(AuthMiddleware(claims)),
                Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid token")),
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid authorization scheme"))
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthMiddleware
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION);
        
        if let Some(auth_header) = auth_header {
            let auth_str = auth_header.to_str()
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid authorization header"))?;

            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                match verify_token(token) {
                    Ok(claims) => Ok(OptionalAuthMiddleware(Some(claims))),
                    Err(_) => Ok(OptionalAuthMiddleware(None)),
                }
            } else {
                Ok(OptionalAuthMiddleware(None))
            }
        } else {
            Ok(OptionalAuthMiddleware(None))
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserClaimsExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION)
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        let auth_str = auth_header.to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid authorization header"))?;

        if auth_str.starts_with("Bearer ") {
            let token = &auth_str[7..];
            match verify_token(token) {
                Ok(claims) => Ok(UserClaimsExtractor(UserClaims::new(claims.sub, &claims.role))),
                Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid token")),
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid authorization scheme"))
        }
    }
}

pub async fn auth(
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    let auth_header = req.headers()
        .get(AUTHORIZATION)
        .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

    let auth_str = auth_header.to_str()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid authorization header"))?;

    if auth_str.starts_with("Bearer ") {
        let token = &auth_str[7..];
        if verify_token(token).is_ok() {
            Ok(next.run(req).await)
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid token"))
        }
    } else {
        Err((StatusCode::UNAUTHORIZED, "Invalid authorization scheme"))
    }
}
