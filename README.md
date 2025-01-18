# tja-rs

An efficient TJA file parser, written in Rust, that supports Rust, Python, and WebAssembly environments.

It is highly optimized for speed and includes features such as a synthesizer for synthesizing music along with don/ka sound effects from a TJA file.

It's fast! [Check out the benchmark](https://jacoblincool.github.io/tja-rs/report/).

## Building Instructions

### Rust

The Rust target requires no additional feature flags.

To build the library, run:

```sh
cargo build
```

To build the CLI tool, run:

```sh
cargo build --bin tja
```

### Python

We use `maturin` to build the Python package. The Python package requires the `python` feature flag to be enabled.

To build the Python package `.whl`, run: 

```sh
maturin build
```

To develop and test, run:

```sh
maturin develop -F python
```

> Python virtual environment is necessary. (e.g. `conda`, `micromamba`, `poetry`, `pixi`)

#### Schema
```json
{
    "title": "PyParsedTJA",
    "type": "class",
    "properties": {
        "charts": {
            "type": "list",
            "items": {
                "PyChart": {
                    "type": "class",
                    "properties": {
                        "player": {
                            "type": "int"
                        },
                        "course": {
                            "type": ["str", "None"]
                        },
                        "balloon": {
                            "type": "list",
                            "items": {
                                "type": "int"
                            }
                        },
                        "level": {
                            "type": ["int", "None"]
                        },
                        "header": {
                            "type": "dict",
                            "properties": {
                                "BALLOON": {
                                    "type": "string"
                                },
                                "COURSE": {
                                    "type": "string"
                                },
                                "LEVEL": {
                                    "type": "string"
                                },
                                "SCOREDIFF": {
                                    "type": "string"
                                },
                                "SCOREINIT": {
                                    "type": "string"
                                }
                            }
                        },
                        "segment": {
                            "type": "list",
                            "items": {
                                "PySegment": {
                                    "type": "class",
                                    "properties": {
                                        "measure_num": {
                                            "type": "int"
                                        },
                                        "measure_den": {
                                            "type": "int"
                                        },
                                        "barline": {
                                            "type": "bool"
                                        },
                                        "branch": {
                                            "type": ["str", "None"]
                                        },
                                        "branch_condition": {
                                            "type": ["str", "None"]
                                        },
                                        "notes": {
                                            "type": "list",
                                            "items": {
                                                "PyNote": {
                                                    "type": "class",
                                                    "properties": {
                                                        "note_type": {
                                                            "type": "str"
                                                        },
                                                        "timestamp": {
                                                            "type": "float"
                                                        },
                                                        "scroll": {
                                                            "type": "float"
                                                        },
                                                        "delay": {
                                                            "type": "float"
                                                        },
                                                        "bpm": {
                                                            "type": "float"
                                                        },
                                                        "gogo": {
                                                            "type": "bool"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                    }
                }
            }
        }
    },
    "metadata": {
        "title": "dict",
        "type": "object",
        "properties": {
            "SUBTITLE": {
                "type": "string"
            },
            "OFFSET": {
                "type": "string"
            },
            "GENRE": {
                "type": "string"
            },
            "DEMOSTART": {
                "type": "string"
            },
            "BPM": {
                "type": "string"
            },
            "TITLE": {
                "type": "string"
            },
            "WAVE": {
                "type": "string"
            }
        }
    }
}
```

### WebAssembly

We use `wasm-pack` to build the WebAssembly package. The WebAssembly package requires the `wasm` feature flag to be enabled.

To build the WebAssembly package, run:

```sh
wasm-pack build --features wasm
```

## Performance Benchmarks

The parser is highly optimized for performance.

It can parse a typical TJA file in under 1 ms in full mode, and in metadata-only mode in under 5 µs.

For detailed benchmarks and comparisons, check out our [benchmark report](https://jacoblincool.github.io/tja-rs/report/).

To run the benchmark:

```sh
cargo bench
```

## Synthesizer

The TJA parser includes a synthesizer binary that can synthesize music along with don/ka sound effects from a TJA file:

```sh
cargo run -F audio --bin synthesize <TJA file> <music file> <don sound file> <ka sound file> --course <course> --branch <branch>
```
