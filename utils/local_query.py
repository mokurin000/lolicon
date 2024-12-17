import os
import json
import argparse
import logging
import multiprocessing
from concurrent.futures import ThreadPoolExecutor

NUM_CPU = multiprocessing.cpu_count()

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)


def read_file(path: str):
    try:
        with open(path, "r", encoding="utf-8") as f:
            c = json.load(f)
    except json.decoder.JSONDecodeError:
        return None
    return c


def main():
    parser = argparse.ArgumentParser(
        description="Process metadata and images from a directory"
    )
    parser.add_argument(
        "directory", type=str, help="Directory containing metadata and images"
    )
    parser.add_argument(
        "tag_groups", type=str, nargs="*", help="Groups of tags to filter images"
    )
    parser.add_argument(
        "--link-dir", type=str, help="Directory to create hard links to the images"
    )

    args = parser.parse_args()

    directory = args.directory
    tag_groups = args.tag_groups
    link_dir = args.link_dir
    files = os.listdir(directory)

    metadata_list = (
        os.path.join(directory, file) for file in files if file.endswith(".json")
    )

    if not tag_groups:
        logging.info(f"metainfo count: {sum(map(lambda _: 1, metadata_list))}")
        return

    logging.info("Reading metadata...")
    executor = ThreadPoolExecutor(max_workers=NUM_CPU)
    metadata = executor.map(read_file, metadata_list)

    def filter_pid(meta):
        if meta is None:
            return None
        for tag_group in tag_groups:
            if not any(
                map(
                    lambda tag: tag in meta["tags"],
                    map(lambda t: t.strip(), tag_group.split("|")),
                ),
            ):
                return None
        else:
            return meta["pid"]

    pids = set(executor.map(filter_pid, metadata))
    pids.remove(None)

    image_paths = []

    def check_path(pid):
        related_paths = list(
            os.path.realpath(os.path.join(directory, file))
            for file in files
            if file.startswith(str(pid)) and not file.endswith(".json")
        )

        return related_paths

    for pid in pids:
        image_paths.extend(check_path(pid))

    if link_dir:
        os.makedirs(link_dir, exist_ok=True)
        for image_path in image_paths:
            link_path = os.path.join(link_dir, os.path.basename(image_path))
            if not os.path.exists(link_path):
                os.link(image_path, link_path)
        logging.info("Hard links created.")
    else:
        print(*image_paths, sep="\n")

    logging.info(f"Count: {len(image_paths)}")


if __name__ == "__main__":
    main()
