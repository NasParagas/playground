#!/usr/bin/env python3

import argparse
from urllib.parse import quote

from _client import ApiError, format_service_time, get_json


DEFAULT_STATION_ID = (
    "scrape-jreast-chuo-rapid:"
    "odpt.Station:JR-East.ChuoRapid.Tokyo"
)


def main():
    parser = argparse.ArgumentParser(description="Show upcoming station departures.")
    parser.add_argument(
        "station_id",
        nargs="?",
        default=DEFAULT_STATION_ID,
        help="feed-qualified station ID",
    )
    parser.add_argument("--date", help="service date in YYYYMMDD format")
    parser.add_argument("--time", help="time in HH:MM or HH:MM:SS format")
    parser.add_argument("--limit", type=int, default=10, choices=range(1, 101))
    args = parser.parse_args()

    params = {"limit": args.limit}
    if args.date:
        params["date"] = args.date
    if args.time:
        params["time"] = args.time

    station_id = quote(args.station_id, safe="")
    try:
        data = get_json(f"/api/v1/stations/{station_id}/departures", params)
    except ApiError as error:
        parser.exit(1, f"error: {error}\n")

    print(f"Station: {data['stationId']}")
    print(f"Date: {data['date']} ({data['timezone']})")

    departures = data["departures"]
    if not departures:
        print("No departures found.")
        return

    for departure in departures:
        time = format_service_time(departure["departureSecs"])
        headsign = departure.get("headsign", "destination unavailable")
        train_type = departure.get("trainType")
        label = departure["routeName"]
        if train_type:
            label += f" / {train_type}"
        if departure["headwayBased"]:
            label += " (headway-based)"
        print(f"{time}  {label}  {headsign}")


if __name__ == "__main__":
    main()
