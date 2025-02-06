import argparse
import glob
import logging
import os
import time
import json
from tja import parse_tja, PyParsingMode


def main():
    parser = argparse.ArgumentParser(
        description="Process .tja files in a given directory."
    )
    parser.add_argument("directory", help="Directory to search for .tja files.")
    parser.add_argument(
        "--mode",
        help="Parsing mode. Full, MetadataOnly, or MetadataAndHeader",
        default="Full",
    )
    parser.add_argument(
        "--output", help="Output directory for the parsed files.", default=None
    )
    args = parser.parse_args()

    logging.basicConfig(
        level=logging.DEBUG, format="%(asctime)s [%(levelname)s] %(message)s"
    )

    search_pattern = os.path.join(args.directory, "**", "*.tja")
    files = glob.glob(search_pattern, recursive=True)
    logging.debug(f"Found {len(files)} .tja files.")

    if args.mode == "Full":
        mode = PyParsingMode.Full
    elif args.mode == "MetadataOnly":
        mode = PyParsingMode.MetadataOnly
    elif args.mode == "MetadataAndHeader":
        mode = PyParsingMode.MetadataAndHeader
    else:
        logging.error("Invalid parsing mode.")
        return

    songs = []

    total_start = time.perf_counter()
    for filepath in files:
        file_start = time.perf_counter()
        try:
            with open(filepath, "r", encoding="utf-8") as f:
                content = f.read()
        except UnicodeDecodeError:
            with open(filepath, "r", encoding="shift_jis") as f:
                content = f.read()
        result = parse_tja(content, mode)
        songs.append(result)
        file_end = time.perf_counter()
        logging.debug(f"Parsed {filepath} in {file_end - file_start:.4f} seconds.")
    total_end = time.perf_counter()

    if args.output:
        os.makedirs(args.output, exist_ok=True)
        for song in songs:
            filename = os.path.basename(song.metadata["TITLE"]) + ".json"
            with open(os.path.join(args.output, filename), "w") as f:
                export_start = time.perf_counter()
                exported = song.export()
                export_end = time.perf_counter()
                f.write(json.dumps(exported))
                logging.debug(
                    f"Exported {filename} in {export_end - export_start:.4f} seconds."
                )

    logging.debug(
        f"Total parsing time: {total_end - total_start:.4f} seconds. {len(songs)} TJAs parsed."
    )


if __name__ == "__main__":
    main()
