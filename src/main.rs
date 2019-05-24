
extern crate libfiresale;
use libfiresale::api::DatabaseContext;

// basic 1.0 support
// read document path

const GOOGLE_APPLICATION_CREDENTIALS_KEY: &'static str = "GOOGLE_APPLICATION_CREDENTIALS";
const PROJECT_ID_KEY: &'static str = "PROJECT_ID";

#[derive(Debug, Clone)]
struct Environment {
    pub service_account_path: String,
    pub project_id: String,
}

fn probe_env() -> Result<Environment, String> {
    use std::env;
    let service_account_path = env::var(GOOGLE_APPLICATION_CREDENTIALS_KEY)
        .map_err(|_| format!("Could not find application credentials, is {} set?", GOOGLE_APPLICATION_CREDENTIALS_KEY))?;
    let project_id = env::var(PROJECT_ID_KEY)
        .map_err(|_| format!("Could not find project id, is {} set?", PROJECT_ID_KEY))?;
    Ok(Environment{
        service_account_path, project_id
    })
}

fn main() -> Result<(), String> {
    let environment = probe_env()?;
    let context = DatabaseContext::new(environment.project_id, environment.service_account_path)
        .map_err(|_| "Failed to acquire database context")?;
    dbg!(context.get_document("cars", "bmw"));
//    dbg!(document);
    Ok(())
}
