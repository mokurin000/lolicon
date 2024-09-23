import os
import sys
import json


def read_file(path: str):
    with open(path, "r", encoding="utf-8") as f:
        c = json.load(f)
    return c


def main():
    if len(sys.argv) < 3:
        print("Usage:", sys.argv[0], "<dir> <tag_group1>..")
        exit(1)

    directory = sys.argv[1]
    tag_groups = sys.argv[2:]
    files = os.listdir(directory)

    metadata_list = (
        os.path.join(directory, file) for file in files if file.endswith(".json")
    )
    print("reading metadata...")
    metadata = map(read_file, metadata_list)

    pids = set()

    for meta in metadata:
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

    print(
        "\n".join(
            os.path.realpath(os.path.join(directory, file))
            for pid in sorted(pids)
            for file in files
            if file.startswith(str(pid)) and not file.endswith(".json")
        )
    )


if __name__ == "__main__":
    main()
