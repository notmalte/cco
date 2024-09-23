int main(void) {
    int a = 3;
    while (a > 0) {
        a -= 1;

        if (a == 1) {
            break;
        }
    }

    int b = 3;
    do {
        b -= 1;

        continue;

        b -= 1;
    } while (b > 0);

    int c = 3;
    for (int i = 0; i < c; i++) {
        c -= 1;
    }

    return a + b + c;
}
