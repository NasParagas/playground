#!/usr/bin/env python3

import argparse

from _client import ApiError, get_json


def main():
    parser = argparse.ArgumentParser(
        description="Search for stations, stops, facilities, and addresses."
    )
    parser.add_argument("query", nargs="?", default="東京", help="search text")
    parser.add_argument("--limit", type=int, default=5, choices=range(1, 31))
    args = parser.parse_args()

    try:
        data = get_json(
            "/api/v1/places/suggest",
            {"q": args.query, "limit": args.limit},
        )
    except ApiError as error:
        parser.exit(1, f"error: {error}\n")

    places = data["places"]
    if not places:
        print("No places found.")
        return

    for index, place in enumerate(places, start=1):
        description = place.get("description")
        print(f"{index}. {place['name']} ({place['kind']}, {place['source']})")
        print(f"   endpoint: {place['endpoint']}")
        print(f"   location: {place['lat']:.6f}, {place['lon']:.6f}")
        if description:
            print(f"   description: {description}")


if __name__ == "__main__":
    main()
