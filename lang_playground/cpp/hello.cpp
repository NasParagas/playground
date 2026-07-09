#include <iostream>

int main() {
    std::cout << "hello from hello.cpp!" << std::endl;

    int count = 0;
    bool is_running = true;  // C++はboolある嬉しい

    while (is_running) {

        // 偶数
        if (count % 2 == 0) {
            // 値分表示
            for (int i = 0; i < count; i++) {
                std::cout << count << ": 偶数\n";
            }
        }
        // 奇数
        else {
            std::cout << count << ": 奇数\n";
        }

        // 3で抜ける
        switch (count) {
            case 3:
                std::cout << ("end\n");
                is_running = false;
                break;
        }
        count++;
    }
    return 0;
}
