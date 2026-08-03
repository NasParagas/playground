
## はじめに

- x86_64(amd64), arm64(aarch64), risc-v って何が違うの？を実感しようの会です
- 具体的には、仕様を追いながらアセンブリと機械語を読みます
- まず ISA とか ABI って何？ 機械語とアセンブリってどう読むの？と(私も)なっているので、

### やること

- ISA などの用語説明
- アセンブリ、機械語の読み方の簡単な説明
- 各アーキテクチャ(arm64とかx86_64とか)における上記の違いを見る

### やらないこと

- 実機での動作確認
  - 追々やりたい

## 全体の流れ

最初に



## 用語

定義を示します

- ISA
  - 後ほど詳解します
  - Instruction Set Architecture: 命令セットアーキテクチャ
  - ハードウェアとしての CPU が、ソフトウェアに対して公開するインターフェース(どのレジスタを使って良いのか、どんな命令が使えるのか、命令はどう書くのか、など)
- 機械語 
  - CPU が直接読める bit 列
  - `001111101100...`
- アセンブリ
  - 機械語と 1:1 で対応する(ギリ)人間が読めるやつ
  - `89 f8` に対する `movl %edi, %eax` など(後述)
- コンパイル
  - C -> アセンブリ -> 機械語 と変換すること
- 逆アセンブル
  - 機械語 -> アセンブリ と変換すること
- ABI
  - 後ほど詳解
  - Application Binary Interface
  - TODO: 一言で

## ISA(とABI) について(超概要)

CPU はただの電子回路なので、電気的に表現できる bit 列しか処理できません...という前提で、「CPUをこう動かしたい！というときに、どんな bit 列を送ればよいか」を定義しているのが ISA です。  
ピンとこなかったら LLM に聞くか、以下のサイト斜め読みすれば感覚はわかるはず

- https://appswingby.com/命令セットアーキテクチャ（isa）-今更聞けないit用/
- https://ja.wikipedia.org/wiki/命令セット
- https://e-words.jp/w/命令セット.html

種類としては、以下の3つがよく聞くものたちでしょう  

- x86_64
  - `Intel Core i~`, `AMD Ryzen`...
  - (細かくは違うらしいとも聞くのですが) amd64 とか x64 も同義です
- arm64
  - `Apple M Series`, `Snapdragon`...
  - aarch64 も同義
- RISC-V
  - `ESP32-C3`, `SiFive`...(どっちも知らない)

なので、例えば CPU が別(`Core i9` と `Ryzen` とか)だとしても、ISA 仕様が同じものであれば同じバイナリが動きます。逆に、ISAが違うと(原則)動かない  
ただし、 ISA のバイナリでも ABI が違うと実行できません。例えば、Windows と Linux では ABI が異なるので、WSL で build したバイナリは windows 出は実行できない  
(ABI の違いについては少し後述するかもですが細かい話は LLM に聞いていただいて...)

<details>
  <summary> ちょっと余談 </summary>
 
- docker はホストのCPUを使用するので、ISA が違うバイナリを実行することはできません(x86 のマシンで base image が arm64 用にビルドされたものは使用できない)  
- 一方、qemu や Rosetta2 (TODO:Macのやつ？MシリーズでIntelの頃の動かすやつだよね) などのハードウェアまでエミュレーションするものやバイナリ変換を行うものは、ISA の違うマシン用のバイナリを動かすこともできます
  - 当然遅いです
  - docker でも `--platform`を指定すれば、裏で qemu 動かして動かせるらしい。

</details>

## ISA ごとの違い(超概要)

細かい歴史的な背景は調べていただくとして、設計の思想的なところを

- x86_64
  - 40年分ぐらいずっと後方互換性を保っているため、命令数が数千個規模まで膨らんでいる
  - 命令は可変長(後述)
- arm64
  - 省電力性が求められるモバイル端末向けの ISA であったのでシンプルな設計
  - 命令は固定長(後述)
- RISC-V
  - もともとは教育研究用にオープンなものとして作られたもの
  - なので、命令数も少なく(50個ぐらい)シンプルな設計
  - 命令は固定長(後述)。(まだ詳しく知らないが、拡張すると固定長とは限らないらしい)

また、`CISC`, `RISC`という概念があります

- CISC
  - Complex Instruction Set Computer
  - 複雑な命令セットのコンピューター、ということで、1命令で CPU に複雑な仕事を任せられます
  - 命令数が少なくなるので、そのプログラムのサイズが小さくなりやすい。かつてはメモリが高価だったからね...
  - `x86_64`はこの思想
- RISC
  - Reduced ~~~
  - 縮小 ~~~ ということで、CISC の逆で、1命令はシンプルです
  - (あんま腹落ちしてないのでclaude原文のまま載せますが)`単純な命令のほうがパイプライン化しやすく、結果的に速くできます`とのこと
  - `arm64`, `RISC-V`はこっち

例えば、(細かい機械語の話は後述しますが)「メモリ上の値に1を足す」という命令を書くとき、`x86`は`add [rax], 1`となりますが、RISC側では`load`,`add`,`store`と手順を踏みます

## Cコードを逆アセンブルして機械語読んでみる

### 環境構築

手元でも target を指定してコンパイルできますが、[Compiler Explorer](https://godbolt.org)を使えば手軽にできます


<details>
  <summary> 手元の環境構築メモ </summary>

```sh
# macOS
brew install llvm
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"

# ubuntu
sudo apt install llvm
``` 

</details>

### 試すぞ！

まずは単純な加減乗除で比較しながら、機械語の読み方を説明します

`sub.c`
```c
int sub(int a, int b) {
    return a - b;
}
```

コンパイル

```sh
# コンパイル(フラグの説明は後述)
clang -c -O1 --target=x86_64-linux-gnu -o x86_64_sub.o sub.c
clang -c -O1 --target=aarch64-linux-gnu -o arm64_sub.o sub.c
clang -c -O1 --target=riscv64-linux-gnu -march=rv64g -mno-relax -o riscv_sub.o sub.c

# 逆アセンブル
llvm-objdump -d x86_64_sub.o
llvm-objdump -d arm64_sub.o
llvm-objdump -d riscv_sub.o
```

こんな感じで出力されるはず

```sh
x86_64_sub.o:   file format elf64-x86-64

Disassembly of section .text:

0000000000000000 <sub>:
       0: 89 f8                         movl    %edi, %eax
       2: 29 f0                         subl    %esi, %eax
       4: c3                            retq

arm64_sub.o:    file format elf64-littleaarch64

Disassembly of section .text:

0000000000000000 <sub>:
       0: 4b010000      sub     w0, w0, w1
       4: d65f03c0      ret

riscv_sub.o:    file format elf64-littleriscv

Disassembly of section .text:

0000000000000000 <sub>:
       0: 40b5053b      subw    a0, a0, a1
       4: 00008067      ret
```


`x86_64` の出力を例にして、要所で他の2つと比べながら読み方の説明をします

```
x86_64_sub.o:   file format elf64-x86-64

Disassembly of section .text:

0000000000000000 <sub>:
        0: 89 f8                         movl    %edi, %eax
        2: 29 f0                         subl    %esi, %eax
        4: c3                            retq
```

#### 出力の構造

- `x86_64_sub.o: file format elf64-x86-64`
  - このオブジェクトファイルの形式
  - そのまま ELF の 64bit、x86-64 用ということ
    - TODO: ELF？
- `Disassembly of section .text:`
  - (まだピンと来ていないので claude より転記)
    - `.text`セクション(= 実行される機械語が入っている場所)を逆アセンブルしている、という宣言
    - 他に`.data`(初期値付きのグローバル変数)などがあるが、今回は関数しかないので`.text`だけ
- `0000000000000000 <sub>:`
  - シンボル`sub`(= C の`sub`関数)がここから始まる、という意味
  - `.o`はまだリンクされていない(= `main()`で処理が走ってない)ので、アドレスは 0 から始まる仮のもの、になっているそう

```
0: 89 f8       movl    %edi, %eax
^   ^          ^
|   |          └─ アセンブリ(人間が読む表現)
|   └─ 機械語(CPU が実際に食うバイト列、16進表記)
└─ この命令が関数の先頭から何バイト目にあるか
```

- `0:`のところ
  - この命令がこの関数の先頭から何バイト目にあるか
  - ここでは一つの命令が`89 f8`と2バイトなので、`0:`,`2:`となっている
- `89 f8`
  - 機械語
- `movl`以降
  - アセンブリ
  - 読み方は後述


<details>
  <summary> 可変長命令・固定長命令の話</summary>

`x86_64`の方は`89 f8`と`c3`のように、命令が2バイトだったり1バイトだったりしても良い  
逆に残り2つは4バイト固定とされている  
(仕様の話なのでそれだけ)

TODO: メリットデメリット入れても良いかな

</details>

#### アセンブリの読み方

参考
- https://qiita.com/kaito_tateyama/items/89272098f4b286b64115
- opus-5
- https://qiita.com/mamaru/items/082bb1ebdf845e523eed

```
movl %edi, %eax
subl %esi, %eax
retq
```

- ニーモニック(`movl`)
  - 何をするか、を表す
  - 末尾の`l`,`q`,`w`などはデータ幅を表す(`l`=long(32bit), `w`=16bit, `q`=64bit)
    - `w`はrisc-vのアセンブリで出てきてます(`w`がつく理由は後述)
  - `mov` = コピー
  - `sub` = 減算
  - `ret` = 関数から戻る
- オペランド(`%edi, %eax`)
  - 何に対してするか、を表す
  - x86_64は`命令 src dest`、arm64/riscvは`命令 dest src1 src2`
  - `%edi, %eax`はレジスタ名(後述)

<details>
  <summary> x86_64 におけるアセンブリの記法の話 </summary>

Intel 記法と AT&T 記法があります。`llvm-objdump` のデフォルトが AT&T 記法なので、ここのもそれです  
AT&T は`命令 src dest`ですが、Intel は`命令 dest src`だったり、命令の書き方が違ったりします  

</details>

####  `%edi`, `w0`, `a0` とかについて

- レジスタです
  - レジスタ = CPU の中にある数個〜数十個ある超高速なメモリのようなもの
- 名前が違うのは単純にアーキテクチャ毎に命名の仕方が異なっているだけで、役割は対応しています
- 前述の通り、「どのレジスタで引数を渡し、どこに戻り値を置くか」などを決めているのが ABI
- 先程から載せている下記のアセンブリだと、下のように対応します

```sh
x86_64_sub.o:   file format elf64-x86-64

Disassembly of section .text:

0000000000000000 <sub>:
       0: 89 f8                         movl    %edi, %eax
       2: 29 f0                         subl    %esi, %eax
       4: c3                            retq

arm64_sub.o:    file format elf64-littleaarch64

Disassembly of section .text:

0000000000000000 <sub>:
       0: 4b010000      sub     w0, w0, w1
       4: d65f03c0      ret

riscv_sub.o:    file format elf64-littleriscv

Disassembly of section .text:

0000000000000000 <sub>:
       0: 40b5053b      subw    a0, a0, a1
       4: 00008067      ret

```

```
x86_64:  movl %edi, %eax   # eax(戻り値) ← edi(第1引数)
         subl %esi, %eax   # eax ← eax - esi(第2引数)
         retq              # 呼び出し元に戻る

arm64:   sub  w0, w0, w1   # w0(戻り値) ← w0(第1引数) - w1(第2引数)
         ret

riscv:   subw a0, a0, a1   # a0(戻り値) ← a0(第1引数) - a1(第2引数)
         ret
```


#### RISC-V のアセンブリを機械語に分解してみる

参考
- https://docs.riscv.org/reference/home/index.html
- https://riscv.org/wp-content/uploads/2024/12/riscv-calling.pdf
- https://zenn.dev/tetsu_koba/articles/b7b31b372f0f40

ABI の中には calling convention (呼び出し規約) と呼ばれる概念があります(ABI が calling convention を包含)  
私もかなりふわっとした理解 & これ自体が何かわかっていなくても以降の話はギリわかるので、気になる方はLLMってください。[これ](https://freak-da.hatenablog.com/entry/2021/03/25/172248)とかはわかりやすかったです  
RISC
[ここ](https://riscv.org/wp-content/uploads/2024/12/riscv-calling.pdf)が RISC-V の calling convention のドキュメント(のはず)なので、これを基に少しレジスタの使われ方を見ていこうと思います  

![RISC-V calling convention](../assets/2026-08-03-14-09-41.png)

先程から載せている RISC-V のアセンブリは以下です  

```sh
subw    a0, a0, a1
ret
```

ここに出てきている `a0`,`a1` は、上記表の `ABI Name` には `Function arguments/return values` という desc があります  
その下の `a2-7` も `function arguments` とのことなので、命令の引数が増えていったらここら辺も使われそう。ただし `ret` には `a0,a1` しかこなそうですね  
例えば  

```c
int sub_ippai(int a, int b, int c, int d, int e) {
    return a - b - c - d - e;
}
```

のアセンブリは、同様のコマンドを使うと

```sh
0000000000000008 <sub_ippai>:
       8: 00c585b3      add     a1, a1, a2
       c: 00e686b3      add     a3, a3, a4
      10: 00d585b3      add     a1, a1, a3
      14: 40b5053b      subw    a0, a0, a1
      18: 00008067      ret
```

となりました。`8`から始まっているのはこれの直前にさっきまで見せてた`sub()`がいるからです。  
せっかくなので、[RISC-V の ISA の仕様書](https://docs.riscv.org/reference/home/index.html)から、各命令が何を表すのか見ます  

![](../assets/2026-08-03-17-10-29.png)
この画像自体は、各命令が 32bit の機械語で表現される時の bit の配置を表します。  
命令によっていくつか配置にも種類があり、これは `R-Type`  
(64bit 環境での話は頭がキャパオーバーなので一旦おいておく)

##### `add`

- (だいぶメモ書きみたいな書き方になってしまってます...真上の画像も確認してね)
- [Integer Register-Register Instructions](https://docs.riscv.org/reference/isa/v20260120/unpriv/rv32.html#1-1-4-2-integer-register-register-instructions) によると、`ADD` は`RV32I`に定義される命令で、ここに定義されている命令は `rs1`,`rs2` レジスタを source として(ここでは src/dest の src ですね) 、`rd` を結果を書き込む(dest)レジスタとして使用するそうです
- `funct7`,`funct3` はどの命令かを識別するためのfield
- `objdump` より、`add a1, a1, a2` は `00c585b3` と表されています。上記画像より、32bit の機械語は `funct7(31-25) rs2(24-20) rs1(19-15) funct3(14-12) rd(11-7) opcode(6-0)` と各 bit 列を並べたものですので
  - `funct7: 0000000`
  - `rs2: 01100`
  - `rs1: 01011`
  - `funct3: 000`
  - `rd: 01011`
  - `opcode: 0110011`
- のように当てはまるはずです(適宜16進数->2進数としています)
- この画像から、`ADD`は`funct3`を見ると1列目にあり、そこの`funct7`は`0000000`なのであってそうですが、他の bit があってるかこの画像からは判断できない
- `funct3`,`opcode`については[RV32/64G Instruction Set Listings](https://docs.riscv.org/reference/isa/v20260120/unpriv/rv-32-64g.html)から確認可能
  - 表がありえないぐらい横長ですが、上の画像のような機械語の構造を表しています(よくみると表の上側に31-27,6-0とかある)
  - `ADD` と記載のある場所を探して確認すると、`funct3`にあたるところは`000`, `opcode` にあたるところは `0110011` となっていました。あってそう
- 残りの`rs2`,`rs1`,`rd`について、これらはレジスタなので、機械語はレジスタの番号を表しているそうです
- そして[Register Convention](https://docs.riscv.org/reference/abi/v1.0/riscv-cc-register-convention.html)や ABI の時に説明に挙げた画像より、ハードウェアにおけるレジスタ`x10-x17`について、RISC-VのABI仕様では `a0-a7` となります
- `rs2: 01100 = 12(10進数)`,`rs1,rd: 01011 = 11`なので、`rs2: a2`, `rs2,rd: a1`のはず！
- というわけで全部あってそうです...長かった...こんな感じで対応しているんですね...

##### `subw`

- `subw`の話は[Integer Register-Register Operations](https://docs.riscv.org/reference/isa/v20260120/unpriv/rv64.html#3-1-2-2-integer-register-register-operations)に記載があります
- 上記によると、`RV64I-only instructions that are defined analogously to ADD and SUB but operate on 32-bit values and produce signed 32-bit results` とのこと
  - 64bit にしかない命令だそうです
  - コンパイル時の target を `--target=riscv64-linux-gnu` と指定していた通り、64bit 環境向けにコンパイルしていたため、、32bit 環境でコンパイルするとこの命令はなくなりそうですね？
 
```sh
$ clang -c -O1 --target=riscv32-linux-gnu -march=rv32g -mno-relax -o riscv_sub.o sub.c
$ llvm-objdump -d riscv_sub.o

00000008 <sub_ippai>:
       8: 00c585b3      add     a1, a1, a2
       c: 00e686b3      add     a3, a3, a4
      10: 00d585b3      add     a1, a1, a3
      14: 40b50533      sub     a0, a0, a1
      18: 00008067      ret
```

- 普通のsubになった！
- C の `int` は 32bit なので、本当は 64bit 環境でも 32bit 環境の命令を使いたいのだが、普通の `sub` は 64bit 整数の減算となってしまう(？)ので、`subw` とするそう

##### `ret`

- `return` ですが、ちょっと深淵そうなので今回は見送ります...

## 余談

<details>
  <summary> `-O1` の最適化とかの話 </summary>

例えば、`-O1`では、以下の二つは等価な機械語にコンパイルされます

```c
int add(int a, int b) {
    return a + b;
}

int add2(int a, int b) {
    int c = a + b;
    return c;
}
```

```
"add(int, int)":
        lea     eax, [rdi+rsi]
        ret
"add2(int, int)":
        lea     eax, [rdi+rsi]
        ret
```

結果は同じになるので、コンパイラが最適化してくれているのですね。  
これが`-O0`で最適化無しにすると、少し変わります

```
"add(int, int)":
        push    rbp
        mov     rbp, rsp
        mov     DWORD PTR [rbp-4], edi
        mov     DWORD PTR [rbp-8], esi
        mov     edx, DWORD PTR [rbp-4]
        mov     eax, DWORD PTR [rbp-8]
        add     eax, edx
        pop     rbp
        ret
"add2(int, int)":
        push    rbp
        mov     rbp, rsp
        mov     DWORD PTR [rbp-20], edi
        mov     DWORD PTR [rbp-24], esi
        mov     edx, DWORD PTR [rbp-20]
        mov     eax, DWORD PTR [rbp-24]
        add     eax, edx
        mov     DWORD PTR [rbp-4], eax
        mov     eax, DWORD PTR [rbp-4]
        pop     rbp
        ret
```

`add2`の方では、(TODO: rdp-n の n が違うのは何？)以外の差分として

```
        mov     DWORD PTR [rbp-4], eax
        mov     eax, DWORD PTR [rbp-4]
```

があります。

</details>


