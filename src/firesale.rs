// This file aims to be a 1:1 representation of the REST Firestore API
use super::error::{Error, Result};

/// Used for `export_documents` and `import_documents`.
const FIRESTORE_BETA_API_URL: &'static str =
    "https://firestore.googleapis.com/v1beta2";

/// Transforms a vector of Ss that can be Ts into Ts
fn all_into<T, S: Into<T>>(source: Vec<S>) -> Vec<T> {
    source.into_iter().map(|item| item.into()).collect::<Vec<T>>()
}

/// Commonly shared structs used in the Firestore API
mod common {
    use serde::de::DeserializeOwned;
    use std::collections::HashMap;

    /// Used in `Operation`
    #[derive(Deserialize)]
    pub struct Metadata {
        #[serde(rename = "@type")]
        kind: String,
        data: HashMap<String, String>,
    }

    pub type Detail = Metadata;

    /// https://firebase.google.com/docs/firestore/reference/rest/Shared.Types/Operation#Status
    #[derive(Deserialize)]
    pub struct Status {
        code: i32,
        message: String,
        details: Vec<Detail>,
    }

    /// https://firebase.google.com/docs/firestore/reference/rest/Shared.Types/Operation
    #[derive(Deserialize)]
    pub struct Operation<T> {
        name: String,
        metadata: Metadata,
        done: bool,
        error: Option<Status>,
        response: Option<T>,
    }

    /// Used for `Operation`'s without a response
    #[derive(Deserialize)]
    pub struct NullResponse;
}

/// https://firebase.google.com/docs/firestore/reference/rest/v1beta2/projects.databases
/// This module contains two functions `export_documents` and `import_documents`
mod databases {
    use super::{
        all_into,
        common::{NullResponse, Operation},
        Error, Result,
    };
    use reqwest::Client;
    use serde::export::fmt::Display;

    /// https://firebase.google.com/docs/firestore/reference/rest/v1beta2/projects.databases/exportDocuments
    /// Exports a copy of all or a subset of documents from
    /// Google Cloud Firestore to another storage system,
    /// such as Google Cloud Storage
    fn export_documents<S: Into<String>>(
        client: Client,
        database_to_export: S,
        collection_ids: Vec<S>,
        output_uri_prefix: S,
    ) -> Result<Operation<NullResponse>> {
        #[derive(Serialize)]
        struct Body {
            #[serde(rename = "collectionIds")]
            collection_ids: Vec<String>,
            #[serde(rename = "outputUriPrefix")]
            output_uri_prefix: String,
        }

        // Stringify input
        let collection_ids = all_into(collection_ids);
        let output_uri_prefix = output_uri_prefix.into();
        let database_to_export = database_to_export.into();

        // Make URL
        let url =
            &*make_url(EXPORT_DOCUMENTS_FUNCTION_NAME, &database_to_export);
        let url = &*format!(
            "{}/{{name={}}}:exportDocuments",
            super::FIRESTORE_BETA_API_URL,
            database_to_export
        );
        // Send request
        client
            .post(url)
            .json(&Body { collection_ids, output_uri_prefix })
            .send()?
            .json::<Operation<NullResponse>>()
            .map_err(Error::from)
    }

    /// https://firebase.google.com/docs/firestore/reference/rest/v1beta2/projects.databases/importDocuments
    /// Imports documents into Google Cloud Firestore.
    /// Existing documents with the same name are overwritten.
    fn import_documents<S: Into<String>>(
        client: Client,
        database_to_import_into: S,
        collection_ids: Vec<S>,
        input_uri_prefix: S,
    ) -> Result<Operation<NullResponse>> {
        #[derive(Serialize)]
        struct Body {
            #[serde(rename = "collectionIds")]
            collection_ids: Vec<String>,
            #[serde(rename = "inputUriPrefix")]
            input_uri_prefix: String,
        }

        // Stringify input
        let collection_ids = all_into(collection_ids);
        let input_uri_prefix = input_uri_prefix.into();
        let database_to_import_into = database_to_import_into.into();

        // Make URL
        let url = &*make_url(
            IMPORT_DOCUMENTS_FUNCTION_NAME,
            &database_to_import_into,
        );
        // Send request
        client
            .post(url)
            .json(&Body { collection_ids, input_uri_prefix })
            .send()?
            .json::<Operation<NullResponse>>()
            .map_err(Error::from)
    }

    const IMPORT_DOCUMENTS_FUNCTION_NAME: &'static str = "importDocuments";
    const EXPORT_DOCUMENTS_FUNCTION_NAME: &'static str = "exportDocuments";

    /// Creates a BETA firestore url with specified function
    /// and database_name
    fn make_url<S: Into<String>>(function: S, name: S) -> String {
        format!(
            "{}/{{name={}}}:{}",
            super::FIRESTORE_BETA_API_URL,
            name.into(),
            function.into()
        )
    }

    mod test {
        use super::{
            make_url, EXPORT_DOCUMENTS_FUNCTION_NAME,
            IMPORT_DOCUMENTS_FUNCTION_NAME,
        };

        #[test]
        fn import_documents_url_works() {
            let url = make_url(
                IMPORT_DOCUMENTS_FUNCTION_NAME,
                "projects/{project_id}/databases/{database_id}",
            );
            assert_eq!(url,
                       "https://firestore.googleapis.com/v1beta2/{name=projects/{project_id}/databases/{database_id}}:importDocuments");
        }

        #[test]
        fn export_documents_url_works() {
            let url = make_url(
                EXPORT_DOCUMENTS_FUNCTION_NAME,
                "projects/{project_id}/databases/{database_id}",
            );
            assert_eq!(url,
                       "https://firestore.googleapis.com/v1beta2/{name=projects/{project_id}/databases/{database_id}}:exportDocuments");
        }
    }
}

mod test {
    use super::all_into;

    #[test]
    fn test_all_into() {
        let alpha = vec!["hello", "world"];
        let alpha_result: Vec<String> = all_into(alpha);
        assert_eq!(
            alpha_result,
            vec![String::from("hello"), String::from("world")]
        )
    }
}
