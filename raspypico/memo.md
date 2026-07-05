
## SDKのセットアップ

### mac

```sh
brew install libusb pkg-config cmake picotool
brew install --cask gcc-arm-embedded
git clone -b master https://github.com/raspberrypi/pico-sdk.git
cd pico-sdk
git submodule update --init
cd ..
git clone -b master https://github.com/raspberrypi/pico-examples.git
export PICO_SDK_PATH="~/ws/playground/study-raspypico/pico-sdk"
cd pico-examples
cmake -B build -DPICO_BOARD=pico2_w -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
cmake --build build -j8
```

(a) BOOTSEL ドラッグ: BOOTSELボタンを押しながらUSB接続 → RP2350 という名前でマウントされる → cp build/hello_usb.uf2 /Volumes/RP2350

### linux

```sh
sudo apt update && sudo apt install -y libusb-1.0-0-dev
git clone https://github.com/raspberrypi/picotool.git
git clone -b master https://github.com/raspberrypi/pico-sdk.git
cd pico-sdk
git submodule update --init 
cd ../
export PICO_SDK_PATH="$PWD/pico-sdk"
cmake -B picotool/build -S picotool
cmake --build picotool/build
ln -s "$PWD/picotool/build/picotool" ~/.local/bin/picotool
picotool version  # picotool v2.3.0 (Linux, GNU-15.2.0, Release)
```

## 動かす


```sh
ls /dev/tty.usbmodem*
screen $(ls /dev/tty.usbmodem*) 115200     # 終了は Ctrl-A → K
```


- 基本的には`while true`でずっと走らせるようにしておく(`main()`を抜けたあとの処理が未定義動作っぽくなる？らしい。確かにOSないから感覚はわかるかも？)
  - mainを終えた後の戻る場所？がないのか

## 参考になりそうな情報源

- https://pip-assets.raspberrypi.com/categories/610-raspberry-pi-pico/documents/RP-008276-DS-1-getting-started-with-pico.pdf
  - Appendix Bにcommand line utilityについて記載
- https://pip-assets.raspberrypi.com/categories/609-microcontroller-boards/documents/RP-009085-KB-1-raspberry-pi-pico-c-sdk.pdf
- https://github.com/raspberrypi/picotool
- https://github.com/raspberrypi/pico-sdk
- https://www.raspberrypi.com/documentation/microcontrollers/c_sdk.html
- Rust
  - https://pico.implrust.com
    - コミュニティらしいけどちゃんとしたドキュメントっぽい
  - https://github.com/rp-rs/rp-hal
    - 公式のらしい..?
    -
