#include <stdio.h>

int main() {
    printf("hello from hello.c!\n");

    int count = 0;
    int is_running = 1;  // 気持ち悪い

    while (is_running) {

        // 偶数なら
        if (count % 2 == 0) {

            // 値分表示
            for (int i = 0; i < count; i++ ) {
                printf("%d: 偶数\n", count);
            }
        }
        else {
            printf("%d: 奇数\n", count);
        }

        // 3で抜ける。嫌いなので
        switch (count) {
            case 3:
                printf("end\n");
                is_running = 0;
                break;
        }
        count++;
    }

    return 0;
}
