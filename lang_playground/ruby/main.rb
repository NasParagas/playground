# 類を見ない書き方
puts "hello from main.rb!"

count = 0
is_running = true

while is_running

  if count % 2 == 0
    count.times do  # shみたい
      puts "#{count}: 偶数"
    end
  else
    puts "#{count}: 奇数"
  end

  case count
  when 3
    puts "end"
    is_running = false
  end

  count += 1
end
