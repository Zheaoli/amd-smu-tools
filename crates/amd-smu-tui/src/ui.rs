use crate::app::App;
use amd_smu_lib::PmTable;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(1),  // Footer
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);
    draw_footer(frame, chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let codename = app.pm_table.as_ref()
        .map(|t| t.codename_str.as_str())
        .unwrap_or("Unknown");

    let version = app.pm_table.as_ref()
        .map(|t| format!("{:#x}", t.version))
        .unwrap_or_else(|| "?".to_string());

    let title = format!(
        " AMD Ryzen ({}) | {} | PM Table v{} | Refresh: {}ms ",
        codename,
        app.smu_version,
        version,
        app.interval.as_millis()
    );

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn draw_main(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref error) = app.error {
        let error_msg = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        frame.render_widget(error_msg, area);
        return;
    }

    let Some(ref table) = app.pm_table else {
        let loading = Paragraph::new("Loading...")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, area);
        return;
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),   // Limits (PPT/TDC/EDC)
            Constraint::Length(6),   // Temperatures
            Constraint::Min(4),      // Cores
        ])
        .split(area);

    if app.show_power {
        draw_limits(frame, table, main_chunks[0]);
    }
    if app.show_temps {
        draw_temps(frame, table, main_chunks[1]);
    }
    if app.show_freq {
        draw_cores(frame, table, main_chunks[2]);
    }
}

fn draw_limits(frame: &mut Frame, table: &PmTable, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    // PPT gauge
    let ppt_pct = (table.ppt_value / table.ppt_limit * 100.0).min(100.0) as u16;
    let ppt_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("PPT (Power)"))
        .gauge_style(Style::default().fg(temp_color(ppt_pct as f32, 70.0, 90.0)))
        .percent(ppt_pct)
        .label(format!("{:.1}W / {:.1}W", table.ppt_value, table.ppt_limit));
    frame.render_widget(ppt_gauge, chunks[0]);

    // TDC gauge
    let tdc_pct = (table.tdc_value / table.tdc_limit * 100.0).min(100.0) as u16;
    let tdc_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("TDC (Current)"))
        .gauge_style(Style::default().fg(temp_color(tdc_pct as f32, 70.0, 90.0)))
        .percent(tdc_pct)
        .label(format!("{:.1}A / {:.1}A", table.tdc_value, table.tdc_limit));
    frame.render_widget(tdc_gauge, chunks[1]);

    // EDC gauge
    let edc_pct = (table.edc_value / table.edc_limit * 100.0).min(100.0) as u16;
    let edc_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("EDC (Peak)"))
        .gauge_style(Style::default().fg(temp_color(edc_pct as f32, 70.0, 90.0)))
        .percent(edc_pct)
        .label(format!("{:.1}A / {:.1}A", table.edc_value, table.edc_limit));
    frame.render_widget(edc_gauge, chunks[2]);
}

fn draw_temps(frame: &mut Frame, table: &PmTable, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Tctl gauge
    let tctl_pct = (table.tctl / table.thm_limit * 100.0).min(100.0) as u16;
    let tctl_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Tctl (Junction)"))
        .gauge_style(Style::default().fg(temp_color(table.tctl, 70.0, 85.0)))
        .percent(tctl_pct)
        .label(format!("{:.1}°C / {:.1}°C", table.tctl, table.thm_limit));
    frame.render_widget(tctl_gauge, chunks[0]);

    // SoC temp
    let soc_pct = (table.soc_temp / 80.0 * 100.0).min(100.0) as u16;
    let soc_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("SoC Temperature"))
        .gauge_style(Style::default().fg(temp_color(table.soc_temp, 50.0, 70.0)))
        .percent(soc_pct)
        .label(format!("{:.1}°C", table.soc_temp));
    frame.render_widget(soc_gauge, chunks[1]);
}

fn draw_cores(frame: &mut Frame, table: &PmTable, area: Rect) {
    let mut lines = Vec::new();

    // Core temps line
    let mut temp_spans = vec![Span::raw("Temps:  ")];
    for (i, temp) in table.core_temps.iter().enumerate() {
        if *temp > 0.0 {
            let color = temp_color(*temp, 70.0, 85.0);
            temp_spans.push(Span::styled(
                format!("C{}: {:5.1}°C  ", i, temp),
                Style::default().fg(color),
            ));
        }
    }
    lines.push(Line::from(temp_spans));

    // Core freqs line
    let mut freq_spans = vec![Span::raw("Freqs:  ")];
    for (i, freq) in table.core_freqs.iter().enumerate() {
        if *freq > 0.0 {
            freq_spans.push(Span::styled(
                format!("C{}: {:4.0}MHz  ", i, freq),
                Style::default().fg(Color::White),
            ));
        }
    }
    lines.push(Line::from(freq_spans));

    // Core power line
    let mut power_spans = vec![Span::raw("Power:  ")];
    for (i, power) in table.core_power.iter().enumerate() {
        if *power > 0.0 {
            power_spans.push(Span::styled(
                format!("C{}: {:5.2}W  ", i, power),
                Style::default().fg(Color::Yellow),
            ));
        }
    }
    lines.push(Line::from(power_spans));

    // C0 residency line
    let mut c0_spans = vec![Span::raw("C0:     ")];
    for (i, c0) in table.core_c0.iter().enumerate() {
        c0_spans.push(Span::styled(
            format!("C{}: {:5.1}%  ", i, c0),
            Style::default().fg(Color::Cyan),
        ));
    }
    lines.push(Line::from(c0_spans));

    let cores = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Per-Core Metrics"));
    frame.render_widget(cores, area);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(" [q] Quit  [t] Temps  [p] Power  [f] Freq  [+/-] Interval ")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}

fn temp_color(value: f32, warn: f32, crit: f32) -> Color {
    if value >= crit {
        Color::Red
    } else if value >= warn {
        Color::Yellow
    } else {
        Color::Green
    }
}
