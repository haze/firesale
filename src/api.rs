
use chrono::DateTime;
use chrono::Utc;
use std::path::{Path, PathBuf};
use smpl_jwt::Jwt;
use goauth::auth::JwtClaims;
use goauth::scopes::Scope;
use anymap::AnyMap;
use std::collections::HashMap;
use json::JsonValue;

const FIRESTORE_BASE_URL: &'static str = "https://firestore.googleapis.com/v1";
const TOKEN_URL: &'static str = "";

// A Firestore document
#[derive(Debug)]
pub struct Document {
    pub name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub data: AnyMap,
}

// A value used in Firestore responses
// i.e
// fields: {
//   speed: {
//     "integerValue": "100"
//   }
// }
struct Value {

}

#[derive(Deserialize, Debug)]
struct PartialDocument {
    pub name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

// Represents a contextualized instance for database handling
#[derive(Debug)]
pub struct DatabaseContext {
    pub project_id: String,
    auth_token: goauth::auth::Token
}

fn map_firestore_fields_to_any_map(fields: &json::JsonValue)
-> AnyMap {
    let any_map = AnyMap::new();
    if fields.is_object() {
        for (name, field) in fields.entries() {
            // is integer?
            println!("{} = {:#}", name, field);
        }
    }
    any_map
}

impl DatabaseContext {

    // Create a new instance that uses project_id as anchoring context
    pub fn new<S>(project_id: S, service_account_path: S)
        -> Result<DatabaseContext, String> where S: Into<String> {
        // ensure String types
        let project_id = project_id.into();
        let service_account_path = service_account_path.into();

        // get jwt & credentials from file
        let credentials = goauth::credentials::Credentials::from_file(&*service_account_path)
            .map_err(|_| "Failed to load credentials from file")?;
        let claims = JwtClaims::new(credentials.iss(),
        &Scope::DataStore, credentials.token_uri(), None, None);
        let jwt = Jwt::new(claims, credentials.rsa_key()
            .map_err(|_| "Failed to get RSA private key from credentials")?, None);
        // cool, we have a token
        let auth_token = goauth::get_token_with_creds(
            &jwt, &credentials
        ).map_err(|_| "Failed to authenticate")?;
        // return success
        Ok(DatabaseContext{
            project_id, auth_token
        })
    }

    // Creates a proper URL for the Firestore REST api
    fn make_document_url(&self,
                         collection_name: String, document_id: String) -> String {
        format!("{}/projects/{}/databases/(default)/documents/{}/{}", FIRESTORE_BASE_URL,
                self.project_id, collection_name, document_id)
        // TODO(hazebooth): Optimize out of format!
    }

    // TODO(hazebooth): support document masks
    pub fn get_document<S: Into<String>>(&self, collection_name: S, document_id: S)
                           -> Result<(), String> where S: Into<String> {
        // ensure String types
        let collection_name = collection_name.into();
        let document_id = document_id.into();

        let document_ref_url = self.make_document_url(collection_name, document_id);
        dbg!(&document_ref_url);
        let client = reqwest::Client::new();
        let mut response =  client.get(&*document_ref_url)
            .header(reqwest::header::AUTHORIZATION, &*self.get_authorization_key())
            .send().map_err(|_| "Failed to get response from Google")?;
        let response_text = response.text().map_err(|_| "Failed to get response text")?;
//        let partial_doc: PartialDocument = response.json().map_err(|e| format!("Failed to convert response to json: {}", e))?;

        let parsed_json = json::parse(&*response_text)
            .map_err(|_| "Failed to parse request json (text)")?;
        dbg!(&parsed_json);
        let fields = &parsed_json["fields"];
        dbg!(&fields);
        map_firestore_fields_to_any_map(fields);
        Ok(())

//        let assembled_doc = partial_doc.assemble(fields);
    }

    // Used to give us the key for our Authorization Header
    // Authorization: Bearer <token>
    // ------------------^
    fn get_authorization_key(&self) -> String {
        format!("Bearer {}", self.auth_token.access_token())
    }
}
