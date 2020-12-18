# 第3章 順序回路

## 組み合わせ回路

- 入力値の組み合わせのみによって出力が決定する回路
    - 純粋関数

## 順序回路

- 時間が経過してもデータを記憶できる回路
    - 時間依存の関数
- 組み合わせ回路の各入力値を同期させることも可能
- 1つ以上のフリップフロップが組み込まれている

### クロック

- 継続的に変化する信号
    - これによって時間の経過を表現する
- クロックの1周期（tickの始まりからtockの終わりまで）を単位時間とする
- クロックのフェーズ（tick/tock）は2値信号によって表現可能

### DFF

- D型フリップフロップ
- in: 1bit, out: 1bit
- tを現在の時間とすると、out(t) = in(t - 1)
    - 一つ前の入力値を出力する

### レジスタ

- データの書き込み・読み出しを行える記憶装置
- if load(t-1) then out(t) = in(t-1) else out(t) = out(t-1)
    - then: 書き込み、else: 読み出し

#### ワード

- 多ビットレジスタ（の値）

#### メモリ

- ワードの配列
    - メモリのサイズ（ワードの個数）と幅（各ワードの幅）は任意

### カウンタ

- 順序回路
- out(t) = out(t-1) + c
    - c は定数（通常1）
- 出力は、次に実行されるプログラム演算のアドレスとして解釈される