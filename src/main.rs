use std::{collections::HashSet, iter::once};

use anyhow::{bail, Context, Result};
use lopdf::{Document, Object};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "PDF Annotation Fixer",
    about = "Fixing messed up PDF annotions. Use at your own risk!"
)]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let mut doc = Document::load(opt.input).context("unable to parse pdf document")?;
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
                Object::Array(a) => a
                    .iter()
                    .flat_map(Object::as_reference)
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
    doc.save(opt.output)
        .context("unable to save pdf document")?;
    println!("recovered {} annotations", recovered_annotations);
    Ok(())
}
