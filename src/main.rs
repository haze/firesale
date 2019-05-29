extern crate libfiresale;
use clap::ArgMatches;
use libfiresale::api::{DatabaseContext, Document};

mod entrypoint;

// basic 1.0 support
// read document path

const GOOGLE_APPLICATION_CREDENTIALS_KEY: &'static str = "GOOGLE_APPLICATION_CREDENTIALS";
const PROJECT_ID_KEY: &'static str = "PROJECT_ID";

#[derive(Debug, Clone)]
struct Environment {
    pub service_account_path: Option<String>,
    pub project_id: Option<String>,
}

// Gathers environment variables before clap parsing to enforce requirements
fn gather_environment() -> Environment {
    use std::env;
    let service_account_path = env::var(GOOGLE_APPLICATION_CREDENTIALS_KEY).ok();
    let project_id = env::var(PROJECT_ID_KEY).ok();
    return Environment {
        service_account_path,
        project_id,
    };
}

/// Used to represent root level applications options
#[derive(Debug)]
struct Options {
    environment: Environment, // cli-defined environment
    database_name: String,
}

/// This represents a query for a certain document
pub struct DocumentQuery {
    collection_name: String,
    document_name: String,
}

/// This represents a query to view an entire collection
pub struct CollectionQuery {
    collection_name: String,
}

/// This represents a query to export a collection or collections
/// to a specified bucket name
pub struct ExportCollectionQuery {
    collections: Vec<String>,
    bucket_name: String,
}

/// Numerous fronts for the entrypoint of a program after CLI parsing
enum EntryPoint {
    GetDocument(DocumentQuery),
    ViewCollection(CollectionQuery),
    DeleteDocument(DocumentQuery),
    DeleteCollection(CollectionQuery),
    ExportCollection(ExportCollectionQuery),
    Usage(String),
}

// Root meta information
const APP_NAME: &'static str = "firesale";
const APP_VERSION: &'static str = "0.1";
const APP_AUTHOR: &'static str = "Haze Booth <isnt@haze.cool>";
const ABOUT_APP: &'static str = "CLI Firestore Interface";

// Application config
const CREDENTIALS_LOCATION_ARG: &'static str = "credentials";
const PROJECT_ID_ARG: &'static str = "project_id";

// Subcommands
const GET_SUB_COMMAND: &'static str = "get";
const DELETE_SUB_COMMAND: &'static str = "delete";
const EXPORT_SUB_COMMAND: &'static str = "export";

const DATABASE_NAME: &'static str = "database";
const DEFAULT_DATABASE_NAME: &'static str = "(default)";

const COLLECTIONS: &'static str = "collections";
const BUCKET_NAME: &'static str = "bucket";

const COLLECTION_NAME: &'static str = "collection";
const COLLECTION_NAME_SHORT: &'static str = "c";

const DOCUMENT_NAME: &'static str = "document";
const DOCUMENT_NAME_SHORT: &'static str = "d";

fn setup_arguments(environ: &Environment) -> (Options, EntryPoint) {
    use clap::{App, Arg, SubCommand};
    let matches = App::new(APP_NAME)
        .version(APP_VERSION)
        .author(APP_AUTHOR)
        .about(ABOUT_APP)
        .arg(Arg::with_name(PROJECT_ID_ARG).required(environ.project_id.is_none()))
        .arg(
            Arg::with_name(CREDENTIALS_LOCATION_ARG)
                .required(environ.service_account_path.is_none()),
        )
        .subcommand(
            SubCommand::with_name(GET_SUB_COMMAND)
                .arg(Arg::with_name(COLLECTION_NAME).required(true))
                .arg(Arg::with_name(DOCUMENT_NAME)),
        )
        .subcommand(
            SubCommand::with_name(DELETE_SUB_COMMAND)
                .arg(Arg::with_name(COLLECTION_NAME).required(true))
                .arg(Arg::with_name(DOCUMENT_NAME)),
        )
        .subcommand(
            SubCommand::with_name(EXPORT_SUB_COMMAND)
                .arg(Arg::with_name(BUCKET_NAME).required(true))
                .arg(Arg::with_name(COLLECTIONS).multiple(true)),
        )
        .arg(
            Arg::with_name(DATABASE_NAME)
                .required(true)
                .default_value(DEFAULT_DATABASE_NAME),
        )
        .get_matches();
    let environment = {
        // TODO(hazebooth): investigate
        let service_account_path = matches.value_of(CREDENTIALS_LOCATION_ARG).map(String::from);
        let project_id = matches.value_of(PROJECT_ID_ARG).map(String::from);
        Environment {
            service_account_path,
            project_id,
        }
    };
    let database_name = matches.value_of(DATABASE_NAME).unwrap();
    let options = Options {
        environment,
        database_name,
    };
    if let Some(get_command) = &matches.subcommand_matches(GET_SUB_COMMAND) {
        if get_command.is_present(DOCUMENT_NAME) {
            let query = DocumentQuery::from_sub_matches(get_command);
            return (options, EntryPoint::GetDocument(query));
        } else {
            let query = CollectionQuery::from_sub_matches(get_command);
            return (options, EntryPoint::ViewCollection(query));
        }
    } else if let Some(delete_command) = &matches.subcommand_matches(DELETE_SUB_COMMAND) {
        if delete_command.is_present(DOCUMENT_NAME) {
            let query = DocumentQuery::from_sub_matches(delete_command);
            return (options, EntryPoint::DeleteDocument(query));
        } else {
            let query = CollectionQuery::from_sub_matches(delete_command);
            return (options, EntryPoint::DeleteCollection(query));
        }
        let query = DocumentQuery::from_sub_matches(delete_command);
        return (options, EntryPoint::DeleteDocument(query));
    } else if let Some(export_command) = &matches.subcommand_matches(EXPORT_SUB_COMMAND) {
        let query = ExportCollectionQuery::from_sub_matches(export_command);
        return (options, EntryPoint::ExportCollection(query));
    }
    return (options, EntryPoint::Usage(matches.usage().to_string()));
}

impl ExportCollectionQuery {
    fn from_sub_matches(matches: &&ArgMatches) -> ExportCollectionQuery {
        ExportCollectionQuery {
            collections: matches
                .values_of_lossy(COLLECTIONS)
                .unwrap_or_else(|| Vec::new()),
            bucket_name: matches.value_of(BUCKET_NAME).unwrap().to_string(),
        }
    }
}

impl DocumentQuery {
    fn from_sub_matches(matches: &&ArgMatches) -> DocumentQuery {
        DocumentQuery {
            collection_name: matches.value_of(COLLECTION_NAME).unwrap().to_string(),
            document_name: matches.value_of(DOCUMENT_NAME).unwrap().to_string(),
        }
    }
}

impl CollectionQuery {
    fn from_sub_matches(matches: &&ArgMatches) -> CollectionQuery {
        CollectionQuery {
            collection_name: matches.value_of(COLLECTION_NAME).unwrap().to_string(),
        }
    }
}

fn main() -> Result<(), String> {
    let environment = gather_environment();
    let (options, entrypoint) = setup_arguments(&environment);
    // if the entrypoint is set, use that
    // if the entrypoint is not set, default to env
    let context = {
        if let (Some(service_account_path), Some(project_id)) = (
            options.environment.service_account_path,
            options.environment.project_id,
        ) {
            DatabaseContext::new(project_id, service_account_path)
        } else if let (Some(service_account_path), Some(project_id)) =
            (environment.service_account_path, environment.project_id)
        {
            DatabaseContext::new(project_id, service_account_path)
        } else {
            Err(String::from("Failed to create database context, not provided in environment variables or cli args"))
        }
    }?;
    match entrypoint {
        EntryPoint::GetDocument(query) => entrypoint::handle_document_get(query, context),
        EntryPoint::ViewCollection(query) => entrypoint::handle_document_view(query, context),
        EntryPoint::DeleteDocument(query) => entrypoint::handle_document_delete(query, context),
        EntryPoint::DeleteCollection(query) => entrypoint::handle_collection_delete(query, context),
        EntryPoint::ExportCollection(query) => entrypoint::handle_database_export(query, context),
        EntryPoint::Usage(usage_str) => Ok(println!("{}", usage_str)),
        _ => {
            println!("entrypoint not implemented");
            Ok(())
        }
    }
}
