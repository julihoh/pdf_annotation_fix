use gloo_file::{callbacks::FileReader, File};
use web_sys::{HtmlInputElement, Event};
use yew::{classes, html, html::TargetCast, Component, Html};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type FileName = String;

pub enum Msg {
    Loaded((FileName, Vec<u8>)),
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
    task: Option<FileReader>,
    result: Option<PDFFixResult>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            task: None,
            result: None,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded((file_name, file)) => {
                self.result = Some(fix_pdf(file, file_name));
                self.task = None;
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded((
                                file_name,
                                res.expect("failed to read file"),
                            )))
                        })
                    };
                    self.task = Some(task);
                }
                true
            }
            _ => false,
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let download_button_text = "2. Save Recovered PDF";
        let save_button = match &self.result {
            Some(PDFFixResult::Success {
                download_filename,
                download_url,
                ..
            }) => {
                html! {
                    <a
                        class={classes!("button")}
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
                        class={classes!("button")}
                        disabled={true}
                    >
                        {download_button_text}
                    </a>
                }
            }
            Some(PDFFixResult::Error(_)) => {
                html! {
                    <a
                        class={classes!("button")}
                        disabled={true}
                    >
                        {download_button_text}
                    </a>
                }
            }
            None => {
                html! {
                    <a
                        class={classes!("button")}
                        disabled={true}
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
                "Success!".to_string(),
                "is-success",
                html! {
                    {format!(
                        "Successfully recovered {} annotations :)",
                        number_of_fixed_annotions
                    )}
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
            <div classes={classes!("file")}>
                <label class="file-label">
                    <input
                        type="file"
                        class="file-input"
                        multiple={false}
                        accept=".pdf"
                        disabled={self.task.is_some()}
                        onchange={
                            ctx.link().callback(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                let mut result = Vec::new();
                                if let Some(files) = input.files() {
                                    let files = js_sys::try_iter(&files)
                                        .unwrap()
                                        .unwrap()
                                        .map(|v| web_sys::File::from(v.unwrap()))
                                        .map(File::from);
                                    result.extend(files);
                                }
                                Msg::Files(result)
                            })
                        }
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
                                            <article class={classes!("message", class)}>
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
                                    <p>{"Privacy"}</p>
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
                            <article class="message is-warning">
                                <div class="message-header">
                                    <p>{"Support ❤️"}</p>
                                </div>
                                <div class="message-body">
                                    <div class="content">
                                        {"Did this tool help you? Consider helping others find it!"}
                                        <ul>
                                            <li>
                                                {"Mark this "}
                                                <a href="https://discussions.apple.com/thread/253093013?answerId=255798424022#255798424022">{"reply"}</a>
                                                {" on the Apple Community Forum as helpful."}
                                            </li>
                                            <li>
                                                {"Star the "}
                                                <a href="https://github.com/julihoh/pdf_annotation_fix">{"repository"}</a>
                                                {" on GitHub."}
                                            </li>
                                            <li>
                                                {"Tweet about it! You can tag me "}
                                                <a href="https://twitter.com/julihoh_">{"@julihoh_"}</a>
                                                {"."}
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

fn fix_pdf(file: Vec<u8>, file_name: String) -> PDFFixResult {
    let mut output = Vec::new();
    match pdf_fixing_lib::fix_pdf_annotations(file.as_slice(), &mut output) {
        Ok(0) => PDFFixResult::NoAnnotationsFixed,
        Ok(number_of_fixed_annotions) => PDFFixResult::Success {
            download_url: create_download_url(output),
            number_of_fixed_annotions,
            download_filename: file_name.replace(".pdf", "_recovered.pdf"),
        },
        Err(e) => PDFFixResult::Error(e),
    }
}

fn create_download_url(output: Vec<u8>) -> String {
    let uint8arr = js_sys::Uint8Array::new(&unsafe { js_sys::Uint8Array::view(&output) }.into());
    let array = js_sys::Array::new();
    array.push(&uint8arr.buffer());
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &array,
        web_sys::BlobPropertyBag::new().type_("application/pdf"),
    )
    .unwrap();
    web_sys::Url::create_object_url_with_blob(&blob).unwrap()
}

fn main() {
    yew::Renderer::<Model>::new().render();
}
