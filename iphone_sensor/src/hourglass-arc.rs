const ROWS: usize = 10;
const WIDTH: usize = 2 * ROWS - 1;
const GRAVITY_THRESHOLD: f64 = 1.0;
const CAPACITY: f64 = 1000.0;
const NANOS_PER_SEC: f64 = 1_000_000_000.0;

pub struct Hourglass {
    top: f64,
    bottom: f64,
    /// 1秒あたりに流れる砂の量 (CAPACITY / 指定した分数)
    flow_per_sec: f64,
    last_time: Option<u64>,
    /// 現在、砂がネックを伝って流れている最中かどうか
    flowing: bool,
}

impl Hourglass {
    /// `minutes` で指定した時間で全ての砂が落ちきるように流量を決める
    pub fn new(minutes: f64) -> Self {
        Self {
            top: CAPACITY,
            bottom: 0.0,
            flow_per_sec: CAPACITY / (minutes * 60.0),
            last_time: None,
            flowing: false,
        }
    }

    /// 重力ベクトルのy成分から端末の向きを判定し、経過時間に応じて砂を流す
    pub fn update(&mut self, gravity_y: f64, time_ns: u64) {
        let dt = match self.last_time {
            Some(last) if time_ns > last => (time_ns - last) as f64 / NANOS_PER_SEC,
            _ => 0.0,
        };
        self.last_time = Some(time_ns);

        let flow = self.flow_per_sec * dt;
        if gravity_y > GRAVITY_THRESHOLD && self.top > 0.0 {
            let flow = flow.min(self.top);
            self.top -= flow;
            self.bottom += flow;
            self.flowing = true;
        } else if gravity_y < -GRAVITY_THRESHOLD && self.bottom > 0.0 {
            let flow = flow.min(self.bottom);
            self.bottom -= flow;
            self.top += flow;
            self.flowing = true;
        } else {
            self.flowing = false;
        }
    }

    pub fn render(&self) -> String {
        let filled_top = scale(self.top);
        let filled_bottom = scale(self.bottom);
        let center = ROWS - 1;

        let mut lines = Vec::with_capacity(ROWS * 2);

        // 上部の部屋: 砂はネック側(行番号が大きいほう)から積もる
        for r in 0..ROWS {
            let width = WIDTH - 2 * r;
            let filled = r >= ROWS - filled_top;
            lines.push(render_row(width, filled, self.flowing, center));
        }

        // 下部の部屋: 砂は底側(行番号が大きいほう)から積もる
        for r in 0..ROWS {
            let width = 1 + 2 * r;
            let filled = r >= ROWS - filled_bottom;
            lines.push(render_row(width, filled, self.flowing, center));
        }

        lines.join("\n")
    }
}

impl Default for Hourglass {
    fn default() -> Self {
        Self::new(3.0)
    }
}

fn scale(value: f64) -> usize {
    let ratio = value / CAPACITY;
    ((ratio * ROWS as f64).round() as usize).min(ROWS)
}

fn render_row(width: usize, filled: bool, flowing: bool, center: usize) -> String {
    let pad = (WIDTH - width) / 2;
    let sand = if filled { '#' } else { '.' };

    let mut chars: Vec<char> = std::iter::repeat(' ')
        .take(pad)
        .chain(std::iter::repeat(sand).take(width))
        .chain(std::iter::repeat(' ').take(pad))
        .collect();

    // 流れている最中は、空いているネック部分に砂の流れを描く
    if flowing && !filled {
        chars[center] = ':';
    }

    chars.into_iter().collect()
}
