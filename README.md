# Image Importer

Parses the filenames and metadata for all files in a directory (recursively) and moves them to another directory with a temporal hierarchy.
The canonical use-case is to import images from cameras or phones, but it could also be used for organising an existing library, or on other types of files than images.


## Usage

To import all files from the input directory to the output directory use:  
```image_importer input output```

To learn more about all options available use:  
```image_importer --help```


## Date Parsing

By default the software parses both the filenames and metadata (oldest date).
With flags this can be changed to only consider filenames or metadata.

The 19th of May 2019 can be recognised, if the filename contains any of these patterns:

| Year-Month-Day | Day-Month-Year |
|----------------|----------------|
| \*2019-05-19\* | \*19-05-2019\* |
| \*2019_05_19\* | \*19_05_2019\* |
|  \*20190519\*  |  \*19052019\*  |
| \*2019 05 19\* | \*19 05 2019\* |


## Temporal Structure

When the files are moved to the output directory, a temporal hierarchy is created.
The names of the directories can be customised with command line arguments:

| Directory Structure | Arguments |
|---|---|
| 2019 / 05 / image.jpg | |
| 2019 / 2019-05 / image.jpg | -y |
| 2019-05 / image.jpg | -f |
| 2019 / 05 May / image.jpg | -m en |
| 2019 / 2019-05 Maj / image.jpg | -y -m swe |


## Example

```{sh}
mkdir example1
touch example1/2005-07-14.txt
image_importer example1 example2 --clean --flat
rm example2/2005-07/2005-07-14.txt
rm -rf example1 example2
```


## Executable

To build this software: install Rust, download this project, and run `cargo build --release`.

Alternatively, some prebuilt binaries can be found in [releases](/releases).
