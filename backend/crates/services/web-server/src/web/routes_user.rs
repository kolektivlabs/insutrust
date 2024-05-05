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
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{Rng, thread_rng};

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/api/health", get(api_health_handler))
		
		.route("/api/login", post(api_login_handler))
		.route("/api/logout", post(api_logout_handler))
		.route("/api/register", post(api_register_handler))

		.route("/api/product/list", post(api_product_list_handler))
		.route("/api/product/detail", post(api_product_detail_handler))

		.route("/api/product/claim/historical_data/config", post(api_product_claim_historical_data_config_handler))
		.route("/api/product/claim/historical_data", post(api_product_claim_historical_data_handler))

		.route("/api/product/review/list", post(api_product_review_list_handler))
		.with_state(mm)
}

async fn api_health_handler() -> Result<Json<Value>> {
	debug!("{:<12} - api_health_handler", "HANDLER");

	// Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
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
    let mut data = Vec::new();
    for i in 0..8 {
        data.push(
			ProductResponse {
				id: (i + 1).to_string(),
				name: "Asuransi Kesehatan".to_string() + &i.to_string(),		
				description: "Dengan asuransi kesehatan, Anda akan mendapatkan perlindungan kesehatan yang lengkap.".to_string(),
				count_claim: 10,
				count_review: 20,
				rating: 4.0,
				company: CompanyInfo {
					name: "AXA Insurance".to_string(),
					logo: "https://upload.wikimedia.org/wikipedia/commons/thumb/9/94/AXA_Logo.svg/640px-AXA_Logo.svg.png".to_string(),
					description: "AXA adalah perusahaan asuransi multinasional yang berkantor pusat di Paris, Prancis.".to_string(),
					website_url: "https://www.axa.com".to_string(),
					location: "Paris, France".to_string(),
					founded_year: 1817,
				},
				banner: "https://umsu.ac.id/health/wp-content/uploads/2023/08/Hospitals-and-Insurance-750x375.jpeg".to_string(),
			}
		);
    }

    // Create the success body with the dummy product data
	let response = ProductListResponse {
		success: true,
		data,
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
	description: String,
	count_claim: u32,
	count_review: u32,
	rating: f64,
	company: CompanyInfo,
	banner: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductListResponse {
	success: bool,
	data: Vec<ProductResponse>,
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
		data: ProductDetail {
			id,
			name: "Asuransi Kesehatan".to_string(),
			description: "Dengan asuransi kesehatan, Anda akan mendapatkan perlindungan kesehatan yang lengkap.".to_string(),
			count_claim: 10,
			count_review: 20,
			rating: 4.0,
			company: CompanyInfo {
				name: "AXA Insurance".to_string(),
				logo: "https://upload.wikimedia.org/wikipedia/commons/thumb/9/94/AXA_Logo.svg/640px-AXA_Logo.svg.png".to_string(),
				description: "AXA adalah perusahaan asuransi multinasional yang berkantor pusat di Paris, Prancis.".to_string(),
				website_url: "https://www.axa.com".to_string(),
				location: "Paris, France".to_string(),
				founded_year: 1817,
			},
			banner: "https://umsu.ac.id/health/wp-content/uploads/2023/08/Hospitals-and-Insurance-750x375.jpeg".to_string(),
			categories: vec!["Health Insurance".to_string(), "Life Insurance".to_string()],
			review_visibility: [true, false][rand::random::<usize>() % 2],
			claim_historical_data_visibility: [true, false][rand::random::<usize>() % 2],
		},
	};
	let body = Json(serde_json::to_value(response)?);

	Ok(body)
}

// region:    --- Product Claim Historical Data Config
async fn api_product_claim_historical_data_config_handler(
	State(_mm): State<ModelManager>,
	_cookies: Cookies,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_product_claim_historical_data_config_handler", "HANDLER");

	// Create the success body with the dummy product data
	let response: ProductClaimHistoricalDataConfig = ProductClaimHistoricalDataConfig {
		success: true,
		resolutions: vec!["All time".to_string(), "1 year".to_string(), "3 year".to_string()],
	};
	let body = Json(serde_json::to_value(response)?);

	Ok(body)
}

// region:    --- Product Claim Historical Data
async fn api_product_claim_historical_data_handler(
	State(_mm): State<ModelManager>,
	_cookies: Cookies,
	Json(payload): Json<ProductClaimHistoricalDataPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_product_claim_historical_data_handler", "HANDLER");

	let ProductClaimHistoricalDataPayload { resolution } = payload;
	if ![ "All time", "1 year", "3 year"].contains(&resolution.as_str()) {
		let body = Json(json!({
			"result": {
				"success": false,
				"error": {
					"code": 400,
					"message": "Invalid resolution",
				},
			}
		}));
	
		return Ok(body);
	}

	let months = match resolution.as_str() {
		"All time" => 60,
		"1 year" => 12,
		"3 year" => 36,
		_ => 0,
	};

	// create list of timestamp_month
	let mut timestamp_month = Vec::new();
	let now = SystemTime::now();
	let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
	let timestamp = since_epoch.as_secs();

	for i in 0..months {
		timestamp_month.push(timestamp - i * 30 * 24 * 60 * 60 /* 30 days */);
	}

	// reverse the list
	timestamp_month.reverse();

	let mut rng = thread_rng();
	
	// Create the success body with the dummy product data, 5 years of monthly data, random volume
	let response = ProductClaimHistoricalDataResponse {
		success: true,
		data: timestamp_month.iter().map(|timestamp| ProductClaimHistoricalData {
			timestamp_month: *timestamp as u32,
			volume: rng.gen_range(0.0..500.0) as u32,
		}).collect(),
	};

	let body = Json(serde_json::to_value(response)?);
	return Ok(body);
}


// region:    --- Product Review List
async fn api_product_review_list_handler(
	State(_mm): State<ModelManager>,
	_cookies: Cookies,
	Json(payload): Json<ProductDetailPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_product_review_list_handler", "HANDLER");

	let ProductDetailPayload { id } = payload;

	// Create the success body with the dummy product data
	let response = ProductReviewListResponse {
		success: true,
		data: vec![
			ProductReview {
				id: "1".to_string(),
				user_name: "John Doe".to_string(),
				rating: 4.0,
				comment: "Great insurance!".to_string(),
				pros: "Good coverage".to_string(),
				cons: "Expensive".to_string(),
				created_at: 1630000000,
			},
			ProductReview {
				id: "2".to_string(),
				user_name: "Jane Doe".to_string(),
				rating: 3.0,
				comment: "Not bad".to_string(),
				pros: "Good coverage".to_string(),
				cons: "Expensive".to_string(),
				created_at: 1630000000,
			},
		],
	};
	let body = Json(serde_json::to_value(response)?);

	Ok(body)
}

#[derive(Debug, Deserialize)]
struct ProductDetailPayload {
	id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductDetail {
	id: String,
	name: String,
	description: String,
	count_claim: u32,
	count_review: u32,
	rating: f64,
	company: CompanyInfo,
	banner: String,
	categories: Vec<String>,
	review_visibility: bool,
	claim_historical_data_visibility: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductDetailResponse {
	success: bool,
	data: ProductDetail,
}
// endregion: --- Product Detail

#[derive(Debug, Serialize, Deserialize)]
struct CompanyInfo {
	name: String,
	logo: String,
	description: String,
	website_url: String,
	location: String,
	founded_year: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductClaimHistoricalDataPayload {
	resolution: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductClaimHistoricalDataResponse {
	data: Vec<ProductClaimHistoricalData>,
	success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductClaimHistoricalData {
	timestamp_month: u32,
	volume: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductClaimHistoricalDataConfig {
	resolutions: Vec<String>,
	success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductReview {
	id: String,
	user_name: String,
	rating: f64,
	comment: String,
	pros: String,
	cons: String,
	created_at: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductReviewData {
	success: bool,
	reviews: Vec<ProductReview>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductReviewListResponse {
	success: bool,
	data: Vec<ProductReview>,
}
