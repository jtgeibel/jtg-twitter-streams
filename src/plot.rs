use super::KEYWORDS;

use plotters::prelude::*;

const CHART_DIMENSIONS: (u32, u32) = (1600, 900);

pub struct Chart(Vec<Vec<f32>>);

impl Chart {
    /// Initialize an empty Vec<f32> for each keyword
    pub fn new() -> Self {
        let mut chart = Vec::new();
        chart.resize_with(KEYWORDS.len(), Default::default);
        Self(chart)
    }

    /// Given a keyword index, pushes the value onto the end of its Vec
    pub fn push(&mut self, idx: usize, val: f32) {
        self.0[idx].push(val)
    }

    /// Save a line series chart of the data to a PNG located at `/tmp/chart.png`
    ///
    /// # Blocking
    ///
    /// This function makes blocking filesystem calls.
    pub fn plot_and_save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = &self.0;
        let (y_min, y_max) = data.iter().flatten().fold((0.0, 0.0), |mut acc, &y| {
            if y < acc.0 {
                acc.0 = y;
            }
            if y > acc.1 {
                acc.1 = y;
            }
            acc
        });

        let root = BitMapBackend::new("/tmp/chart2.png", CHART_DIMENSIONS).into_drawing_area();
        root.fill(&WHITE)?;

        let mut cc = ChartBuilder::on(&root)
            .margin(10)
            .caption("Keyword Sentiment on Twitter", ("Arial", 30).into_font())
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_ranged(0..data[0].len() as u32, y_min..y_max)?;

        cc.configure_mesh()
            .x_label_formatter(&|x| format!("{}", x))
            .y_label_formatter(&|y| format!("{}", y))
            .x_labels(15)
            .y_labels(5)
            .x_desc("Seconds")
            .y_desc("Cumulative Average Score per Tweet")
            .axis_desc_style(("Arial", 15).into_font())
            .draw()?;

        for (idx, data) in data.iter().enumerate() {
            cc.draw_series(LineSeries::new(
                data.iter().enumerate().map(|(a, b)| (a as u32, *b)),
                &Palette99::pick(idx),
            ))?
            .label(KEYWORDS[idx])
            .legend(move |(x, y)| {
                Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(idx))
            });
        }

        cc.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        // Write file then rename so the web server doesn't see a partially written file
        drop(cc);
        drop(root);
        std::fs::rename("/tmp/chart2.png", "/tmp/chart.png")?;

        Ok(())
    }
}
