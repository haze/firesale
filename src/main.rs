
extern crate libfiresale;
use libfiresale::api::DatabaseContext;

// basic 1.0 support
// read document path

fn main() -> Result<(), String> {
    let service_account = "./service_account.json";
    let project_id = "hazes-test-project";
    let context = DatabaseContext::new(project_id, service_account)
        .map_err(|_| "Failed to acquire database context")?;
    dbg!(context.get_document("cars", "bmw"));
//    dbg!(document);
    Ok(())
}
