use chrono::Utc;
use chrono::{Date, DateTime};
use goauth::auth::JwtClaims;
use goauth::scopes::Scope;
use goauth::scopes::Scope::Firebase;
use serde::Deserializer;
use smpl_jwt::Jwt;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

mod errors {
    // To be used when a request fails, formatted with the request error
    pub fn http_error(err: reqwest::Error) -> String {
        format!("Failed to get response from firestore: {}", err.to_string())
    }

    // To be used when the json decoding of a request fails, formatted with the decoding error
    pub fn json_decode_error(err: reqwest::Error) -> String {
        format!("Failed to decode JSON: {}", err.to_string())
    }

    // To be used when the json encoding of a request fails, formatted with the decoding error
    pub fn json_encode_error(err: serde_json::Error) -> String {
        format!("Failed to encode JSON: {}", err.to_string())
    }
}

const FIRESTORE_BASE_URL: &'static str = "https://firestore.googleapis.com/v1";
const FIRESTORE_BETA_BASE_URL: &'static str = " https://firestore.googleapis.com/v1beta1";

//// the `fields` attribute for Firestore Documents
#[derive(Debug, Deserialize)]
pub struct FirestoreFields(HashMap<String, FirestoreType>);

#[derive(Debug, Deserialize)]
struct Map {
    fields: FirestoreFields,
}

#[derive(Debug, Deserialize)]
struct Array {
    values: Vec<FirestoreType>,
}

#[derive(Debug)]
pub struct DatabaseContext {
    pub project_id: String,
    auth_token: goauth::auth::Token,
    client: reqwest::Client,
}

// Firestore GeoPoint type
#[derive(Debug, Deserialize, Clone, Copy)]
struct GeoPoint {
    latitude: i32,
    longitude: i32,
}

use serde_aux::field_attributes::deserialize_number_from_string;

// Represents a mapping between Firestore data types and Rust types
#[derive(Debug, Deserialize)]
enum FirestoreType {
    #[serde(rename = "integerValue")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    Integer(i32),
    #[serde(rename = "booleanValue")]
    Boolean(bool),
    #[serde(rename = "stringValue")]
    String(String),
    #[serde(rename = "geoPointValue")]
    GeoLocation(GeoPoint),
    #[serde(rename = "arrayValue")]
    Array(Array),
    #[serde(rename = "mapValue")]
    Map(Map),
    #[serde(rename = "timestampValue")]
    Timestamp(DateTime<Utc>),
    #[serde(rename = "nullValue")]
    Null,
}

#[derive(Debug, Deserialize)]
pub struct Document {
    name: String,
    fields: FirestoreFields,
    #[serde(rename = "createTime")]
    create_time: DateTime<Utc>,
    #[serde(rename = "updateTime")]
    update_time: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct DocumentMask {
    #[serde(rename = "fieldPaths")]
    field_paths: Vec<String>,
}

#[derive(Serialize)]
pub enum ConsistencySelector {
    Transaction(String),
    #[serde(rename = "readTime")]
    ReadTime(DateTime<Utc>),
}

pub mod list_documents {
    #[derive(Serialize)]
    pub struct Request {
        #[serde(rename = "pageSize")]
        pub page_size: i32,
        #[serde(rename = "orderBy")]
        pub order_by: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mask: Option<super::DocumentMask>,
        #[serde(rename = "showMissing")]
        pub show_missing: bool,
        pub consistency_selector: super::ConsistencySelector,
    }

    #[derive(Debug, Deserialize)]
    pub struct Response {
        documents: Vec<super::Document>,
        #[serde(rename = "nextPageToken")]
        next_page_token: String,
    }
}

pub mod batch_get {
    #[derive(Serialize)]
    pub struct Request {
        documents: Vec<String>,
    }

    #[derive(Deserialize)]
    pub struct Response {
        transaction: String,
        read_time: String,
        found: Option<super::Document>,
        missing: Option<String>,
    }
}

impl DatabaseContext {
    fn auth_header_map(&self) -> Result<reqwest::header::HeaderMap, String> {
        let mut map = reqwest::header::HeaderMap::new();
        let str = &*self.auth_token.access_token();
        map.insert(
            reqwest::header::AUTHORIZATION,
            str.parse().map_err(|_| "Invalid Header Value")?,
        );
        Ok(map)
    }

    // Create a new instance that uses project_id as anchoring context
    pub fn new<S>(project_id: S, service_account_path: S) -> Result<DatabaseContext, String>
    where
        S: Into<String>,
    {
        // ensure String types
        let project_id = project_id.into();
        let service_account_path = service_account_path.into();

        // get jwt & credentials from file
        let credentials = goauth::credentials::Credentials::from_file(&*service_account_path)
            .map_err(|_| "Failed to load credentials from file")?;
        let claims = JwtClaims::new(
            credentials.iss(),
            &Scope::DataStore,
            credentials.token_uri(),
            None,
            None,
        );
        let jwt = Jwt::new(
            claims,
            credentials
                .rsa_key()
                .map_err(|_| "Failed to get RSA private key from credentials")?,
            None,
        );
        // cool, we have a token
        let auth_token = goauth::get_token_with_creds(&jwt, &credentials)
            .map_err(|_| "Failed to authenticate")?;
        let client = reqwest::Client::new();
        // return success
        Ok(DatabaseContext {
            client,
            project_id,
            auth_token,
        })
    }

    fn make_api_base(&self) -> String {
        format!("{}/projects/{}", FIRESTORE_BASE_URL, self.project_id)
    }

    // Creates a proper URL for the Firestore REST api
    fn make_document_url(&self, collection_name: String, document_id: String) -> String {
        format!(
            "{}/databases/(default)/documents/{}{}",
            self.make_api_base(),
            collection_name,
            document_id
        )
    }

    fn make_batch_get_url(&self, database_name: String) -> String {
        format!(
            "{}/{{database={}}}/documents:batchGet",
            FIRESTORE_BETA_BASE_URL,
            format!("projects/{}/databases/{}", self.project_id, database_name)
        )
    }

    // Deletes a document from said collection

    pub fn delete_document<S>(&self, collection_name: S, document_id: S) -> Result<Document, String>
    where
        S: Into<String>,
    {
        // ensure String types
        let collection_name = collection_name.into();
        let document_id = document_id.into();

        let document_ref_url = self.make_document_url(collection_name, document_id);
        self.delete_document_at_path(&*document_ref_url)
    }

    // TODO(hazebooth): support document masks
    // GETs a document from said collection
    // https://firebase.google.com/docs/firestore/reference/rest/v1beta1/projects.databases.documents/get
    pub fn get_document<S>(&self, collection_name: S, document_id: S) -> Result<Document, String>
    where
        S: Into<String>,
    {
        // ensure String types
        let collection_name = collection_name.into();
        let document_id = document_id.into();

        let document_ref_url = self.make_document_url(collection_name, document_id);
        self.retrieve_document(&*document_ref_url)
    }

    // Inner implementation of `delete_document`
    fn delete_document_at_path(&self, path: &str) -> Result<Document, String> {
        let mut response = self
            .client
            .delete(path)
            .headers(self.auth_header_map()?)
            .send()
            .map_err(errors::http_error)?;
        let document = response
            .json::<Document>()
            .map_err(errors::json_decode_error)?;
        Ok(document)
    }

    // Inner implementation of `get_document`.
    fn retrieve_document(&self, path: &str) -> Result<Document, String> {
        let mut response = self
            .client
            .get(path)
            .headers(self.auth_header_map()?)
            .send()
            .map_err(errors::http_error)?;
        let document = response
            .json::<Document>()
            .map_err(errors::json_decode_error)?;
        Ok(document)
    }

    // Internal for batch_get_documents
    // https://firebase.google.com/docs/firestore/reference/rest/v1beta1/projects.databases.documents/batchGet#google.firestore.v1beta1.Firestore.BatchGetDocuments
    fn batch_get(&self, documents: Vec<String>, path: &str) -> Result<batch_get::Response, String> {
        let mut response = self
            .client
            .post(path)
            .headers(self.auth_header_map()?)
            .send()
            .map_err(errors::http_error)?;
        response
            .json::<batch_get::Response>()
            .map_err(errors::json_decode_error)
    }

    pub fn batch_get_documents<S>(
        &self,
        documents: Vec<S>,
        database_name: S,
    ) -> Result<batch_get::Response, String>
    where
        S: Into<String>,
    {
        let documents = documents
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<String>>();
        let database_name: String = database_name.into();
        self.batch_get(documents, &*self.make_batch_get_url(database_name))
    }

    // want https://firestore.googleapis.com/v1beta1/{parent=projects/*/databases/*/documents/*/**}/{collectionId}
    // ours https://firestore.googleapis.com/v1beta1/{parent=projects/hazes-test-project/databases/default/documents/*/**}/cars
    fn make_list_documents_url(&self, database_name: &str, collection_name: &str) -> String {
        let parent = format!("projects/{}/databases/{}", self.project_id, database_name);
        format!(
            "{}/{{parent={}}}/{}",
            FIRESTORE_BETA_BASE_URL, parent, collection_name
        )
    }

    pub fn list_documents(
        &self,
        page_size: i32,
        order_by: String,
        mask: Option<DocumentMask>,
        show_missing: bool,
        consistency_selector: ConsistencySelector,
        database_name: &str,
        collection_name: &str,
    ) -> Result<list_documents::Response, String> {
        let request = list_documents::Request {
            page_size,
            order_by,
            mask,
            show_missing,
            consistency_selector,
        };
        let request_json = serde_json::to_string(&request).map_err(errors::json_encode_error)?;
        println!("{}", &*self.make_list_documents_url(database_name, collection_name));
        let mut response = self
            .client
            .get(&*self.make_list_documents_url(database_name, collection_name))
            .headers(self.auth_header_map()?)
            .body(request_json)
            .send()
            .map_err(errors::http_error)?;
        response
            .json::<list_documents::Response>()
            .map_err(errors::json_decode_error)
    }

    // Used to give us the key for our Authorization Header
    // Authorization: Bearer <token>
    // ------------------^
    fn get_authorization_key(&self) -> String {
        format!("Bearer {}", self.auth_token.access_token())
    }
}
