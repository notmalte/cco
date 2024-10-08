int foo(void) {
    static int bar = 0;

    bar++;

    return bar;
}

int main(void) {
    foo();
    foo();
    foo();

    int baz = foo();

    return baz;
}
