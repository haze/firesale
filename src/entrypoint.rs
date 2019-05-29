use libfiresale::errors::Result;
use libfiresale::firestore;

pub fn handle_database_export(
    query: crate::ExportCollectionQuery,
    ctx: crate::DatabaseContext,
) -> Result<()> {
    ctx.export_database(firestore::databases::ExportDocumentQuery {
        database_name: "".to_string(),
        collection_ids: None,
        output_uri_prefix: "".to_string(),
    })
}
