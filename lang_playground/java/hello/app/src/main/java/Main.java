public class Main {
    public static void main(String[] args) {
        System.out.println("hello from Main.java");

        int count = 0;
        boolean isRunning = true;

        while (isRunning) {
            if (count % 2 == 0) {
                for (int i = 0; i < count; i++) {
                    System.out.println(count + ": 偶数");
                }
            } else {
                System.out.println(count + ": 奇数");
            }

            switch (count) {
                case 3:
                    System.out.println("end");
                    isRunning = false;
                    break;
            }
            count++;
        }
    }
}
