pub fn handle_document_get(
    query: crate::DocumentQuery,
    ctx: crate::DatabaseContext,
) -> Result<(), String> {
    // ok, we want document at collection
    // lets print the debug output for now
    let document = ctx.get_document(query.collection_name, query.document_name)?;
    println!("{:#?}", document);
    Ok(())
}

pub fn handle_document_delete(
    query: crate::DocumentQuery,
    ctx: crate::DatabaseContext,
) -> Result<(), String> {
    let document = ctx.delete_document(query.collection_name, query.document_name);
    Ok(())
}

pub fn handle_collection_delete(
    query: crate::CollectionQuery,
    ctx: crate::DatabaseContext,
) -> Result<(), String> {

}

pub fn handle_document_view(
    query: crate::CollectionQuery,
    ctx: crate::DatabaseContext,
) -> Result<(), String> {
    /*
    page_size: i32,
        order_by: String,
        mask: DocumentMask,
        show_missing: bool,
        consistency_selector: ConsistencySelector,
        database_name: &str,
        collection_name: &str
        */
    dbg!(ctx.list_documents(
        0,
        String::new(),
        None,
        true,
        libfiresale::api::ConsistencySelector::ReadTime(chrono::Utc::now()),
        "(default)",
        "cars"
    ));
    Ok(())
}
