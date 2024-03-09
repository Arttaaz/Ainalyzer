use iced::widget::button;
use iced::Element;
use iced::widget::svg;
use iced_native::svg::Handle;

use charts_rs::{ LineChart, Line, Series, SeriesLabel};

pub struct WinratePlot {
    pub data_points: Vec<f32>,
    pub black_percentage: bool,
    pub chart: LineChart,
}

impl WinratePlot {
    pub fn new() -> Self {
        let mut data_points = vec![0.0];
        // allocate now to prevent frequent allocations
        data_points.reserve_exact(300);
        Self {
            data_points,
            black_percentage: true,
            chart: Self::setup_chart(),
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, crate::Message> {
        let chart: String = self.chart.svg().unwrap();
        let chart = Box::leak(chart.into_boxed_str());
        iced::widget::Column::new()
            .push(svg(Handle::from_memory(chart.as_bytes())))
            .height(iced::Length::FillPortion(3))
            .align_items(iced::Alignment::Center)
            .into()
    }

    pub fn update_plot(&mut self, (idx, winrate): (u64, f32)) {
        let idx = idx as usize;
        // updating already existing points
        if self.data_points.len() > idx {
            self.data_points[idx] = winrate;
            // for now we remove all further points as they are not up to date anymore
            self.data_points.truncate(idx+1);
        // inserting new points
        } else {
            // resize array with placeholder value
            self.data_points.resize(idx+1, -1.0);
            self.data_points[idx] = winrate;
        }

        self.update_chart();
    }
}

impl WinratePlot {
    
    fn setup_chart() -> LineChart {

        let mut chart = LineChart::new(vec![Series::new("Winrate".to_string(), vec![10.0, 30.0, 20.0, 76.4])], (0..100).step_by(10).map(|x| x.to_string()).collect());

        chart.background_color = charts_rs::Color::black();
        chart.series_fill = true;
        chart.series_smooth = true;
        let mut y_axis_configs = charts_rs::YAxisConfig::default();
        y_axis_configs.axis_min = Some(0.0);
        y_axis_configs.axis_max = Some(100.0);
        y_axis_configs.axis_width = Some(2.0);
        chart.y_axis_configs = vec![y_axis_configs];

        chart
    }

    fn update_chart(&mut self) {

        //let mut chart = builder
        //    .x_label_area_size(40)
        //    .y_label_area_size(30)
        //    .margin(20)
        //    .build_cartesian_2d(1..moves_range, (0.0 as f32..101.0 as f32).step(5.0))
        //    .expect("failed to build chart");
        //
        //chart.configure_mesh()
        //    .label_style(&plotters::style::WHITE)
        //    .axis_style(ShapeStyle::from(plotters::style::colors::WHITE.mix(0.45)).stroke_width(1))
        //    .bold_line_style(plotters::style::WHITE.mix(0.1))
        //    .x_desc("move number")
        //    .y_label_formatter(&|y| format!("{}%", y))
        //    .y_labels(10)
        //    .draw()
        //    .expect("failed to draw chart mesh");

        //let series = AreaSeries::new(self.data_points.iter().enumerate().filter_map(|(i, x)| {
        //        if *x == -1.0 {
        //            None
        //        } else if self.black_percentage {
        //            Some((i as u64, *x))
        //        } else {
        //            Some((i as u64, 100.0 - *x))
        //        }
        //}), 0.0, plotters::style::WHITE.mix(0.8));
        //chart.draw_series(series)
        //    .expect("Could not draw line series");
    }
}
