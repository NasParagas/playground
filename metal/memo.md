## (備考)beta版への切り替え

- https://developer.apple.com/account へアクセス
- 普段使っているApple IDでサインイン
- Developer Agreementに同意
  - とりあえず使うだけなら金はかからんです
- developer.apple.com/downloadページから、いま入れているmacOS betaに対応するXcode betaがダウンロードできる

## 環境構築

以下が実行できるか確認

```sh
xcrun -sdk macosx metal --version
```

```sh
xcrun: error: unable to find utility "metal", not a developer tool or in PATH
```

これはできてない

```sh
# TODO: ??
# 私はmacosがbeta版なので`Xcode-beta`ですが、betaじゃないなら`Xcode`のはず
sudo xcode-select -s /Applications/Xcode-beta.app/Contents/Developer
# TODO: ??
sudo xcodebuild -license accept
# TODO: ??
xcodebuild -runFirstLaunch
```

```sh
error: error: cannot execute tool 'metal' due to missing Metal Toolchain; use: xcodebuild -downloadComponent MetalToolchain
```

toolchainが入ってないと  
最近`metal`コンパイラは`metal toolchain`という別のcomponentになってて、本体とは別にインストールしなければいけないらしい

```sh
xcodebuild -downloadComponent metalToolchain
```

800MBぐらい

```sh
xcrun -sdk macosx metal --version
>>>
Apple metal version 32023.918 (metalfe-32023.918.1)
Target: air64-apple-darwin27.0.0
Thread model: posix
InstalledDir: /private/var/run/com.apple.security.cryptexd/mnt/com.apple.MobileAsset.MetalToolchain-v27.1.5218.8.xH7d5X/Metal.xctoolchain/usr/metal/current/bin
```

ok

C++で書きたいので[ここ](https://github.com/apple/metal-cpp)より`metal-cpp`のヘッダーを取得してくる  
(star数少なすぎないか...)

```sh
git clone https://github.com/apple/metal-cpp.git
# osのversionにあわせて適宜tagへcheckout
```










