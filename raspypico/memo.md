SDkのセットアップ

```sh
brew install cmake picotool
brew install --cask gcc-arm-embedded
git clone -b master https://github.com/raspberrypi/pico-sdk.git
cd pico-sdk
git submodule update --init     # ← cyw43(WiFi/LED)とtinyusb(USB)に必須。忘れるとビルド失敗
cd ..
git clone -b master https://github.com/raspberrypi/pico-examples.git
export PICO_SDK_PATH="~/ws/playground/study-raspypico/pico-sdk"
cd pico-examples
cmake -B build -DPICO_BOARD=pico2_w -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
cmake --build build -j8
```

(a) BOOTSEL ドラッグ: BOOTSELボタンを押しながらUSB接続 → RP2350 という名前でマウントされる → cp build/hello_usb.uf2 /Volumes/RP2350

```sh
ls /dev/tty.usbmodem*
screen $(ls /dev/tty.usbmodem*) 115200     # 終了は Ctrl-A → K
```


- 基本的には`while true`でずっと走らせるようにしておく(`main()`を抜けたあとの処理が未定義動作っぽくなる？らしい。確かにOSないから感覚はわかるかも？)
  - mainを終えた後の戻る場所？がないのか

## 参考になりそうな情報源

- https://github.com/raspberrypi/picotool
- https://github.com/raspberrypi/pico-sdk
- https://www.raspberrypi.com/documentation/microcontrollers/c_sdk.html
- Rust
  - https://pico.implrust.com
    - コミュニティらしいけどちゃんとしたドキュメントっぽい
  - https://github.com/rp-rs/rp-hal
    - 公式のらしい..?
    -
