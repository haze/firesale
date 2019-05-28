// This file contains 1:1 representations of the REST APIs firestore provides

use super::errors;

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

mod databases {
    use super::errors;
    use super::types::{EmptyResponse, Operation};
    use failure::Error;
    use reqwest::Client;

    type Result<T> = std::result::Result<T, Error>;

    /// Represents the input parameters for `export_documents`
    struct ExportDocumentQuery {
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
    fn export_documents(
        client: Client,
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
        let request_body = params.into_body();
        // send request
        let url = &*make_url(database_name);
        let mut response = client.post(url).send().map_err(errors::api::Error::from)?;
        response
            .json::<Operation<EmptyResponse>>()
            .map_err(errors::parsing::Error::from)
    }
}
