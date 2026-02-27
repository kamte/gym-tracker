use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::State,
    response::{Html, Redirect},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;

use crate::error::AppError;
use crate::handlers::AppState;
use crate::middleware::auth::create_token;
use crate::models::exercise::Exercise;
use crate::models::user::User;
use crate::templates;

#[derive(Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn register_form() -> Result<Html<String>, AppError> {
    let tmpl = templates::RegisterTemplate { error: None };
    Ok(Html(tmpl.to_string()))
}

pub async fn register(
    State(state): State<AppState>,
    axum::Form(form): axum::Form<RegisterForm>,
) -> Result<axum::response::Response, AppError> {
    // Validate input
    if form.username.len() < 3 || form.username.len() > 50 {
        let tmpl = templates::RegisterTemplate {
            error: Some("Username must be between 3 and 50 characters".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }
    if form.email.len() < 5 || !form.email.contains('@') {
        let tmpl = templates::RegisterTemplate {
            error: Some("Please enter a valid email address".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }
    if form.password.len() < 8 {
        let tmpl = templates::RegisterTemplate {
            error: Some("Password must be at least 8 characters".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }
    if form.password != form.password_confirm {
        let tmpl = templates::RegisterTemplate {
            error: Some("Passwords do not match".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    // Check if user exists
    if User::exists_by_username_or_email(&state.pool, &form.username, &form.email).await? {
        let tmpl = templates::RegisterTemplate {
            error: Some("Username or email already taken".to_string()),
        };
        return Ok(Html(tmpl.to_string()).into_response());
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(form.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))?
        .to_string();

    let user_id = User::create(&state.pool, &form.username, &form.email, &password_hash).await?;

    // Seed default exercises for the new user
    Exercise::seed_defaults(&state.pool, user_id).await?;

    Ok(Redirect::to("/login?message=Registration+successful.+Please+log+in.").into_response())
}

use axum::response::IntoResponse;

pub async fn login_form(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    let message = params.get("message").cloned();
    let tmpl = templates::LoginTemplate {
        error: None,
        message,
    };
    Ok(Html(tmpl.to_string()))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    axum::Form(form): axum::Form<LoginForm>,
) -> Result<(CookieJar, Redirect), axum::response::Response> {
    let user = User::find_by_username(&state.pool, &form.username)
        .await
        .map_err(|e| AppError::Database(e).into_response())?;

    let user = match user {
        Some(u) => u,
        None => {
            let tmpl = templates::LoginTemplate {
                error: Some("Invalid username or password".to_string()),
                message: None,
            };
            return Err(Html(tmpl.to_string()).into_response());
        }
    };

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("Invalid hash: {e}")).into_response())?;

    if Argon2::default()
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        let tmpl = templates::LoginTemplate {
            error: Some("Invalid username or password".to_string()),
            message: None,
        };
        return Err(Html(tmpl.to_string()).into_response());
    }

    let token = create_token(&state.jwt_secret, user.id, &user.username)
        .map_err(|e| e.into_response())?;

    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(true)
        .secure(state.cookie_secure)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .max_age(time::Duration::hours(24))
        .build();

    Ok((jar.add(cookie), Redirect::to("/")))
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Redirect) {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(0))
        .build();

    (jar.remove(cookie), Redirect::to("/login"))
}
