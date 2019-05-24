
use chrono::DateTime;
use chrono::Utc;
use std::path::{Path, PathBuf};
use smpl_jwt::Jwt;
use goauth::auth::JwtClaims;
use goauth::scopes::Scope;
use anymap::AnyMap;
use std::collections::HashMap;
use json::{JsonValue, JsonError};
use goauth::scopes::Scope::Firebase;
use std::convert::TryFrom;

const FIRESTORE_BASE_URL: &'static str = "https://firestore.googleapis.com/v1";
const TOKEN_URL: &'static str = "";

// the `fields` attribute for Firestore Documents
pub type FirestoreFields = HashMap<String, FirestoreType>;

// A Firestore document
#[derive(Debug)]
pub struct Document {
    pub name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub fields: FirestoreFields,
}

#[derive(Deserialize, Debug)]
struct PartialDocument {
    pub name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl PartialDocument {
    fn assemble(&self, fields: FirestoreFields) -> Document {
        let name = self.name.clone(); // FIXME(hazebooth): clones are bad :(
        let create_time = self.create_time.clone();
        let update_time = self.update_time.clone();
        Document {
            name, create_time, update_time, fields,
        }
    }
}

// Represents a contextualized instance for database handling
#[derive(Debug)]
pub struct DatabaseContext {
    pub project_id: String,
    auth_token: goauth::auth::Token
}

// Firestore GeoPoint type
#[derive(Debug, Clone, Copy)]
struct GeoPoint {
    latitude: i32,
    longitude: i32,
}

// Represents a mapping between Firestore data types and Rust types
#[derive(Debug)]
enum FirestoreType {
    Integer(i32),
    Boolean(bool),
    String(String),
    GeoLocation(GeoPoint),
    Array(Vec<FirestoreType>),
    Map(FirestoreFields),
    Timestamp(DateTime<Utc>),
    Null
}

impl FirestoreType {
    fn extract_integer(value: &JsonValue) -> Result<i32, String> {
        let value_str = value["integerValue"].as_str()
            .ok_or_else(|| String::from("No string value found"))?.to_string();
        value_str.parse::<i32>()
            .map_err(|_| String::from("Failed to parse string value to integer"))
    }

    fn extract_boolean(value: &JsonValue) -> Result<bool, String> {
        value["booleanValue"].as_bool()
            .ok_or_else(|| String::from("Found no boolean value"))
    }

    fn extract_geopoint(value: &JsonValue) -> Result<GeoPoint, String> {
        let latitude = value["geoPointValue"]["latitude"].as_i32()
            .ok_or_else(|| String::from("Failed to parse latitude"))?;
        let longitude = value["geoPointValue"]["longitude"].as_i32()
            .ok_or_else(|| String::from("Failed to parse longitude"))?;
        Ok(GeoPoint{
            latitude, longitude
        })
    }

    fn extract_string(value: &JsonValue) -> Result<String, String> {
        value["stringValue"].as_str()
            .ok_or_else(|| String::from("Failed to get stringValue string"))
            .map(String::from)
    }

    fn extract_timestamp(value: &JsonValue) -> Result<DateTime<Utc>, String> {
        let timestamp_str = value["timestampValue"].as_str()
            .ok_or_else(|| String::from("Failed to get timestamp string"))?;
        timestamp_str.parse::<DateTime<Utc>>()
            .map_err(|e| format!("Failed to parse timestamp: {}", e))
    }

    fn extract_array(value: &JsonValue) -> Result<Vec<FirestoreType>, String> {
        let values = &value["arrayValue"]["values"];
        let array_len = values.len();
        let mut firestore_values = Vec::with_capacity(array_len);
        for item in values.members() {
            firestore_values.push(FirestoreType::try_from(item)?);
        }
        Ok(firestore_values)
    }

    fn extract_map(value: &JsonValue) -> Result<FirestoreFields, String> {
        let mut fields = FirestoreFields::new();
        let map_value = &value["mapValue"]["fields"];
        for (key, entry ) in map_value.entries() {
            let parsed_value = FirestoreType::try_from(entry)?;
            fields.insert(key.to_string(), parsed_value);
        }
        Ok(fields)
    }

}

impl TryFrom<&JsonValue> for FirestoreType {
    type Error = String;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        if is_integer(value) {
            let value = FirestoreType::extract_integer(value)?;
            return Ok(FirestoreType::Integer(value))
        } else if is_string(value) {
            let value = FirestoreType::extract_string(value)?;
            return Ok(FirestoreType::String(value))
        } else if is_boolean(value) {
            let value = FirestoreType::extract_boolean(value)?;
            return Ok(FirestoreType::Boolean(value))
        } else if is_null(value) {
            return Ok(FirestoreType::Null)
        } else if is_geopoint(value) {
            let value = FirestoreType::extract_geopoint(value)?;
            return Ok(FirestoreType::GeoLocation(value))
        } else if is_timestamp(value) {
            let value = FirestoreType::extract_timestamp(value)?;
            return Ok(FirestoreType::Timestamp(value))
        } else if is_array(value) {
            let values = FirestoreType::extract_array(value)?;
            return Ok(FirestoreType::Array(values))
        } else if is_map(value) {
            let values = FirestoreType::extract_map(value)?;
            return Ok(FirestoreType::Map(values))
        }
        Err("No value found".to_string())
    }
}

fn is_integer(value: &JsonValue) -> bool {
    value.has_key("integerValue")
}

fn is_boolean(value: &JsonValue) -> bool {
    value.has_key("booleanValue")
}

fn is_null(value: &JsonValue) -> bool {
    value.has_key("nullValue")
}

fn is_geopoint(value: &JsonValue) -> bool {
    value.has_key("geoPointValue")
}

fn is_timestamp(value: &JsonValue) -> bool {
    value.has_key("timestampValue")
}

fn is_string(value: &JsonValue) -> bool {
    value.has_key("stringValue")
}

fn is_map(value: &JsonValue) -> bool {
    value.has_key("mapValue")
}

fn is_array(value: &JsonValue) -> bool {
    value.has_key("arrayValue")
}


fn get_firestore_fields(fields: &json::JsonValue)
-> Result<FirestoreFields, String> {
    let mut field_map = FirestoreFields::new();
    if fields.is_object() {
        for (name, field) in fields.entries() {
            let fsv = FirestoreType::try_from(field)?;
            field_map.insert(name.to_string(), fsv);
        }
    }
    Ok(field_map)
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
        println!("{}", response_text);
        let parsed_json = json::parse(&*response_text)
            .map_err(|_| "Failed to parse request json (text)")?;
        let fields = &parsed_json["fields"];
        let fields = get_firestore_fields(fields)?;
//        let assembled_doc = partial_doc.assemble(fields);
//        Ok(assembled_doc)
        Ok(())
    }

    // Used to give us the key for our Authorization Header
    // Authorization: Bearer <token>
    // ------------------^
    fn get_authorization_key(&self) -> String {
        format!("Bearer {}", self.auth_token.access_token())
    }
}
