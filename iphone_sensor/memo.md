
# iphoneをセンサーデバイスとして使う

よくパズルゲームとかでジャイロ機能を使うゲームとかあり、どうやってるんだろうな〜と思って調べてみた

## どう取るの

iphone（というかAppleのOS）でセンサーデータを取りたい場合は[Core Motion](https://developer.apple.com/documentation/coremotion/)フレームワークを使うことになりそう

> Core Motion reports motion- and environment-related data from the available onboard hardware of iOS, iPadOS, watchOS, and visionOS devices. This hardware includes the device’s accelerometers and gyroscopes, and, when available, the pedometer, magnetometer, and barometer. Use this data in your app as input for user interactions, fitness tracking, health-related matters, and more. For example, a game might use accelerometer and gyroscope input to control onscreen game behavior


あんまりXCodeさわったことないけど、いい感じにPC側に送る方法あるかな...それともiphone内で完結するような何かしらを作るか...でも確かiphone向けのswiftのbuildって年額1万ぐらいの奴入らなきゃいけないような気が...
とか考えてたら、sensor loggerのアプリがあったので使ってみる。きっと内部では`core motion`つかっているんでしょう...  
'Core Motion`もいつか触る

## sensor loggerアプリでとってみる

### とりあえず概要を見てみる

[sensor logger](https://apps.apple.com/app/id1531582925)というまんますぎる名前のアプリがあります  
[storeから辿れるWebサイト](https://www.tszheichoi.com/sensorlogger)に飛ぶと結構しっかりしたサイトが  
Androidからも同様にセンサーデータが取れるようで、iPhoneとAndroidだと得られるセンサーデータの形式とか単位が違うらしく、(生のデータを取ってきてから)それを標準化しているぞ！というのも強みらしい。あまり意識したことがない問題でした

> CROSS-PLATFORM STANDARDISATION
> NO MORE BLACK BOXES: OPEN-SOURCE SENSOR ZOO

らへんの記述。

以下のようなエコシステム

![](https://images.squarespace-cdn.com/content/v1/54cbd20be4b09f43359af978/3a2bf892-66f1-4323-9e2a-461b99d6bf98/Screenshot+2026-03-14+at+20.22.55.png?format=2500w)

`Streaming HTTP`があるらしいのでこれ使ってPCに流す感じになりそう  
`MQTT`って初めて聞きましたが、IoT方面向けの軽量なpub/sub方式の通信プロトコルだそう。ROS2(というかDDSか)触った時にはかなり目新しい感覚だったけど、結構こういう思想のプロトコルあるんだ...

### 使ってみる

起動すると以下のような画面のはず。加速度、ジャイロ、スクロールしていくとApple Watchからの心拍数、バッテリーなど、結構色々ある

これらのtoggleを切り替えて`Start Recording`ボタンを押せば記録が始まる  
が、デフォルトだと記録後にcsv or Jsonでのexportしかできないので、http pushを有効にする

左下の⚙️→ Data Streamingで、Enable HTTP PushをOnにして、 Push URLに受け取りたいPCのIPとportを入れてあげればok

とりあえず何が来るかを確認してみる  

`main.rs`
```rust
use axum::{Json, Router, routing::post};
use serde_json::Value;

async fn receive(Json(payload): Json<Value>) -> &'static str {
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    "ok"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/data", post(receive));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("listening on :8000/data");
    axum::serve(listener, app).await.unwrap();
}
```

実行し、iphone側の設定にあった`Test Push`を使って導通を確認します  

```sh
$ cargo run

   Compiling iphone_sensor v0.1.0 (/Users/niiyama/ws/iphone_sensor)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.71s
     Running `target/debug/iphone_sensor`
listening on :8000/data
{
  "deviceId": "e79811dd-26d6-4e51-bc92-e90c6de6f02e",
  "messageId": 0,
  "payload": [
    {
      "name": "test",
      "time": 0,
      "values": []
    }
  ],
  "sessionId": "8adf3ce8-9822-409c-ae16-91d259deb476"
}
```

okそう  

`Accelerometer`をONにして`Start Recording`して、中身確認  

```sh
{
  "deviceId": "bdc48106-c638-4314-90e6-968e544894a5",
  "messageId": 0,
  "payload": [
    {
      "name": "accelerometeruncalibrated",
      "time": 1780530295467601700,
      "values": {
        "x": 0.0073089599609375,
        "y": -0.201873779296875,
        "z": -0.947174072265625
      }
    },
    {
      "name": "accelerometeruncalibrated",
      "time": 1780530295477644800,
      "values": {
        "x": 0.0020904541015625,
        "y": -0.2127838134765625,
        "z": -0.971893310546875
      }
    },
    (以降これがめっちゃ続く)

```

`Gravity`をみてみると

```txt
Sample { name: "gravity", time: 1781479958653153500, values: Values { x: Some(0.5609794303292408), y: Some(9.785193954995274), z: Some(-0.3250318357950076) } }
```

のようにでてきていて、端末が縦の状態でひっくり返すとy成分の正負が反転します  
ので、こんなものも作れる

