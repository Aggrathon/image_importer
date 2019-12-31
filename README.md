# Image Importer

Parses the filenames and metadata for all files in a directory (recursively) and moves them to another directory with a temporal hierarchy.
The canonical use-case is to import images from cameras or phones, but it could also be used for organising an existing library, or on other types of files than images.


## Usage

To import all files from the input directory to the output directory use:  
```image_importer input output```

To learn more about all options available use:  
```image_importer --help```

## Date Parsing

By default the software first parses the filenames (first match), and then falls back to metadata (oldest date).
With flags this can be changed to only consider filenames *or* metadata.

The 19th of May 2019 can be recognised, if the filenames have any of these patterns:

- \*2019-05-19\*
- \*2019_05_19\*
- \*20190519\*
- \*2019 05 19\*
- \*2019.05.19\*
- \*19-05-2019\*
- \*19_05_2019\*
- \*19052019\*
- \*19 05 2019\*
- \*19.05.2019\*


## Temporal Structure

When the files are moved to the output directory, a temporal hierarchy is created.
The software comes with a couple of different options that can be toggled with a command line parameter:

| Directory Structure | Tag |
|---|---|
| 2019 / 05 / image.jpg | Y_M |
| 2019 / 2019-05 / image.jpg | Y_YM |
| 2019-05 / image.jpg | YM |
| 2019 / 05 May / image.jpg | Y_Meng |
| 2019 / 05 Maj / image.jpg | Y_Mswe |


## Executable

To build this software: install Rust, download this project, and run `cargo build --release`.

Alternatively, some prebuilt binaries can be found in [releases](/releases).
