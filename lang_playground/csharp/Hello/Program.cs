class Program
{
    static void Main()
    {
        Console.WriteLine("Hello from Program.sc!");
        int count = 0;
        // キャメルケース慣れない
        bool isRunning = true;

        while (isRunning)
        {
            if (count % 2 == 0)
            {
                for (int i = 0; i < count; i++)
                {
                    // この書き方の変数の入れ方が一番直感的に感じるのはPythonが初めてだったからか...
                    Console.WriteLine($"{count}: 偶数");
                }
            }
            else
            {
                Console.WriteLine($"{count}: 奇数");
            }

            switch (count)
            {
                case 3:
                    Console.WriteLine("end");
                    isRunning = false;
                    break;
            }
            count++;
        }
    }
}
