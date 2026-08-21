#!/usr/bin/env python3

import argparse

from _client import ApiError, format_duration, format_service_time, get_json


def main():
    parser = argparse.ArgumentParser(description="Plan a public-transit journey.")
    parser.add_argument(
        "--from",
        dest="origin",
        default="geo:35.681,139.767",
        help="station ID or geo:<lat>,<lon>",
    )
    parser.add_argument(
        "--to",
        dest="destination",
        default="geo:35.690,139.700",
        help="station ID or geo:<lat>,<lon>",
    )
    parser.add_argument("--date", help="service date in YYYYMMDD format")
    parser.add_argument("--time", help="time in HH:MM or HH:MM:SS format")
    parser.add_argument(
        "--type",
        choices=("departure", "arrival", "first", "last"),
        default="departure",
    )
    parser.add_argument("--max-transfers", type=int, default=3, choices=range(0, 9))
    parser.add_argument("--limit", type=int, default=3, choices=range(1, 7))
    args = parser.parse_args()

    params = {
        "from": args.origin,
        "to": args.destination,
        "type": args.type,
        "maxTransfers": args.max_transfers,
        "numItineraries": args.limit,
    }
    if args.date:
        params["date"] = args.date
    if args.time:
        params["time"] = args.time

    try:
        data = get_json("/api/v1/plan", params)
    except ApiError as error:
        parser.exit(1, f"error: {error}\n")

    print(
        f"{data['from']['name']} -> {data['to']['name']} "
        f"({data['date']}, {data['timezone']})"
    )

    journeys = data["journeys"]
    if not journeys:
        print("No journeys found.")
        return

    for index, journey in enumerate(journeys, start=1):
        departure = format_service_time(journey["departureSecs"])
        arrival = format_service_time(journey["arrivalSecs"])
        summary = (
            f"{index}. {departure} -> {arrival}, "
            f"{format_duration(journey['durationSecs'])}, "
            f"{journey['transferCount']} transfer(s)"
        )
        fare = journey.get("fare")
        if fare:
            summary += f", {fare['ticket']} {fare['currency']}"
        print(summary)

        access_walk = journey.get("accessWalkSecs", 0)
        if access_walk:
            print(f"   walk to first stop: {format_duration(access_walk)}")

        for leg in journey["legs"]:
            leg_departure = format_service_time(leg["departureSecs"])
            leg_arrival = format_service_time(leg["arrivalSecs"])
            if leg["kind"] == "walk":
                label = "walk"
            else:
                label = leg["routeName"]
                if leg.get("trainType"):
                    label += f" / {leg['trainType']}"
                if leg.get("headsign"):
                    label += f" toward {leg['headsign']}"
            print(
                f"   {leg_departure}-{leg_arrival} {label}: "
                f"{leg['from']['name']} -> {leg['to']['name']}"
            )

        egress_walk = journey.get("egressWalkSecs", 0)
        if egress_walk:
            print(f"   walk from last stop: {format_duration(egress_walk)}")


if __name__ == "__main__":
    main()
