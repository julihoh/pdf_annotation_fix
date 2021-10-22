# PDF Annotation Fixer

macOS Preview sometimes 'forgets' about annotations that were added to a PDF file.
This can be particularly frustrating after providing feedback on a draft for many hours, just to loose the precious annotations when closing and re-opening the file.

This tool attempts to fix the annotations based on [a technique described by 'thorimur'](https://discussions.apple.com/thread/251532057?answerId=251532057021#251532057021).


## How to use

You can either use the command line app or try the [web app](https://julihoh.github.io/pdf_annotation_fix/web-app/dist/index.html) hosted on GitHub Pages.

### Web App

Thanks to WebAssembly, you can also use this tool from the browser [here](https://julihoh.github.io/pdf_annotation_fix/web-app/dist/index.html).

The web app runs entirely in the browser and the PDF is never sent anywhere.

The source code can be found under `web-app` directory of this repository.

### Command Line

You need to have a working [rust toolchain installed](https://www.rust-lang.org/tools/install).

Then just run the program using cargo:
```shell
~ cargo run -- my_messed_up.pdf fixed.pdf
# ...
recovered 188 annotations
```
