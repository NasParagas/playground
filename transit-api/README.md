# Transit API サンプル

[`https://api.transit.ls8h.com`](https://api.transit.ls8h.com) を Python から利用する小さな CLI サンプルです。認証や外部パッケージは不要で、Python 3 の標準ライブラリだけで動きます。

## 実行方法

このディレクトリへ移動して、各スクリプトを `python3` で実行します。

### 駅・施設・住所を検索

```sh
python3 search_places.py 東京 --limit 5
python3 search_places.py 東京タワー
```

結果の `endpoint` は、`plan_journey.py` の `--from` または `--to` にそのまま渡せます。

路線フィードごとの駅 ID が必要な場合は、駅専用の検索を使います。

```sh
python3 search_stations.py 東京 --limit 5
```

### 座標付近の施設を検索

引数なしの場合は東京駅付近を検索します。

```sh
python3 nearby_places.py
python3 nearby_places.py --lat 35.6586 --lon 139.7454 --radius 200
```

### 経路を検索

引数なしの場合は東京駅付近から新宿駅付近までを検索します。

```sh
python3 plan_journey.py
python3 plan_journey.py \
  --from 'geo:35.681,139.767' \
  --to 'geo:35.6586,139.7454' \
  --time 09:00 \
  --limit 2
```

駅 ID も指定できます。ID は `search_stations.py` の検索結果から取得します。`search_places.py` が返す `endpoint` も指定可能です。

```sh
python3 plan_journey.py --from '<検索結果の endpoint>' --to 'geo:35.690,139.700'
```

`--type` には `departure`、`arrival`、`first`、`last` を指定できます。日付を固定する場合は `--date YYYYMMDD` を追加します。

### 駅の発車案内を取得

引数なしの場合は東京駅の中央線快速を表示します。

```sh
python3 station_departures.py --limit 5
python3 station_departures.py '<feedId:stopId>' --time 18:00
```

`feedId:stopId` は `search_stations.py` の `id` から取得できます。

データ提供条件により、発車案内を公開していないフィードでは `403` が返る場合があります。

## ファイル構成

- `_client.py`: GET リクエスト、API エラー、時刻表示の共通処理
- `search_places.py`: `/api/v1/places/suggest` の例
- `search_stations.py`: `/api/v1/locations/suggest` の例
- `nearby_places.py`: `/api/v1/places/reverse` の例
- `plan_journey.py`: `/api/v1/plan` の例
- `station_departures.py`: `/api/v1/stations/{id}/departures` の例

接続先は環境変数 `TRANSIT_API_BASE_URL` で変更できます。

```sh
TRANSIT_API_BASE_URL='https://api.transit.ls8h.com' python3 plan_journey.py
```

## API 利用時の注意

- 駅・停留所 ID は `feedId:stopId` 形式です。
- 座標は `geo:<緯度>,<経度>` 形式で経路検索へ渡します。
- 時刻はサービス日の 0:00 からの秒数で、翌日の列車は `24:00` 以上になる場合があります。
- データのライセンスと帰属は [`/api/v1/feeds`](https://api.transit.ls8h.com/api/v1/feeds) と [`/api/v1/operators`](https://api.transit.ls8h.com/api/v1/operators) を確認してください。
- 詳細は [API リファレンス](https://api.transit.ls8h.com/api/docs) と [利用規約](https://transit.ls8h.com/terms) を参照してください。
