use reqwest;
use serde_json;
use url::Url;

use super::account;
use crate::configuration::Partition;

const AWS_DOMAIN: &str = "aws.amazon.com";

#[derive(serde::Serialize, Debug)]
struct SignInTokenRequest {
    #[serde(rename = "Action")]
    action: String,
    #[serde(rename = "Session")]
    session: SignInTokenRequestSession,
}

#[derive(serde::Serialize, Debug)]
struct SignInTokenRequestSession {
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "sessionKey")]
    session_key: String,
    #[serde(rename = "sessionToken")]
    session_token: String,
}

#[derive(serde::Deserialize, Debug)]
struct SignInTokenResponse {
    #[serde(rename = "SigninToken")]
    token: String,
}

pub async fn get_console_url(
    account_id: String,
    role_name: String,
    partition: Partition,
    app: tauri::AppHandle,
) -> String {
    // Create a signed URL for AWS console
    // https://docs.aws.amazon.com/IAM/latest/UserGuide/example_sts_Scenario_ConstructFederatedUrl_section.html
    let credentials = account::get_credentials(partition, role_name, account_id, app).await;
    let token = SignInTokenRequestSession {
        session_id: credentials.access_key_id,
        session_key: credentials.secret_access_key,
        session_token: credentials.session_token,
    };
    let mut target =
        Url::parse(format!("https://signin.{}/federation", AWS_DOMAIN).as_str()).unwrap();
    target
        .query_pairs_mut()
        .append_pair("Action", "getSigninToken")
        .append_pair("Session", serde_json::to_string(&token).unwrap().as_str())
        .finish();
    log::debug!("Request to send for signin token: {}", target.to_string());

    let console_token = reqwest::get(target.to_string())
        .await
        .unwrap()
        .json::<SignInTokenResponse>()
        .await
        .unwrap();

    let mut console =
        Url::parse(format!("https://signin.{}/federation", AWS_DOMAIN).as_str()).unwrap();
    console
        .query_pairs_mut()
        .append_pair("Action", "login")
        .append_pair(
            "Destination",
            format!(
                "https://{}.console.{}/console/home",
                "us-west-2", AWS_DOMAIN
            )
            .as_str(),
        )
        .append_pair("SigninToken", console_token.token.as_str())
        .finish();
    log::debug!("Finished console URL: {}", console.to_string());
    console.to_string()
}
