#!/usr/bin/env python3

import argparse

from _client import ApiError, get_json


def main():
    parser = argparse.ArgumentParser(
        description="Search for stations and their feed-qualified IDs."
    )
    parser.add_argument("query", nargs="?", default="東京", help="station name")
    parser.add_argument("--limit", type=int, default=5, choices=range(1, 31))
    args = parser.parse_args()

    try:
        data = get_json(
            "/api/v1/locations/suggest",
            {"q": args.query, "limit": args.limit},
        )
    except ApiError as error:
        parser.exit(1, f"error: {error}\n")

    stations = data["stations"]
    if not stations:
        print("No stations found.")
        return

    for index, station in enumerate(stations, start=1):
        print(f"{index}. {station['name']} ({station['feedName']})")
        print(f"   id: {station['id']}")
        if "lat" in station and "lon" in station:
            print(f"   location: {station['lat']:.6f}, {station['lon']:.6f}")


if __name__ == "__main__":
    main()
