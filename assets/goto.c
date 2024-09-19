int main(void) {
    int a = 1;

    goto label;

    a += 3;

    label: a -= 1;

    return a;
}
