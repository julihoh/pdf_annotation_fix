use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::{classes, html, ChangeData, Component, ComponentLink, Html, ShouldRender};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type FileName = String;

pub enum Msg {
    Loaded((FileName, FileData)),
    Files(Vec<File>),
    ToggleByChunks,
}

enum PDFFixResult {
    Success {
        download_url: String,
        number_of_fixed_annotions: usize,
        download_filename: String,
    },
    NoAnnotationsFixed,
    Error(anyhow::Error),
}

pub struct Model {
    link: ComponentLink<Model>,
    task: Option<ReaderTask>,
    result: Option<PDFFixResult>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            task: None,
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
                            download_filename: file_name.replace(".pdf", "_recovered.pdf"),
                        },
                        Err(e) => PDFFixResult::Error(e),
                    };
                self.result = Some(result);
                self.task = None;
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
                    self.task = Some(task);
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
        let download_button_text = "2. Save Recovered PDF";
        let save_button = match &self.result {
            Some(PDFFixResult::Success {
                download_filename,
                download_url,
                ..
            }) => {
                html! {
                    <a
                        class=classes!("button")
                        href={download_url.clone()}
                        download={download_filename.clone()}
                    >
                        {download_button_text}
                    </a>
                }
            }
            Some(PDFFixResult::NoAnnotationsFixed) => {
                html! {
                    <a
                        class=classes!("button")
                        disabled=true
                    >
                        {download_button_text}
                    </a>
                }
            }
            Some(PDFFixResult::Error(_)) => {
                html! {
                    <a
                        class=classes!("button")
                        disabled=true
                    >
                        {download_button_text}
                    </a>
                }
            }
            None => {
                html! {
                    <a
                        class=classes!("button")
                        disabled=true
                    >
                        {download_button_text}
                    </a>
                }
            }
        };

        let result_message = match &self.result {
            Some(PDFFixResult::Success {
                number_of_fixed_annotions,
                ..
            }) => Some((
                format!(
                    "Successfully Recovered {} Annotations :)",
                    number_of_fixed_annotions
                ),
                "is-success",
                html! {
                    {"Hello!"}
                },
            )),
            Some(PDFFixResult::NoAnnotationsFixed) => Some((
                "No Annotations Found".to_string(),
                "is-warning",
                html! {
                    <p class="content">
                        {"Unable to recover annotations. This can have several reasons:"}
                        <ul>
                            <li>{"The PDF contains no annotations"}</li>
                            <li>{"The PDF contains no lost annotations"}</li>
                            <li>{"The annotations are lost in such a way that this site can't recover. "}</li>
                        </ul>
                        <a href="https://github.com/julihoh/pdf_annotation_fix/issues/new">{"Please file an issue"}</a>
                    </p>
                },
            )),
            Some(PDFFixResult::Error(e)) => Some((
                "Internal Error".to_string(),
                "is-danger",
                html! {
                    <p>
                        {"Unable to recover annotions due to an internal error:"}
                        <pre>
                            { html_escape::encode_text(&format!("{:?}", e)) }
                        </pre>
                        <a href="https://github.com/julihoh/pdf_annotation_fix/issues/new">{"Please file an issue"}</a>
                    </p>
                },
            )),
            None => None,
        };

        let file_selector = html! {
            <div classes=classes!("file")>
                <label class="file-label">
                    <input
                        type="file"
                        class="file-input"
                        multiple=false
                        accept=".pdf"
                        disabled={self.task.is_some()}
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
                        {"1. Choose PDF to Recover Annotations"}
                        </span>
                    </span>
                </label>
            </div>
        };

        html! {
            <>
            <div id="wrapper">
                <div class="mx-4">
                    <div class="columns is-desktop is-centered">
                        <div class="column is-half">
                            <h1 class="title">{"PDF Annotation Recovery"}</h1>
                            <div class="block">{"Recover PDF annotations in two simple steps:"}</div>
                            <div class="block">{file_selector}</div>
                            <div class="block">{save_button}</div>
                            <div class="block">
                            {
                                if let Some((heading, class, content)) = result_message {
                                    html! {
                                            <article class=classes!("message", class)>
                                                <div class="message-header">
                                                    <p>{heading}</p>
                                                </div>
                                                <div class="message-body">
                                                    {content}
                                                </div>
                                            </article>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            </div>
                            <article class="message is-info">
                                <div class="message-header">
                                    <p>{"Privacy Info"}</p>
                                </div>
                                <div class="message-body">
                                    <div class="content">
                                        {"This site does not track you or gather any information about you."}
                                        <ul>
                                            <li>
                                                {"No 🍪s whatsoever."}
                                            </li> 
                                            <li>
                                                {"Thanks to "}
                                                <a href="https://webassembly.org">{"WebAssembly"}</a>
                                                {" and "} <a href="https://www.rust-lang.org">{"Rust"}</a>
                                                {", your PDF is processed on your device and can not be tracked by anyone."}
                                            </li>
                                        </ul>
                                    </div>
                                </div>
                            </article>
                        </div>
                    </div>
                </div>
            </div>
            <footer class="footer">
                <div class="content has-text-centered">
                    <p>
                        <strong>{"PDF Annotation Recovery Tool"}</strong> 
                        {" by "} 
                        <a href="https://twitter.com/julihoh_">{"@julihoh_"}</a>
                        {". The source code is licensed "}
                        <a href="http://opensource.org/licenses/mit-license.php">{"MIT"}</a>
                        {" and is available at "} 
                        <a href="https://github.com/julihoh/pdf_annotation_fix">{"GitHub"}</a>
                        {"."}
                    </p>
                </div>
            </footer>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
