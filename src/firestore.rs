// This file contains 1:1 representations of the REST APIs firestore provides

use super::errors::{Error, Result};
use reqwest::header::HeaderMap;

const FIRESTORE_BASE_1BETA2: &'static str = "https://firestore.googleapis.com/v1beta2";

/// Contains 1:1 representations of gRPC firestore types
mod types {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize)]
    pub struct Metadata {
        #[serde(flatten)]
        data: HashMap<String, String>,
    }

    #[derive(Deserialize)]
    pub struct Detail {
        #[serde(rename = "@type")]
        kind: String,
        #[serde(flatten)]
        data: HashMap<String, String>, // TODO(hazebooth): transform
    }

    /// https://firebase.google.com/docs/firestore/reference/rest/Shared.Types/Operation#Status
    #[derive(Deserialize)]
    pub struct Status {
        code: i32,
        message: String,
        details: Vec<Detail>,
    }

    /// https://firebase.google.com/docs/firestore/reference/rest/Shared.Types/Operation
    /// N.B. T is the response type, see `response` field for error
    #[derive(Deserialize)]
    pub struct Operation<T> {
        name: String,
        metadata: Metadata,
        done: bool,
        error: Option<Status>,
        response: Option<T>,
    }

    /// Represents `google.protobuf.Empty`
    #[derive(Deserialize)]
    pub struct EmptyResponse;
}

pub mod databases {
    use super::types::{EmptyResponse, Operation};
    use super::{Error, HeaderMap, Result};
    use reqwest::Client;
    use snafu::ResultExt;

    /// Represents the input parameters for `export_documents`
    pub struct ExportDocumentQuery {
        /// Database to export. Should be of the form:
        /// projects/{project_id}/databases/{database_id}.
        database_name: String,
        collection_ids: Option<Vec<String>>,
        output_uri_prefix: String,
    }

    #[derive(Serialize)]
    struct ExportDocumentBody {
        #[serde(rename = "collectionIds")]
        collection_ids: Option<Vec<String>>,
        #[serde(rename = "outputUriPrefix")]
        output_uri_prefix: String,
    }

    impl ExportDocumentQuery {
        fn into_body(self) -> ExportDocumentBody {
            let collection_ids = self.collection_ids;
            let output_uri_prefix = self.output_uri_prefix;
            ExportDocumentBody {
                collection_ids,
                output_uri_prefix,
            }
        }
    }

    /// https://firebase.google.com/docs/firestore/reference/rest/v1beta2/projects.databases/exportDocuments
    pub fn export_documents(
        client: Client,
        headers: HeaderMap,
        params: ExportDocumentQuery,
    ) -> Result<Operation<EmptyResponse>> {
        fn make_url(name: &str) -> String {
            format!(
                "{}/{{name={}}}:exportDocuments",
                super::FIRESTORE_BASE_1BETA2,
                name
            )
        }
        // setup parameters
        let database_name = &*params.database_name;
        let url = &*make_url(database_name);
        let request_body = params.into_body();
        // send request
        let mut response = client.post(url).headers(headers).send()?;
        response
            .json::<Operation<EmptyResponse>>()
            .map_err(Error::from)
    }

    pub struct ImportDocumentQuery {
        database_name: String,
        collection_ids: Vec<String>,
        input_uri_prefix: String,
    }

    impl ImportDocumentQuery {
        fn into_body(self) -> ImportDocumentBody {
            let collection_ids = self.collection_ids;
            let input_uri_prefix = self.input_uri_prefix;
            ImportDocumentBody {
                collection_ids,
                input_uri_prefix,
            }
        }
    }

    /// Input body for `import_documents`
    #[derive(Serialize)]
    struct ImportDocumentBody {
        /// Which collection ids to import. Unspecified means all collections included in the import.
        #[serde(rename = "collectionIds")]
        collection_ids: Vec<String>,

        /// Location of the exported files. This must match the
        /// `output_uri_prefix` of an ExportDocumentsResponse from an export that has completed successfully
        #[serde(rename = "inputUriPrefix")]
        input_uri_prefix: String,
    }

    /// https://firebase.google.com/docs/firestore/reference/rest/v1beta2/projects.databases/importDocuments
    pub fn import_documents(
        client: Client,
        headers: HeaderMap,
        params: ImportDocumentQuery,
    ) -> Result<Operation<EmptyResponse>> {
        fn make_url(name: &str) -> String {
            format!(
                "{}/{{name={}}}:importDocuments",
                super::FIRESTORE_BASE_1BETA2,
                name
            )
        }
        // setup parameters
        let database_name = &*params.database_name;
        let url = &*make_url(database_name);
        let request_body = params.into_body();
        // send request
        let mut response = client.post(url).headers(headers).send()?;
        response
            .json::<Operation<EmptyResponse>>()
            .map_err(Error::from)
    }

}
