package main

import "fmt"

func main() {
	fmt.Println("hello from main.go")

	// 変数の宣言と初期化を同時に行う時は広角らしい
	count := 0
	isRunning := true

	for isRunning {
		if count%2 == 0 {
			for i := 0; i < count; i++ {
				fmt.Printf("%d: 偶数\n", count)
			}
		} else {
			fmt.Printf("%d: 奇数\n", count)
		}
		switch count {
		// フォールスルーしないからbreakはいらないとのこと
		case 3:
			fmt.Println("end")
			isRunning = false
		}

		count++
	}
}
