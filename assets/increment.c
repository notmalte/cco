int main(void) {
    int x = 1;
    ++x;
    --x;
    x++;
    x--;

    int a = x++;
    int b = ++x;

    x = a + b;

    return x;
}
