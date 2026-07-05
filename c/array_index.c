#include <stdio.h>

int main() {
    int a[] = {10, 20, 30};
    printf("%d\n", a[1]); // 20
    printf("%d\n", 1[a]); // 20

    // a[1], 1[a]はどちらも仕様上、*(a + 1), *(1 + a)置き換えられる
    // この*(a + 1)は、ポインタをaが指している型の要素分進める操作
    // 例えば以下のような感じでも使える
    
    int* a_ptr = a;
    printf("p     = %p\n", a_ptr);
    printf("p + 1 = %p\n", (a_ptr + 1));  // sizeof int分進むはず
    printf("sizeof int  = %zu bytes\n", sizeof(int));

    char n = 'n';
    char* n_ptr = &n;
    printf("n     = %p\n", n_ptr);
    printf("n + 1 = %p\n", (n_ptr + 1));  // sizeof char分進むはず
    printf("sizeof char  = %zu bytes\n", sizeof(char));

    // つまり、Cの加算演算子はint+int, int+ptr, ptr+intが許されている
    
    // そしてe1[e2] は仕様上 *((e1) + (e2)) と定義されている。
    // そのため a[1] は *(a + 1)、1[a] は *(1 + a) と同じ意味になる。
    // 配列 a はこの式の中では先頭要素へのポインタに変換される。
    // ptr+int, int+ptr 型の要素数だけ進む
    // ...ということが重なってこんな不思議なことになっているっぽい

    // この二つは同じになるはず
    printf("&a[1] = %p\n", &a[1]);
    printf("p + 1 = %p\n", (a_ptr + 1));

    printf("(p + 1) - p = %td elements\n", (a_ptr + 1) - a_ptr);
    printf("byte diff   = %td bytes\n", (char *)(a_ptr + 1) - (char *)a_ptr);

    printf("*(p + 1) = %d\n", *(a_ptr + 1));
    printf("*(1 + p) = %d\n", *(1 + a_ptr));

    return 0;

}

