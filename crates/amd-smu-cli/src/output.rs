use amd_smu_lib::PmTable;

pub struct OutputOptions {
    pub temps_only: bool,
    pub power_only: bool,
    pub freq_only: bool,
}

impl OutputOptions {
    pub fn show_all(&self) -> bool {
        !self.temps_only && !self.power_only && !self.freq_only
    }
}

pub fn format_text(table: &PmTable, smu_version: &str, opts: &OutputOptions) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("AMD Ryzen ({})\n", table.codename_str));
    out.push_str(&format!("{} | PM Table v{:#x}\n", smu_version, table.version));
    out.push('\n');

    // Temperatures
    if opts.show_all() || opts.temps_only {
        out.push_str("Temperatures:\n");
        out.push_str(&format!("  Tctl:           {:+.1}°C  (limit: {:.1}°C)\n",
            table.tctl, table.thm_limit));
        out.push_str(&format!("  SoC:            {:+.1}°C\n", table.soc_temp));

        // Group cores by CCD (8 cores per CCD)
        let cores_per_ccd = 8;
        let total_cores = table.core_temps.len();
        let num_ccds = (total_cores + cores_per_ccd - 1) / cores_per_ccd;

        for ccd in 0..num_ccds {
            let start = ccd * cores_per_ccd;
            let end = (start + cores_per_ccd).min(total_cores);
            let ccd_temps: Vec<_> = table.core_temps[start..end].iter()
                .filter(|t| **t > 0.0)
                .collect();

            if !ccd_temps.is_empty() {
                out.push_str(&format!("  CCD{}:\n", ccd));
                for (i, temp) in table.core_temps[start..end].iter().enumerate() {
                    if *temp > 0.0 {
                        out.push_str(&format!("    Core {:2}:      {:+.1}°C\n", start + i, temp));
                    }
                }
            }
        }
        out.push('\n');
    }

    // Power
    if opts.show_all() || opts.power_only {
        out.push_str("Power:\n");
        out.push_str(&format!("  Package:        {:.1}W / {:.1}W (PPT)\n",
            table.ppt_value, table.ppt_limit));
        out.push_str(&format!("  TDC:            {:.1}A / {:.1}A\n",
            table.tdc_value, table.tdc_limit));
        out.push_str(&format!("  EDC:            {:.1}A / {:.1}A\n",
            table.edc_value, table.edc_limit));
        out.push_str(&format!("  SoC:            {:.1}W\n", table.soc_power));

        for (i, power) in table.core_power.iter().enumerate() {
            if *power > 0.0 {
                out.push_str(&format!("  Core {:2}:        {:.2}W\n", i, power));
            }
        }
        out.push('\n');
    }

    // Frequencies
    if opts.show_all() || opts.freq_only {
        out.push_str("Frequencies:\n");
        out.push_str(&format!("  FCLK:           {:.0} MHz\n", table.fclk));
        out.push_str(&format!("  MCLK:           {:.0} MHz\n", table.mclk));

        for (i, (freq, eff)) in table.core_freqs.iter()
            .zip(table.core_freqs_eff.iter())
            .enumerate()
        {
            if *freq > 0.0 {
                let c0 = table.core_c0.get(i).unwrap_or(&0.0);
                out.push_str(&format!("  Core {:2}:        {:.0} MHz (eff: {:.0})  C0: {:.1}%\n",
                    i, freq, eff, c0));
            }
        }
        out.push('\n');
    }

    // Voltages
    if opts.show_all() {
        out.push_str("Voltages:\n");
        out.push_str(&format!("  VCore:          {:.3}V\n", table.core_voltage));
        out.push_str(&format!("  VSoC:           {:.3}V\n", table.soc_voltage));
    }

    out
}

pub fn format_json(table: &PmTable) -> String {
    serde_json::to_string_pretty(table).unwrap_or_else(|_| "{}".to_string())
}
