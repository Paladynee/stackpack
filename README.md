# stackpack

Stackpack is a program to build compresion pipelines and create your own file formats.

## Building

Currently unavailable as bsc_m03_sys hasn't been released (by me) yet.

## Usage

`stackpack` has a very rich CLI. Basic commands include:

```sh
# process a single file
$ stackpack encode <input> <output> [ --using "pipeline -> string" | --from_file <path to pipeline file> | --preset <preset name> ]

# undo process a single file
$ stackpack decode <input> <output> [ --using "pipeline -> string" | --from_file <path to pipeline file> | --preset <preset name> ]

# test roundtrip process of single file
$ stackpack test [ --using "pipeline -> string" | --from_file <path to pipeline file> | --preset <preset name> ]

# test rountrip of many files
$ stackpack corpus <path>

# pipeline management
$ stackpack pipeline [ --list-compressors | --list-plugins ]
```

## Current Compressors

Stackpack currently ships with 5 built-in compressors and a plugin system allowing you to make your own plugins. An example plugin can be found in the `sample_plugin` directory. Currently, the only requirements are the 4 static symbols for name, description, encode and decode implementations.

Already implemented compressors are:

1. Arithmetic Coding
1. Burrows Wheeler Transform by Ilya Grabnov (using libsais)
1. Bsc-m03 by Ilya Grebnov
1. Move-to-Front transform

## Current goals

- implement generic bytestream decoders to work with things easier
- image decoders for custom image formats
- embedding pipelines in files (semantic versioned, hash of the dynamic library or something?)
- compressors that already have bindings in rust, feature gated
