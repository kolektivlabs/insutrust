use crate::web::{self, remove_token_cookie, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::routing::get;
use axum::{Json, Router};
use lib_auth::pwd::{self, ContentToHash, SchemeStatus};
use lib_core::ctx::Ctx;
use lib_core::model::user::{UserBmc, UserForLogin};
use lib_core::model::ModelManager;
use serde::Serialize;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::Cookies;
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/api/health", get(api_health_handler))
		
		.route("/api/login", post(api_login_handler))
		.route("/api/logout", post(api_logout_handler))
		.route("/api/register", post(api_register_handler))

		.route("/api/product/list", post(api_product_list_handler))
		.route("/api/product/detail", post(api_product_detail_handler))
		.with_state(mm)
}

async fn api_health_handler() -> Json<Value> {
	debug!("{:<12} - api_health_handler", "HANDLER");

	// Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	body
}

// region:    --- Login
async fn api_login_handler(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let LoginPayload {
		username,
		pwd: pwd_clear,
	} = payload;
	let root_ctx = Ctx::root_ctx();

	// -- Get the user.
	let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
		.await?
		.ok_or(Error::LoginFailUsernameNotFound)?;
	let user_id = user.id;

	// -- Validate the password.
	let Some(pwd) = user.pwd else {
		return Err(Error::LoginFailUserHasNoPwd { user_id });
	};

	let scheme_status = pwd::validate_pwd(
		ContentToHash {
			salt: user.pwd_salt,
			content: pwd_clear.clone(),
		},
		pwd,
	)
	.await
	.map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

	// -- Update password scheme if needed
	if let SchemeStatus::Outdated = scheme_status {
		debug!("pwd encrypt scheme outdated, upgrading.");
		UserBmc::update_pwd(&root_ctx, &mm, user.id, &pwd_clear).await?;
	}

	// -- Set web token.
	web::set_token_cookie(&cookies, &user.username, user.token_salt)?;

	// Create the success body.
	let body = Json(json!({
		"success": true
	}));

	Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
	username: String,
	pwd: String,
}
// endregion: --- Login

// region:    --- Logout
async fn api_logout_handler(
	cookies: Cookies,
	Json(payload): Json<LogoutPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_logout_handler", "HANDLER");
	let should_logout = payload.logout;

	if should_logout {
		remove_token_cookie(&cookies)?;
	}

	// Create the success body.
	let body = Json(json!({
		"success": should_logout
	}));

	Ok(body)
}

#[derive(Debug, Deserialize)]
struct LogoutPayload {
	logout: bool,
}
// endregion: --- Logout

// region:    --- Register
async fn api_register_handler(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	Json(payload): Json<RegisterPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_register_handler", "HANDLER");

	let RegisterPayload {
		email,
		phone,
		full_name,
		gender,
		birth_date,
		address,
		marital_status,
		occupation,
		income,
		dependents,
		region,
		familiarity,
		interests,
	} = payload;

	// Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}

#[derive(Debug, Deserialize)]
struct RegisterPayload {
	email: String,
	phone: String,
	full_name: String,
	gender: String,
	birth_date: String,
	address: String,
	marital_status: String,
	occupation: String,
	income: f64,
	dependents: u32,
	region: String,
	familiarity: String,
	interests: String,
}
// endregion: --- Register


// region:    --- Product List 
async fn api_product_list_handler(
    State(_mm): State<ModelManager>,
    _cookies: Cookies,
	Json(payload): Json<ProductListPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_register_handler", "HANDLER");

    // Create a vector to hold the dummy product data
    let mut rows = Vec::new();
    for i in 0..8 {
        rows.push(
			ProductResponse {
				id: (i + 1).to_string(),
				name: "Dummy Product".to_string(),
				desc: "This is a dummy product description.".to_string(),
				count_claim: 10,
				count_review: 20,
				rating: 4.0,
				company: "Dummy Company".to_string(),
				company_logo: "https://dummycompany.com/logo.png".to_string(),
				banner: "https://dummycompany.com/banner.png".to_string(),
			}
		);
    }

    // Create the success body with the dummy product data
	let response = ProductListResponse {
		success: true,
		rows,
	};
	let body = Json(serde_json::to_value(response)?);

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct ProductListPayload {
	search: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
struct ProductResponse {
	id: String,
	name: String,
	desc: String,
	count_claim: u32,
	count_review: u32,
	rating: f64,
	company: String,
	company_logo: String,
	banner: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductListResponse {
	success: bool,
	rows: Vec<ProductResponse>,
}
// endregion: --- Product List

// region:    --- Product Detail
async fn api_product_detail_handler(
	State(_mm): State<ModelManager>,
	_cookies: Cookies,
	Json(payload): Json<ProductDetailPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_product_detail_handler", "HANDLER");

	let ProductDetailPayload { id } = payload;

	// Create the success body with the dummy product data
	let response = ProductDetailResponse {
		success: true,
		id,
		name: "Dummy Product".to_string(),
		desc: "This is a dummy product description.".to_string(),
		count_claim: 10,
		count_review: 20,
		rating: 4.0,
		company: "Dummy Company".to_string(),
		company_logo: "https://dummycompany.com/logo.png".to_string(),
		banner: "https://dummycompany.com/banner.png".to_string(),
	};
	let body = Json(serde_json::to_value(response)?);

	Ok(body)
}

#[derive(Debug, Deserialize)]
struct ProductDetailPayload {
	id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductDetailResponse {
	success: bool,
	id: String,
	name: String,
	desc: String,
	count_claim: u32,
	count_review: u32,
	rating: f64,
	company: String,
	company_logo: String,
	banner: String,
}
// endregion: --- Product Detail