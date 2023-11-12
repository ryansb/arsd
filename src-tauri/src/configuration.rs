use awsregion::Region;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub path: PathBuf,
    pub partitions: Vec<Partition>,
    pub aliases: Aliases,
}
impl Settings {
    pub fn partition(&self, p: String) -> Option<Partition> {
        for candidate in self.partitions.iter() {
            if candidate.slug() == p {
                return Some(candidate.clone());
            }
        }
        None
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Aliases {
    accounts: HashMap<String, String>,
    roles: HashMap<String, String>,
}
impl Aliases {
    pub fn map_role(&self, role_name: String) -> String {
        match self.roles.get(&role_name) {
            Some(a) => a.to_owned(),
            None => role_name,
        }
    }
    pub fn map_account(&self, account_email: String) -> Option<String> {
        self.accounts.get(&account_email).cloned()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub struct Partition {
    pub start_url: String,
    pub account_id: Option<String>,
    pub region: String,
}
impl Partition {
    pub fn scopes(&self) -> Vec<String> {
        vec![String::from("sso:account:access")]
    }
    pub fn sso_region(&self) -> Region {
        Region::from_str(&self.region).unwrap()
    }
    pub fn slug(&self) -> String {
        let re = Regex::new(r"^https://(.+)\.awsapps\.com/start#$").unwrap();
        let caps = re.captures(&self.start_url).unwrap();
        format!(
            "{}-{}",
            self.region,
            caps.get(1).unwrap().as_str().to_string()
        )
    }
    pub fn sso_start_url(&self) -> String {
        let re = Regex::new(r"^https://.+\.awsapps\.com/start#$").unwrap();
        if re.is_match(self.start_url.as_str()) {
            self.start_url.clone()
        } else {
            panic!("Invalid start_url: {}", self.start_url);
        }
    }
}

pub fn get_configuration(file: PathBuf) -> Result<Settings, config::ConfigError> {
    let mut partial = config::Config::builder();
    if file.is_file() {
        partial = partial.add_source(config::File::from(file.clone()));
    }
    let settings = partial
        .set_override("path", file.to_string_lossy().to_string())?
        .add_source(
            // `ARSD_AWSSDK__DEBUG=1` would set `Settings.awssdk.debug`
            config::Environment::with_prefix("ARSD")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
