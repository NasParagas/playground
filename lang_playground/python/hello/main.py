print("hello from main.py!")

count = 0
is_running = True

while is_running:
    if count % 2 == 0:
        for _ in range(count):
            print(f"{count}: 偶数")
    else:
        print(f"{count}: 奇数")

    match count:
        case 3:
            print("end")
            is_running = False

    count += 1


