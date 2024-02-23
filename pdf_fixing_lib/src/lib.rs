use std::{
    collections::HashSet,
    io::{Read, Write},
    iter::once,
};

use anyhow::{bail, Context};
use lopdf::{Document, Object};

pub fn fix_pdf_annotations(input: impl Read, mut output: impl Write) -> anyhow::Result<usize> {
    let original_doc = Document::load_from(input).context("unable to parse pdf document")?;
    let mut doc = original_doc.clone();
    let pages = doc.get_pages();
    let reference_objects = doc
        .objects
        .values()
        .flat_map(Object::as_array)
        .filter(|o| o.iter().all(|o| o.as_reference().is_ok()))
        .map(|r| {
            r.iter()
                .flat_map(Object::as_reference)
                .collect::<HashSet<_>>()
        })
        .collect::<Vec<_>>();
    let mut recovered_annotations = 0;
    for page_id in pages.values() {
        let page = doc
            .get_object_mut(*page_id)
            .context("unable to get page object")?;
        let dict = page
            .as_dict_mut()
            .context("page object is not a dictionary")?;

        if let Ok(annots) = dict.get(b"Annots") {
            let annots = match annots {
                lopdf::Object::Array(a) => a
                    .iter()
                    .flat_map(Object::as_reference)
                    .filter(|o| !original_doc.get_object(*o).is_ok_and(Object::is_null))
                    .collect::<HashSet<_>>(),
                Object::Reference(r) => once(r).cloned().collect::<HashSet<_>>(),
                _ => bail!("annotations are neither an array nor a single reference"),
            };
            if let Some(replacement_annotations) = reference_objects
                .iter()
                .find(|r| annots.len() != r.len() && annots.is_subset(r))
            {
                dict.set(
                    b"Annots".to_vec(),
                    Object::Array(
                        replacement_annotations
                            .iter()
                            .copied()
                            .map(Object::Reference)
                            .collect::<Vec<_>>(),
                    ),
                );
                recovered_annotations += replacement_annotations.len() - annots.len()
            }
        }
    }
    doc.save_to(&mut output)
        .context("unable to save pdf document")?;
    Ok(recovered_annotations)
}
