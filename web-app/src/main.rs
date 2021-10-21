use std::collections::HashMap;

use yew::services::reader::{File, FileChunk, FileData, ReaderService, ReaderTask};
use yew::{classes, html, ChangeData, Component, ComponentLink, Html, ShouldRender};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type FileName = String;

pub enum Msg {
    Loaded((FileName, FileData)),
    Chunk((FileName, Option<FileChunk>)),
    Files(Vec<File>),
    ToggleByChunks,
}

enum PDFFixResult {
    Success {
        download_url: String,
        number_of_fixed_annotions: usize,
    },
    NoAnnotationsFixed,
    Error(anyhow::Error),
}

pub struct Model {
    link: ComponentLink<Model>,
    tasks: HashMap<FileName, ReaderTask>,
    result: Option<PDFFixResult>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            tasks: HashMap::default(),
            result: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Loaded((file_name, file)) => {
                let mut output = Vec::new();
                let result =
                    match pdf_fixing_lib::fix_pdf_annotations(file.content.as_slice(), &mut output)
                    {
                        Ok(0) => PDFFixResult::NoAnnotationsFixed,
                        Ok(number_of_fixed_annotions) => PDFFixResult::Success {
                            download_url: {
                                let uint8arr = js_sys::Uint8Array::new(
                                    &unsafe { js_sys::Uint8Array::view(&output) }.into(),
                                );
                                let array = js_sys::Array::new();
                                array.push(&uint8arr.buffer());
                                let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
                                    &array,
                                    web_sys::BlobPropertyBag::new().type_("application/pdf"),
                                )
                                .unwrap();
                                web_sys::Url::create_object_url_with_blob(&blob).unwrap()
                            },
                            number_of_fixed_annotions,
                        },
                        Err(e) => PDFFixResult::Error(e),
                    };
                self.result = Some(result);
                self.tasks.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let callback = self
                            .link
                            .callback(move |data| Msg::Loaded((file_name.clone(), data)));
                        ReaderService::read_file(file, callback).unwrap()
                    };
                    self.tasks.insert(file_name, task);
                }
                true
            }
            _ => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {

            <div class="hero">
            <div class="container">
                <div class="box">
                <div class="block">
                <div classes=classes!("file")>
                    <label class="file-label">
                        <input
                            type="file"
                            class="file-input"
                            multiple=false
                            accept=".pdf"
                            disabled={!self.tasks.is_empty()}
                            onchange=self.link.callback(move |value| {
                                let mut result = Vec::new();
                                if let ChangeData::Files(files) = value {
                                    let files = js_sys::try_iter(&files)
                                        .unwrap()
                                        .unwrap()
                                        .map(|v| File::from(v.unwrap()));
                                    result.extend(files);
                                }
                                Msg::Files(result)
                            })
                            />
                        <span class="file-cta">
                            <span class="file-label">
                            {"Choose PDF to Recover Annotations"}
                            </span>
                        </span>
                    </label>
                </div>
                </div>
                {
                    if let Some(ref result) = self.result {
                        match result {
                            PDFFixResult::Success {download_url, number_of_fixed_annotions} => {
                                html! {
                                    <>
                                    <div class="block">
                                        <a href={download_url.clone()} download="recovered.pdf" class=classes!("button", "is-success")>{"Save Recovered PDF"}</a>
                                    </div>
                                    <div class="block">
                                        <p>
                                            {format!("Successfully recovered {} annotations!", number_of_fixed_annotions)}
                                        </p>
                                    </div>
                                    </>
                                }
                            },
                            PDFFixResult::NoAnnotationsFixed => {
                                html! {
                                    <>
                                    <div class="block">
                                        <button disabled=true class=classes!("button", "is-warning")>{"No Annotations Recovered"}</button>
                                    </div>
                                    <div class="block">
                                        <p>
                                            {"Unable to recover annotations. This can have several reasons:"}
                                            <ul>
                                                <li>{"The PDF contains no annotations"}</li>
                                                <li>{"The PDF contains no lost annotations"}</li>
                                                <li>{"This site is unable to recover the lost annotations. "}<a href="https://github.com/julihoh/pdf_annotation_fix/issues/new">{"Please file an issue"}</a></li>
                                            </ul>
                                        </p>
                                    </div>
                                    </>
                                }
                            }
                            PDFFixResult::Error(e) => {
                                html! {
                                    <>
                                    <div class="block">
                                        <button disabled=true class=classes!("button", "is-error")>{"Internal Error :("}</button>
                                    </div>
                                    <div class="block">
                                        <p>
                                            {"Unable to recover annotions due to an internal error:"}
                                            <pre>
                                                { html_escape::encode_text(&format!("{:?}", e)) }
                                            </pre>
                                            <a href="https://github.com/julihoh/pdf_annotation_fix/issues/new">{"Please file an issue"}</a>
                                        </p>
                                    </div>
                                    </>
                                }
                            }
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
            </div>
            </div>

        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
