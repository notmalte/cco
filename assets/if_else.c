int main(void) {
    int a = 1;
    a += 3;

    int b = 0;

    if (a == 4)
        b = 1;
    else
        b = 2;

    int c = b < 2 ? 3 : 4;

    return b;
}
