import os
import sys
import json
import multiprocessing
from concurrent.futures import ThreadPoolExecutor

NUM_CPU = multiprocessing.cpu_count()


def read_file(path: str):
    with open(path, "r", encoding="utf-8") as f:
        c = json.load(f)
    return c


def main():
    if len(sys.argv) < 2 or sys.argv[1] in ["-h", "--help"]:
        print("Usage:")
        print(sys.argv[0], "<dir> <tag_group1>..")
        print(sys.argv[0], "<dir>")
        exit(1)

    directory = sys.argv[1]
    tag_groups = sys.argv[2:]
    files = os.listdir(directory)

    metadata_list = (
        os.path.join(directory, file) for file in files if file.endswith(".json")
    )

    if not tag_groups:
        print(f"metainfo count: {sum(map(lambda _: 1, metadata_list))}")
        return

    print("reading metadata...")
    executor = ThreadPoolExecutor(max_workers=NUM_CPU)
    metadata = executor.map(read_file, metadata_list)

    pids = set()

    def filter_pid(meta):
        for tag_group in tag_groups:
            if not any(
                map(
                    lambda tag: tag in meta["tags"],
                    map(lambda t: t.strip(), tag_group.split("|")),
                ),
            ):
                break
        else:
            pids.add(meta["pid"])

    executor.map(filter_pid, metadata)
    image_paths = []

    for pid in sorted(pids):
        related_paths = list(
            os.path.realpath(os.path.join(directory, file))
            for file in files
            if file.startswith(str(pid)) and not file.endswith(".json")
        )
        if related_paths:
            image_paths.extend(related_paths)
        else:
            print(f"WARN: lost image: https://www.pixiv.net/artworks/{pid}")

    print(*image_paths, sep="\n")
    print("count:", len(image_paths))


if __name__ == "__main__":
    main()
