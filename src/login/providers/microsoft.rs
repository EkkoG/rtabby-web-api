use async_trait::async_trait;
use crate::{login::providers::{LoginProvider, ThirdPartyUserInfo}, login::tools};
use actix_web::Error;
use crate::login::env;
use crate::env as app_;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    id: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

pub struct Microsoft;

#[async_trait]
impl LoginProvider for Microsoft {
    fn name(&self) -> String {
        String::from("Microsoft")
    }

    fn login_url(&self, host: String, state: String) -> String {
        let client_id = app_::var(env::ENV_MICROSOFT_APP_CLIENT_ID).expect("Missing MICROSOFT_APP_CLIENT_ID env var");
        let params = vec![
            ("client_id", client_id),
            ("state", state),
            ("response_type", "code".to_string()),
            ("scope", "https://graph.microsoft.com/User.Read".to_string()),
            ("redirect_uri", format!("{}://{}/login/microsoft/callback", tools::scheme(), host)),
        ];
        reqwest::Url::parse_with_params("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize", params).unwrap().to_string()
    }

    async fn user_info(&self, host: String, code: String) -> Result<ThirdPartyUserInfo, Error> {
        let client_id = app_::var(env::ENV_MICROSOFT_APP_CLIENT_ID).expect("Missing MICROSOFT_APP_CLIENT_ID env var");
        let client_secret = app_::var(env::ENV_MICROSOFT_APP_CLIENT_SECRET).expect("Missing MICROSOFT_APP_CLIENT_SECRET env var");
        let redirect_uri = format!("{}://{}/login/microsoft/callback", tools::scheme(), host);
        let token = Self.get_access_token("https://login.microsoftonline.com/consumers/oauth2/v2.0/token".to_string(), code, client_id, client_secret, "authorization_code".to_string(), Some(redirect_uri)).await.unwrap();
        let user_info = Self.get_user_info("https://graph.microsoft.com/v1.0/me", token).await.unwrap().json::<UserInfo>().await.unwrap();
        Ok(ThirdPartyUserInfo {
            id: user_info.id.to_string(),
            name: user_info.display_name,
            platform: self.name().to_lowercase(),
        })
    }
}