int main(void) {
    int a = 3;

    switch (a) {
    case 1:
        return 1;
    case 2:
        return 2;
    case 3:
        a++;
        break;
    default:
        return 0;
    }

    return a;
}
