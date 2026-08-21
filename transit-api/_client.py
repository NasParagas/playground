#!/usr/bin/env python3

import json
import os
import socket
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode
from urllib.request import Request, urlopen


BASE_URL = os.environ.get(
    "TRANSIT_API_BASE_URL", "https://api.transit.ls8h.com"
).rstrip("/")


class ApiError(Exception):
    pass


def get_json(path, params=None):
    query = urlencode(params or {}, doseq=True)
    url = f"{BASE_URL}{path}"
    if query:
        url = f"{url}?{query}"

    request = Request(
        url,
        headers={
            "Accept": "application/json",
            "User-Agent": "transit-api-samples/1.0",
        },
    )

    try:
        with urlopen(request, timeout=60) as response:
            return json.load(response)
    except HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        try:
            detail = json.loads(body).get("error", body)
        except json.JSONDecodeError:
            detail = body
        raise ApiError(f"HTTP {error.code}: {detail}") from error
    except (URLError, TimeoutError, socket.timeout) as error:
        raise ApiError(f"Request failed: {error}") from error


def format_service_time(seconds):
    value = round(seconds)
    sign = "-" if value < 0 else ""
    hours, remainder = divmod(abs(value), 3600)
    minutes, secs = divmod(remainder, 60)
    return f"{sign}{hours:02}:{minutes:02}:{secs:02}"


def format_duration(seconds):
    minutes = round(seconds / 60)
    hours, minutes = divmod(minutes, 60)
    if hours:
        return f"{hours}h {minutes}m"
    return f"{minutes}m"
