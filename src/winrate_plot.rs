use iced::Element;
//use iced::widget::svg;
//use iced_native::svg::Handle;

use plotters::series::AreaSeries;
use plotters::style::Color;
use plotters::style::ShapeStyle;
use plotters_iced::{Chart, ChartWidget, DrawingBackend, ChartBuilder};

use crate::Message;

pub struct WinratePlot {
    pub data_points: Vec<f32>,
    pub black_percentage: bool,
    //pub chart: Chart<Message>,
}

impl WinratePlot {
    pub fn new() -> Self {
        let mut data_points = vec![0.0];
        // allocate now to prevent frequent allocations
        data_points.reserve_exact(300);
        Self {
            data_points,
            black_percentage: true,
            //chart: ,
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, crate::Message> {
        ChartWidget::new(self)
            .into()
        //let chart: String = self.chart.svg().unwrap();
        //let chart = Box::leak(chart.into_boxed_str());
        //iced::widget::Column::new()
        //    .push(svg(Handle::from_memory(chart.as_bytes())))
        //    .height(iced::Length::FillPortion(3))
        //    .align_items(iced::Alignment::Center)
        //    .into()
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

        //self.update_chart();
    }
}

impl Chart<crate::Message> for WinratePlot {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let moves_range: u64 = 100;
        let mut chart = builder
            .x_label_area_size(40)
            .y_label_area_size(30)
            .margin(20)
            .build_cartesian_2d(1..moves_range, 0.0..101.0 as f32)
            .expect("failed to build chart");
        
        chart.configure_mesh()
            .label_style(&plotters::style::WHITE)
            .axis_style(ShapeStyle::from(plotters::style::colors::WHITE.mix(0.45)).stroke_width(1))
            .bold_line_style(plotters::style::WHITE.mix(0.1))
            .x_desc("move number")
            .y_label_formatter(&|y| format!("{}%", y))
            .y_labels(10)
            .draw()
            .expect("failed to draw chart mesh");

        let series = AreaSeries::new(self.data_points.iter().enumerate().filter_map(|(i, x)| {
                if *x == -1.0 {
                    None
                } else if self.black_percentage {
                    Some((i as u64, *x))
                } else {
                    Some((i as u64, 100.0 - *x))
                }
        }), 0.0, plotters::style::WHITE.mix(0.8));
        chart.draw_series(series)
            .expect("Could not draw line series");
        
    }
}

//impl WinratePlot {
//    
//    fn setup_chart() -> LineChart {
//        //let mut chart = LineChart::new_with_theme(vec![Series::new("Winrate".to_string(), vec![10.0, 30.0, 20.0, 76.4])], vec![], "grafana");//(0..100).step_by(10).map(|x| x.to_string()).collect(), "grafana");
//
//        //chart.background_color = charts_rs::Color::black();
//        //chart.series_fill = true;
//        //chart.series_smooth = true;
//        //chart.series_list[0].category = Some(charts_rs::SeriesCategory::Line);
//        //chart.series_list[0].y_axis_index = 0;
//        //chart.series_list[0].label_show = true;
//        //chart.x_axis_font_color = charts_rs::Color::white();
//        //let mut y_axis_configs = charts_rs::YAxisConfig::default();
//        //y_axis_configs.axis_min = Some(-100.0);
//        //y_axis_configs.axis_max = Some(100.0);
//        //y_axis_configs.axis_width = Some(1.0);
//        //y_axis_configs.axis_stroke_color = charts_rs::Color::white();
//        //y_axis_configs.axis_font_color = charts_rs::Color::white();
//        //chart.y_axis_configs = vec![y_axis_configs];
//        //chart.width = 1980.0/2.0;
//        //chart.height = 1080.0/2.0;
//        //chart.legend_show = Some(true);
//
//        //chart
//    }
//
//    fn update_chart(&mut self) {
//
//        //let mut chart = builder
//        //    .x_label_area_size(40)
//        //    .y_label_area_size(30)
//        //    .margin(20)
//        //    .build_cartesian_2d(1..moves_range, (0.0 as f32..101.0 as f32).step(5.0))
//        //    .expect("failed to build chart");
//        //
//        //chart.configure_mesh()
//        //    .label_style(&plotters::style::WHITE)
//        //    .axis_style(ShapeStyle::from(plotters::style::colors::WHITE.mix(0.45)).stroke_width(1))
//        //    .bold_line_style(plotters::style::WHITE.mix(0.1))
//        //    .x_desc("move number")
//        //    .y_label_formatter(&|y| format!("{}%", y))
//        //    .y_labels(10)
//        //    .draw()
//        //    .expect("failed to draw chart mesh");
//
//        //let series = AreaSeries::new(self.data_points.iter().enumerate().filter_map(|(i, x)| {
//        //        if *x == -1.0 {
//        //            None
//        //        } else if self.black_percentage {
//        //            Some((i as u64, *x))
//        //        } else {
//        //            Some((i as u64, 100.0 - *x))
//        //        }
//        //}), 0.0, plotters::style::WHITE.mix(0.8));
//        //chart.draw_series(series)
//        //    .expect("Could not draw line series");
//    }
//}
