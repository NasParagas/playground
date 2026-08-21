#!/usr/bin/env python3

import argparse

from _client import ApiError, get_json


def main():
    parser = argparse.ArgumentParser(
        description="Find places near a latitude and longitude."
    )
    parser.add_argument("--lat", type=float, default=35.681)
    parser.add_argument("--lon", type=float, default=139.767)
    parser.add_argument("--radius", type=float, default=100, help="radius in meters")
    parser.add_argument("--limit", type=int, default=5, choices=range(1, 11))
    args = parser.parse_args()

    try:
        data = get_json(
            "/api/v1/places/reverse",
            {
                "lat": args.lat,
                "lon": args.lon,
                "radiusMeters": args.radius,
                "limit": args.limit,
            },
        )
    except ApiError as error:
        parser.exit(1, f"error: {error}\n")

    places = data["places"]
    if not places:
        print("No nearby places found.")
        return

    for index, place in enumerate(places, start=1):
        print(
            f"{index}. {place['name']} ({place['kind']}) "
            f"{place['distanceMeters']:.1f}m away"
        )
        print(f"   endpoint: {place['endpoint']}")


if __name__ == "__main__":
    main()
